use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

mod wasm_math;
mod tree_structure;

#[wasm_bindgen]
pub struct TreeObject {
    pub seed: u32,
    pub trunk_height: f32,
    pub butressing: f32,
    pub split_height: f32,
    tree: tree_structure::TreeStructure,
}

#[wasm_bindgen]
pub struct Branch {
    pub length: f32,
    pub start_radius: f32,
    pub end_radius: f32,
    pub depth: u32,
    pub direction: wasm_math::Quaternion,
    pub start: wasm_math::Vector3d,
    pub end: wasm_math::Vector3d,
}

#[wasm_bindgen]
impl TreeObject {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, trunk_height: f32, butressing: f32) -> Result<TreeObject, JsValue> {
        let mut tree_obj = TreeObject {
            seed,
            trunk_height,
            butressing,
            split_height: trunk_height * 0.6, // Default split at 60% of height
            tree: tree_structure::TreeStructure::new(tree_structure::TreeSpecies {
                branching_angle_range: (0.3, 0.8),
                ring_spacing: 0.1,
                taper_rate: 0.9,
                max_branch_depth: 8,
            }),
        };
        
        tree_obj.regenerate_tree();
        Ok(tree_obj)
    }
    
    fn regenerate_tree(&mut self) {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        use glam::{Vec3, Quat};
        
        // Clear existing rings
        self.tree.rings.clear();
        
        let species = &self.tree.species_params;
        let _rng = SmallRng::seed_from_u64(self.seed as u64);
        
        // Create root ring
        let root_ring = tree_structure::TreeRing {
            center: Vec3::ZERO,
            radius: self.butressing,
            orientation: Quat::IDENTITY,
            parent_index: None,
            children_indices: Vec::new(),
        };
        self.tree.rings.push(root_ring);
        
        // Generate trunk rings up to split height
        let total_rings = (self.trunk_height / species.ring_spacing) as u32;
        let split_ring_index = (self.split_height / species.ring_spacing) as u32;
        
        // Single trunk rings before split
        for i in 1..=split_ring_index {
            let height = i as f32 * species.ring_spacing;
            let ring = tree_structure::TreeRing {
                center: Vec3::new(0.0, height, 0.0), // Y is up axis
                radius: self.butressing * (1.0 - height / self.trunk_height * 0.3),
                orientation: Quat::IDENTITY,
                parent_index: Some((i - 1) as usize),
                children_indices: Vec::new(),
            };
            if i > 1 {
                self.tree.rings[(i - 1) as usize].children_indices.push(i as usize);
            }
            self.tree.rings.push(ring);
        }
        
        // After split height, create two branches
        if split_ring_index < total_rings {
            let split_parent_idx = split_ring_index as usize;
            let branch_angle = 30.0_f32.to_radians(); // 30 degree branch angle
            let branch_separation = 0.3; // Distance between branch centers
            
            // Create branching rings from split point to top
            for i in (split_ring_index + 1)..total_rings {
                let height = i as f32 * species.ring_spacing;
                let height_above_split = height - self.split_height;
                let branch_radius = self.butressing * (1.0 - height / self.trunk_height * 0.4);
                
                // Calculate parent indices before creating rings
                let current_len = self.tree.rings.len();
                let left_parent_idx = if i == split_ring_index + 1 { 
                    split_parent_idx 
                } else { 
                    current_len - 2 
                };
                let right_parent_idx = if i == split_ring_index + 1 { 
                    split_parent_idx 
                } else { 
                    current_len - 1 
                };
                
                // Left branch
                let left_offset = Vec3::new(
                    -branch_separation - height_above_split * branch_angle.sin(),
                    height_above_split * (1.0 - branch_angle.cos() * 0.2),
                    0.0
                );
                let left_ring = tree_structure::TreeRing {
                    center: Vec3::new(0.0, self.split_height, 0.0) + left_offset,
                    radius: branch_radius,
                    orientation: Quat::from_rotation_y(-branch_angle * 0.5),
                    parent_index: Some(left_parent_idx),
                    children_indices: Vec::new(),
                };
                
                // Right branch  
                let right_offset = Vec3::new(
                    branch_separation + height_above_split * branch_angle.sin(),
                    height_above_split * (1.0 - branch_angle.cos() * 0.2),
                    0.0
                );
                let right_ring = tree_structure::TreeRing {
                    center: Vec3::new(0.0, self.split_height, 0.0) + right_offset,
                    radius: branch_radius,
                    orientation: Quat::from_rotation_y(branch_angle * 0.5),
                    parent_index: Some(right_parent_idx),
                    children_indices: Vec::new(),
                };
                
                // Store indices before pushing to avoid borrow conflicts
                let current_len = self.tree.rings.len();
                let left_ring_idx = current_len;
                let right_ring_idx = current_len + 1;
                
                // Update parent's children
                if i == split_ring_index + 1 {
                    // First branching rings - both are children of split parent
                    self.tree.rings[split_parent_idx].children_indices.push(left_ring_idx);
                    self.tree.rings[split_parent_idx].children_indices.push(right_ring_idx);
                } else {
                    // Subsequent rings - each branch continues from its previous ring
                    let left_parent_idx = current_len - 2;
                    let right_parent_idx = current_len - 1;
                    self.tree.rings[left_parent_idx].children_indices.push(left_ring_idx);
                    self.tree.rings[right_parent_idx].children_indices.push(right_ring_idx);
                }
                
                self.tree.rings.push(left_ring);
                self.tree.rings.push(right_ring);
            }
        }
    }

    pub fn set_trunk_height(&mut self, height: f32) {
        self.trunk_height = height;
        self.regenerate_tree();
    }

    pub fn set_butressing(&mut self, butressing: f32) {
        self.butressing = butressing;
        self.regenerate_tree();
    }

    pub fn set_split_height(&mut self, split_height: f32) {
        self.split_height = split_height;
        self.regenerate_tree();
    }

    pub fn render(&mut self) {
        // TODO: Generate mesh data for rendering
    }

    pub fn rings_count(&self) -> usize {
        self.tree.rings.len()
    }
    
    pub fn ring_center(&self, index: usize) -> Option<wasm_math::Vector3d> {
        self.tree.rings.get(index).map(|ring| {
            wasm_math::Vector3d::new(ring.center.x, ring.center.y, ring.center.z)
        })
    }
    
    pub fn ring_radius(&self, index: usize) -> Option<f32> {
        self.tree.rings.get(index).map(|ring| ring.radius)
    }
}

// Public API: generate a Tree
#[wasm_bindgen]
pub fn generate(seed: u32, trunk_height: f32, butressing: f32) -> Result<TreeObject, JsValue> {
    TreeObject::new(seed, trunk_height, butressing)
}

