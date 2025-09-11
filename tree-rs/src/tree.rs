use std::collections::{HashSet, VecDeque};

use micromath::{vector::Vector3d, Quaternion};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand_distr::{Normal, Uniform, Poisson};

// ----- Branch -----

const PRUNE_HEIGHT_THRESHOLD: f32 = 0.50;
const BARK_TO_GROWTH_RATIO: f32 = 0.00002;

#[derive(Debug)]
pub struct Branch {
    pub length: f32,
    pub radius: f32,
    pub depth: u32,
    pub direction: Quaternion,
    pub parent: Option<usize>,
    pub children: Vec<usize>, // Indices of children in the Tree's branches vector
    pub counter: u32,
    pub priority: f32,
    pub priority_change: f32, // experimental
    pub pruned: bool,
    pub previously_split: bool,

    pub _cache_valid: bool, // whether the cached values below are still valid or need to be recalculated
    pub _total_weight: f32, // (cached) total weight of this branch and all its ancestors
    pub _average_height: f32, // (cached) average height of all terminal branches
    pub _branch_end: Vector3d<f32>, // (cached) end position of this branch
}

impl Branch {
    pub fn new(direction: Quaternion, length: f32, radius: f32, depth: u32, counter: u32, priority: f32, parent: Option<usize>) -> Branch {
        Branch {
            length,
            radius,
            depth,
            direction,
            parent,
            children: Vec::new(),
            counter,
            priority,
            priority_change: 0.0,
            pruned: false,
            previously_split: false,

            _cache_valid: false,
            _total_weight: 0.0,
            _average_height: 0.0,
            _branch_end: Vector3d{ x: 0.0, y: 0.0, z: 0.0 },
        }
    }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
pub enum DistributionFamily {
    Normal,
    Uniform,
    Poisson,
}

#[derive(Debug)]
#[derive(Clone, Copy)]
pub struct Distribution {
    pub family: DistributionFamily,
    pub location: f32, // for Normal a.k.a. mean, µ
    pub scale: f32, // for Normal a.k.a. standard deviation, σ
}

// ----- Tree -----


pub struct Tree {
    pub seed: u32,
    pub segment_length: Distribution,
    pub straightness_priority: Distribution,
    pub max_depth: u32,
    pub angle: Distribution,
    pub branches: Vec<Branch>,
    pub root: usize,  // Index of the root branch
    pub rng: StdRng,
    pub growth: f32,
}

impl Tree {
    pub fn new(seed: u32, segment_length: Distribution, straightness_priority: Distribution, angle: Distribution, max_depth: u32) -> Tree {
        let mut tree = Tree {
            seed,
            segment_length,
            straightness_priority,
            max_depth,
            angle,
            branches: Vec::new(),
            root: 0,
            rng: StdRng::seed_from_u64(seed as u64),
            growth: 0.0,
        };
        tree.branches.push(Branch::new(Quaternion::new(1.0, 0.0, 0.0, 0.0), 0.1, 0.01, 0, 0, 1.0, None));
        tree
    }

    pub fn grow(&mut self, amount: f32) {
        let mut left = amount;
        if !self.branches.is_empty() {
            while left > 0.0 {
                let used = self.grow_branch(self.root, left);
                if used < 0.001 {
                    break;
                }
                left -= used;
            }
        }
        self.growth += amount;
        self.recalculate_values();
        // self.update_priorities();
        if self.growth > 3.0 {
            self.prune();
            self.growth = 0.0;
        }
    }

    fn recalculate_values(&mut self) {
        self.branches.iter_mut().for_each(|branch| branch._cache_valid = false);
        self.calculate_total_weight(self.root);
        self.calculate_average_height(self.root);
    }

    fn calculate_total_weight(&mut self, branch_idx: usize) -> f32 {
        // Temporarily take the data we need without holding the mutable borrow
        let (radius, length, children_is_empty, children) = {
            let branch = &self.branches[branch_idx];
            (
                branch.radius,
                branch.length,
                branch.children.is_empty(),
                branch.children.clone(), // clone indices so we can recurse safely
            )
        };
    
        let total_weight = if children_is_empty {
            std::f32::consts::PI * radius * radius * length
        } else {
            let mut sum = 0.0;
            for child_idx in children {
                sum += self.calculate_total_weight(child_idx);
            }
            sum
        };
    
        // Now we borrow mutably again to write the result
        self.branches[branch_idx]._total_weight = total_weight;
        total_weight
    }
    
    fn calculate_average_height(&mut self, branch_idx: usize) -> f32 {
        // Extract the data we need without keeping a borrow across recursion
        let children = {
            let branch = &self.branches[branch_idx];
            branch.children.clone()
        };
    
        let average_height = if children.is_empty() {
            self.branch_end(branch_idx).y
        } else {
            let mut sum = 0.0;
            for child_idx in &children {
                sum += self.calculate_average_height(*child_idx);
            }
            sum / children.len() as f32
        };
    
        // Mutably borrow again just to store the result
        self.branches[branch_idx]._average_height = average_height;
        average_height
    }    
    
    fn update_priorities(&mut self) {
        const TARGET_HEIGHT: f32 = 10.0;
        let mut heights = Vec::with_capacity(self.branches.len());
    
        // First pass: collect heights using indices (no &self.branches borrow held)
        for idx in 0..self.branches.len() {
            heights.push(self.branches[idx]._average_height);
        }
    
        // Compute median
        let mut sorted_heights = heights.clone();
        sorted_heights.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
        // Second pass: update priority_change
        for idx in 0..self.branches.len() {
            let branch = &mut self.branches[idx];
            if branch.children.is_empty() {
                let diff = heights[idx] - TARGET_HEIGHT;
                branch.priority_change = -diff * 0.1;
            } else {
                branch.priority_change = 0.0;
            }
        }
    
        // Third pass: apply priority change
        for branch in &mut self.branches {
            branch.priority += branch.priority_change;
        }
    }    

    fn prune(&mut self) {
        // Prune the child that:
        //  - has average height below 50% of the max height
        //  - has lower depth than the other child(ren)

        let max_height = self
            .branches
            .iter()
            .map(|branch| branch._average_height)
            .fold(0.0, f32::max);
    
        let mut initial_to_prune: Vec<usize> = Vec::new();

        // Step 1: look at children of every branch
        for parent in &self.branches {
            if parent.children.is_empty() {
                continue;
            }
            if parent.depth >= 2 {
                continue;
            }

            // find minimum depth among siblings
            let min_depth = parent
                .children
                .iter()
                .map(|&child_idx| self.branches[child_idx].depth)
                .min()
                .unwrap();

            // select children for pruning based on the two conditions
            for &child_idx in &parent.children {
                let child = &self.branches[child_idx];
                if child._branch_end.y < max_height * PRUNE_HEIGHT_THRESHOLD && child.depth > min_depth {
                    initial_to_prune.push(child_idx);
                }
            }
        }

        // Step 2: collect descendants of everything selected
        let mut all_to_prune: HashSet<usize> = HashSet::new();
        let mut queue: VecDeque<usize> = initial_to_prune.into();

        while let Some(idx) = queue.pop_front() {
            if all_to_prune.insert(idx) {
                for &child_idx in &self.branches[idx].children {
                    queue.push_back(child_idx);
                }
            }
        }

        // Step 3: mark pruned
        for &idx in &all_to_prune {
            self.branches[idx].pruned = true;
        }

        // Step 4: update parents' children lists
        for &idx in &all_to_prune {
            if let Some(parent_idx) = self.branches[idx].parent {
                let parent = &mut self.branches[parent_idx];
                parent.children.retain(|&x| x != idx);
            }
        }
    }

    fn grow_branch(&mut self, branch_idx: usize, amount: f32) -> f32 {
        // if amount < 0.001 {
        //     self.branches[branch_idx].counter += 1;
        //     return amount;
        // }
        let mut used = 0.0;
        let children: Vec<usize> = self.branches[branch_idx].children.iter().copied().filter(|&child_idx| !self.branches[child_idx].pruned).collect();
        let min_child_counter = children.iter().map(|&child_idx| self.branches[child_idx].counter).min().unwrap_or(0);
        let sampled_segment_length = self.sample_segment_length();
        let sampled_angle_a = self.sample_angle();
        let sampled_straightness = self.sample_straightness_priority();
        let parent_counter = {
            let has_parent = self.branches[branch_idx].parent.is_some();
            if has_parent {
                i64::from(self.branches[self.branches[branch_idx].parent.unwrap()].counter)
            } else {
                -1
            }
        };
        let relative_priority = self.branch_relative_priority(branch_idx);
        let branch = &mut self.branches[branch_idx];
        let should_grow = (min_child_counter == 0 || min_child_counter >= branch.counter) && (parent_counter > i64::from(branch.counter) || parent_counter == -1);
        if should_grow {
            branch.counter += 1;
        }
        if !children.is_empty() {
            if should_grow {
                // Widen this branch
                let r2 = branch.radius + BARK_TO_GROWTH_RATIO * amount;
                let r1 = branch.radius;
                used = std::f32::consts::PI * (r2 * r2 - r1 * r1) * branch.length;
                branch.radius = r2;
            }
            let total_priority: f32 = children.iter().map(|&child_idx| self.branches[child_idx].priority).sum();
            // let available_per_child = (amount - used) / children.len() as f32;
            for &child_idx in &children {
                let p = self.branches[child_idx].priority;
                let available_to_branch = (amount - used) * p / total_priority;
                used += self.grow_branch(child_idx, available_to_branch);
            }
        } else if branch.previously_split {
            // This branch has already been split: only widen it
            // branch.length += amount * 0.1;
            // branch.radius += 0.01;
            // branch.counter += 1;
        } else if branch.length < sampled_segment_length && should_grow {
            // This is a terminal branch: lengthen it
            let max_growth = sampled_segment_length - branch.length;
            used = f32::min(amount, max_growth);
            branch.length += used;
        } else if branch.depth < self.max_depth && should_grow {
            // Split this branch into two
            let direction: Quaternion = branch.direction;
            
            let r: f32 = self.rng.random();
            let mut v = sampled_angle_a;
            if r < 0.5 {
             v = -v
            }
            let direction_a = direction * Quaternion::new(1.0, 0.0, 0.0, 0.0 );
            let direction_b = direction * Quaternion::new(1.0, 0.0, 0.0, v);

            // Create new branches with counter one less than parent to ensure they're always behind
            let child_counter = branch.counter.saturating_sub(1);
            let new_branch_a = Branch::new(direction_a, 0.0, 0.001, branch.depth, child_counter, sampled_straightness, Some(branch_idx));
            let new_branch_b = Branch::new(direction_b, 0.0, 0.001, branch.depth + 1, child_counter, 1.0, Some(branch_idx));
            
            // Add new branches to the tree and get their indices
            let new_idx_a = self.branches.len();
            self.branches.push(new_branch_a);
            let new_idx_b = self.branches.len();
            self.branches.push(new_branch_b);
            
            // Update children of current branch
            // We need to get the branch again to avoid multiple mutable borrows
            let parent = &mut self.branches[branch_idx];
            parent.children.push(new_idx_a);
            parent.children.push(new_idx_b);
            parent.previously_split = true;
            used += self.grow_branch(new_idx_a, amount * 0.5);
            used += self.grow_branch(new_idx_b, amount * 0.5);
        } else if should_grow {
            branch.previously_split = true; // prevent further growth
            // branch.previously_split = true; // Mark this branch as previously split to prevent further growth
        }
        used
    }

    pub fn branch_relative_priority(&self, branch_idx: usize) -> f32 {
        let parent = self.branches[branch_idx].parent;
        if parent.is_none() {
            return 1.0;
        }
        let total_priority: f32 = self.branches[parent.unwrap()].children.iter().map(|&child_idx| self.branches[child_idx].priority).sum();
        self.branches[branch_idx].priority / total_priority
    }

    pub fn branch_start(&mut self, branch_idx: usize) -> Vector3d<f32> {
        if let Some(parent_idx) = self.branches[branch_idx].parent {
            self.branch_end(parent_idx)
        } else {
            Vector3d{ x: 0.0, y: 0.0, z: 0.0 }
        }
    }

    pub fn branch_end(&mut self, branch_idx: usize) -> Vector3d<f32> {
        let (direction, length, cache_valid) = {
            let branch = &self.branches[branch_idx];
            (branch.direction, branch.length, branch._cache_valid)
        };
    
        if !cache_valid {
            let end = self.branch_start(branch_idx)
                + direction * Vector3d { x: 0.0, y: length, z: 0.0 };
            self.branches[branch_idx]._branch_end = end;
            self.branches[branch_idx]._cache_valid = true;
        }
    
        self.branches[branch_idx]._branch_end
    }
    

    fn sample(&mut self, dist: &Distribution) -> f32 {
        match dist.family {
            DistributionFamily::Normal => self.rng.sample(&Normal::new(dist.location, dist.scale).unwrap()),
            DistributionFamily::Uniform => self.rng.sample(&Uniform::new(dist.location - dist.scale, dist.location + dist.scale).unwrap()),
            DistributionFamily::Poisson => self.rng.sample(&Poisson::new(dist.location).unwrap()),
        }
    }

    fn sample_segment_length(&mut self) -> f32 {
        let segment_length = self.segment_length;
        self.sample(&segment_length)
    }

    fn sample_straightness_priority(&mut self) -> f32 {
        let straightness_priority = self.straightness_priority;
        self.sample(&straightness_priority)
    }

    fn sample_angle(&mut self) -> f32 {
        let angle = self.angle;
        self.sample(&angle)
    }
}