use micromath::{vector::Vector3d, Quaternion};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand_distr::{Normal, Uniform, Poisson};

// ----- Branch -----

#[derive(Debug)]
pub struct Branch {
    pub index: usize,
    pub length: f32,
    pub radius: f32,
    pub direction: Quaternion,
    pub parent: Option<usize>,
    pub children: Vec<usize>, // Indices of children in the Tree's branches vector
    pub counter: u32,
    pub priority: f32,
}

impl Branch {
    pub fn new(index: usize, direction: Quaternion, length: f32, radius: f32, priority: f32, parent: Option<usize>) -> Branch {
        Branch {
            index,
            length,
            radius,
            direction,
            parent,
            children: Vec::new(),
            counter: 0,
            priority,
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
    pub angle: Distribution,
    pub branches: Vec<Branch>,
    pub root: usize,  // Index of the root branch
    pub rng: StdRng,
}

impl Tree {
    pub fn new(seed: u32, segment_length: Distribution, straightness_priority: Distribution, angle: Distribution) -> Tree {
        let mut tree = Tree {
            seed,
            segment_length,
            straightness_priority,
            angle,
            branches: Vec::new(),
            root: 0,
            rng: StdRng::seed_from_u64(seed as u64),
        };
        tree.branches.push(Branch::new(0, Quaternion::new(1.0, 0.0, 0.0, 0.0), 0.1, 0.01, 1.0, None));
        tree
    }

    pub fn grow(&mut self, amount: f32) {
        if !self.branches.is_empty() {
            self.grow_branch(self.root, amount);
        }
    }

    fn grow_branch(&mut self, branch_idx: usize, amount: f32) {
        if amount < 0.001 {
            return;
        }
        let branches_len = self.branches.len();
        let min_child_counter = self.branches[branch_idx].children.iter().map(|&child_idx| self.branches[child_idx].counter).min().unwrap_or(0);
        let _segment_length = self.sample_segment_length();
        let sampled_angle = self.sample_angle();
        let sampled_straightness = self.sample_straightness_priority();
        let branch = &mut self.branches[branch_idx];
        if !branch.children.is_empty() {
            let should_grow = min_child_counter == branch.counter;
            let mut used = 0.0;
            if should_grow {
                // Widen this branch, and grow children
                let r2 = branch.radius + 0.01;
                let r1 = branch.radius;
                used = std::f32::consts::PI * (r2 * r2 - r1 * r1) * branch.length;
                branch.radius += 0.01;
                branch.counter += 1;
                // experiment: increase angle with age
                // branch.direction = branch.direction * branch.direction;
            }
            let children: Vec<usize> = branch.children.iter().copied().collect();
            let total_priority: f32 = children.iter().map(|&child_idx| self.branches[child_idx].priority).sum();
            // let available_per_child = (amount - used) / children.len() as f32;
            for &child_idx in &children {
                let p = self.branches[child_idx].priority;
                let available_to_branch = (amount - used) * p / total_priority;
                self.grow_branch(child_idx, available_to_branch);
            }
        } else if branch.length < _segment_length {
            // This is a terminal branch: lengthen it
            branch.length += amount * 0.1; //  / (std::f32::consts::PI * branch.radius * branch.radius);
        } else {
            // Split this branch into two
            let direction: Quaternion = branch.direction;
            
            let direction_a = direction * Quaternion::new(1.0, 0.0, 0.0, 0.0);
            let r: f32 = self.rng.random();
            let mut v = sampled_angle;
            if r < 0.5 {
             v = -v;
            }
            let direction_b = direction * Quaternion::new(1.0, 0.0, 0.0, v);

            // Create new branches
            let new_branch_a = Branch::new(branches_len, direction_a, 0.0, 0.01, sampled_straightness, Some(branch_idx));
            let new_branch_b = Branch::new(branches_len + 1, direction_b, 0.0, 0.01, 1.0, Some(branch_idx));
            
            // Add new branches to the tree and get their indices
            let new_idx_a = self.branches.len();
            self.branches.push(new_branch_a);
            let new_idx_b = self.branches.len();
            self.branches.push(new_branch_b);
            
            // Update children of current branch
            // We need to get the branch again to avoid multiple mutable borrows
            self.branches.get_mut(branch_idx).unwrap().children.push(new_idx_a);
            self.branches.get_mut(branch_idx).unwrap().children.push(new_idx_b);
        }
    }

    pub fn branch_start(&self, branch_idx: usize) -> Vector3d<f32> {
        if let Some(parent_idx) = self.branches[branch_idx].parent {
            self.branch_end(parent_idx)
        } else {
            Vector3d{ x: 0.0, y: 0.0, z: 0.0 }
        }
    }

    pub fn branch_end(&self, branch_idx: usize) -> Vector3d<f32> {
        let branch = &self.branches[branch_idx];
        self.branch_start(branch_idx) + branch.direction * Vector3d{ x: 0.0, y: branch.length, z: 0.0 }
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