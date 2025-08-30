use micromath::{vector::Vector3d, Quaternion};

// ----- Branch -----

#[derive(Debug)]
pub struct Branch {
    pub index: usize,
    pub length: f32,
    pub radius: f32,
    pub direction: Quaternion,
    pub parent: Option<usize>,
    pub children: Vec<usize>,  // Indices of children in the Tree's branches vector
}

impl Branch {
    pub fn new(index: usize, direction: Quaternion, length: f32, radius: f32, parent: Option<usize>) -> Branch {
        Branch {
            index,
            length,
            radius,
            direction,
            parent,
            children: Vec::new(),
        }
    }
}

// ----- Tree -----

#[derive(Debug)]
pub struct Tree {
    pub seed: u32,
    pub age: f32,
    pub branches: Vec<Branch>,
    pub root: usize,  // Index of the root branch
}

impl Tree {
    pub fn new(seed: u32, age: f32) -> Tree {
        let mut tree = Tree {
            seed,
            age,
            branches: Vec::new(),
            root: 0,
        };
        tree.branches.push(Branch::new(0, Quaternion::new(1.0, 0.0, 0.0, 0.0), 0.1, 0.01, None));
        tree
    }

    pub fn grow(&mut self, amount: f32) {
        self.age += amount;
        if !self.branches.is_empty() {
            self.grow_branch(self.root, amount);
        }
    }

    fn grow_branch(&mut self, branch_idx: usize, amount: f32) {
        let branch = &mut self.branches[branch_idx];
        if !branch.children.is_empty() {
            // Widen this branch, and grow children
            branch.radius += amount * 0.01;
            let children: Vec<usize> = branch.children.iter().copied().collect();
            for &child_idx in &children {
                self.grow_branch(child_idx, amount * 0.99);
            }
        } else if branch.length < 0.5 {
            // This is a leaf: lengthen it
            branch.length += amount * 0.1;
        } else {
            // Split this branch into two
            let direction: Quaternion = branch.direction;
            
            let direction_a = direction * Quaternion::new(1.0, 0.0, 0.0, 0.1);
            let direction_b = direction * Quaternion::new(1.0, 0.0, 0.0, -0.1);

            // Create new branches
            let new_branch_a = Branch::new(self.branches.len(), direction_a, 0.0, 0.01, Some(branch_idx));
            let new_branch_b = Branch::new(self.branches.len() + 1, direction_b, 0.0, 0.01, Some(branch_idx));
            
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
}