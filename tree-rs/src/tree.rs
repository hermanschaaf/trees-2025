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
    pub trunk_height: f32,
    pub butressing: f32,
    pub branches: Vec<Branch>,
    pub root: usize,  // Index of the root branch
    pub rng: StdRng,
}

impl Tree {
    pub fn new(seed: u32, trunk_height: f32, butressing: f32) -> Tree {
        let mut tree = Tree {
            seed,
            trunk_height,
            butressing,
            branches: Vec::new(),
            root: 0,
            rng: StdRng::seed_from_u64(seed as u64),
        };
        tree.branches.push(Branch::new(Quaternion::new(1.0, 0.0, 0.0, 0.0), 3.0, 1.0, 0, 0, 1.0, None));
        tree
    }

    pub fn grow(&mut self) {
        // TODO
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
}