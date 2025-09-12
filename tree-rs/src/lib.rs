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
    pub segment_length: f32,
    pub branch_angle_min: f32,
    pub branch_angle_max: f32,
    pub bend_angle_min: f32,
    pub bend_angle_max: f32,
    pub branch_frequency_min: u32,
    pub branch_frequency_max: u32,
    pub max_depth: u32,
    pub radius_taper: f32,
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
pub struct TreeMesh {
    vertices: Vec<f32>,
    normals: Vec<f32>,
    uvs: Vec<f32>,
    indices: Vec<u32>,
}

#[wasm_bindgen]
impl TreeMesh {
    #[wasm_bindgen(getter)]
    pub fn vertices(&self) -> Vec<f32> {
        self.vertices.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn normals(&self) -> Vec<f32> {
        self.normals.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn uvs(&self) -> Vec<f32> {
        self.uvs.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn indices(&self) -> Vec<u32> {
        self.indices.clone()
    }
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
            segment_length: 0.3,
            branch_angle_min: 25.0,
            branch_angle_max: 45.0,
            bend_angle_min: -15.0,
            bend_angle_max: 15.0,
            branch_frequency_min: 2,
            branch_frequency_max: 4,
            max_depth: 6,
            radius_taper: 0.8,
            tree: tree_structure::TreeStructure::new(tree_structure::TreeSpecies {
                branching_angle_range: (0.3, 0.8),
                ring_spacing: trunk_height / 3.0, // Only 3-4 rings for trunk (root, mid, split, top)
                taper_rate: 0.9,
                max_branch_depth: 8,
            }),
        };
        
        tree_obj.regenerate_tree();
        Ok(tree_obj)
    }
    
    fn regenerate_tree(&mut self) {
        use rand::{Rng, SeedableRng};
        use rand::rngs::SmallRng;
        use glam::{Vec3, Quat};
        
        // Tree generation parameters from instance
        let segment_length = self.segment_length;
        let branch_angle_range = (self.branch_angle_min, self.branch_angle_max);
        let bend_angle_range = (self.bend_angle_min, self.bend_angle_max);
        let branch_frequency_range = (self.branch_frequency_min, self.branch_frequency_max);
        let max_depth = self.max_depth;
        let radius_taper = self.radius_taper;
        
        // Clear existing rings
        self.tree.rings.clear();
        
        let mut rng = SmallRng::seed_from_u64(self.seed as u64);
        
        // Create root ring
        let root_ring = tree_structure::TreeRing {
            center: Vec3::ZERO,
            radius: self.butressing,
            orientation: Quat::IDENTITY,
            parent_index: None,
            children_indices: Vec::new(),
        };
        self.tree.rings.push(root_ring);
        
        // Start recursive generation from root
        Self::generate_branch_recursive_static(
            &mut self.tree.rings,
            0,                    // parent_ring_index
            Vec3::new(0.0, 1.0, 0.0), // growth_direction (up)
            self.butressing,      // current_radius
            0,                    // depth
            0,                    // segments_since_branch
            &mut rng,
            segment_length,
            branch_angle_range,
            bend_angle_range,
            branch_frequency_range,
            max_depth,
            radius_taper,
        );
    }
    
    fn generate_branch_recursive_static(
        rings: &mut Vec<tree_structure::TreeRing>,
        parent_index: usize,
        growth_direction: glam::Vec3,
        current_radius: f32,
        depth: u32,
        segments_since_branch: u32,
        rng: &mut rand::rngs::SmallRng,
        segment_length: f32,
        branch_angle_range: (f32, f32),
        bend_angle_range: (f32, f32),
        branch_frequency_range: (u32, u32),
        max_depth: u32,
        radius_taper: f32,
    ) {
        use rand::Rng;
        use glam::{Vec3, Quat};
        
        // Stop recursion if too deep or radius too small
        if depth >= max_depth || current_radius < 0.05 {
            return;
        }
        
        // Calculate next ring position
        let parent_center = rings[parent_index].center;
        
        // Add some bend to the growth direction for natural curves
        let bend_angle = rng.gen_range(bend_angle_range.0..=bend_angle_range.1).to_radians();
        let bend_axis = Vec3::new(rng.gen_range(-1.0..=1.0), 0.0, rng.gen_range(-1.0..=1.0)).normalize();
        let bend_rotation = Quat::from_axis_angle(bend_axis, bend_angle);
        let bent_direction = (bend_rotation * growth_direction).normalize();
        
        let next_center = parent_center + bent_direction * segment_length;
        let next_radius = current_radius * 0.95; // Slight taper per segment
        
        // Create the next ring
        let ring = tree_structure::TreeRing {
            center: next_center,
            radius: next_radius,
            orientation: Quat::from_rotation_arc(Vec3::Y, bent_direction),
            parent_index: Some(parent_index),
            children_indices: Vec::new(),
        };
        
        let ring_index = rings.len();
        rings[parent_index].children_indices.push(ring_index);
        rings.push(ring);
        
        // Determine if branching should occur
        let should_branch = segments_since_branch >= rng.gen_range(branch_frequency_range.0..=branch_frequency_range.1);
        
        if should_branch && depth < max_depth - 1 {
            // Create two branches
            let branch_angle = rng.gen_range(branch_angle_range.0..=branch_angle_range.1).to_radians();
            let branch_radius = current_radius * radius_taper;
            
            // Generate random branch directions
            let up_component = Vec3::Y;
            let perpendicular = if bent_direction.cross(up_component).length() > 0.1 {
                bent_direction.cross(up_component).normalize()
            } else {
                Vec3::X
            };
            
            // Left branch direction
            let left_rotation = Quat::from_axis_angle(perpendicular, branch_angle);
            let left_direction = (left_rotation * bent_direction).normalize();
            
            // Right branch direction  
            let right_rotation = Quat::from_axis_angle(perpendicular, -branch_angle);
            let right_direction = (right_rotation * bent_direction).normalize();
            
            // Recursively generate both branches
            Self::generate_branch_recursive_static(
                rings,
                ring_index,
                left_direction,
                branch_radius,
                depth + 1,
                0, // Reset segment counter
                rng,
                segment_length,
                branch_angle_range,
                bend_angle_range,
                branch_frequency_range,
                max_depth,
                radius_taper,
            );
            
            Self::generate_branch_recursive_static(
                rings,
                ring_index,
                right_direction,
                branch_radius,
                depth + 1,
                0, // Reset segment counter
                rng,
                segment_length,
                branch_angle_range,
                bend_angle_range,
                branch_frequency_range,
                max_depth,
                radius_taper,
            );
        } else {
            // Continue growing this branch
            Self::generate_branch_recursive_static(
                rings,
                ring_index,
                bent_direction,
                next_radius,
                depth,
                segments_since_branch + 1,
                rng,
                segment_length,
                branch_angle_range,
                bend_angle_range,
                branch_frequency_range,
                max_depth,
                radius_taper,
            );
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

    pub fn set_segment_length(&mut self, segment_length: f32) {
        self.segment_length = segment_length;
        self.regenerate_tree();
    }

    pub fn set_branch_angle_range(&mut self, min: f32, max: f32) {
        self.branch_angle_min = min;
        self.branch_angle_max = max;
        self.regenerate_tree();
    }

    pub fn set_bend_angle_range(&mut self, min: f32, max: f32) {
        self.bend_angle_min = min;
        self.bend_angle_max = max;
        self.regenerate_tree();
    }

    pub fn set_branch_frequency_range(&mut self, min: u32, max: u32) {
        self.branch_frequency_min = min;
        self.branch_frequency_max = max;
        self.regenerate_tree();
    }

    pub fn set_max_depth(&mut self, max_depth: u32) {
        self.max_depth = max_depth;
        self.regenerate_tree();
    }

    pub fn set_radius_taper(&mut self, radius_taper: f32) {
        self.radius_taper = radius_taper;
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
    
    pub fn generate_tree_mesh(&self, resolution: u32) -> TreeMesh {
        let ring_mesh = self.tree.generate_mesh(resolution);
        
        // Convert Vec3 vertices to flat f32 array
        let mut vertices = Vec::with_capacity(ring_mesh.vertices.len() * 3);
        for vertex in &ring_mesh.vertices {
            vertices.push(vertex.x);
            vertices.push(vertex.y);
            vertices.push(vertex.z);
        }
        
        // Convert Vec3 normals to flat f32 array
        let mut normals = Vec::with_capacity(ring_mesh.normals.len() * 3);
        for normal in &ring_mesh.normals {
            normals.push(normal.x);
            normals.push(normal.y);
            normals.push(normal.z);
        }
        
        // Convert Vec2 UVs to flat f32 array
        let mut uvs = Vec::with_capacity(ring_mesh.uvs.len() * 2);
        for uv in &ring_mesh.uvs {
            uvs.push(uv.x);
            uvs.push(uv.y);
        }
        
        TreeMesh {
            vertices,
            normals,
            uvs,
            indices: ring_mesh.indices,
        }
    }
}

// Public API: generate a Tree
#[wasm_bindgen]
pub fn generate(seed: u32, trunk_height: f32, butressing: f32) -> Result<TreeObject, JsValue> {
    TreeObject::new(seed, trunk_height, butressing)
}

