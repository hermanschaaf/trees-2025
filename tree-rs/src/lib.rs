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
        
        // Create exactly 5 rings for simple tree structure
        // Ring 0: Root (already created)
        // Ring 1: Lower trunk  
        // Ring 2: Split point
        // Ring 3: Left branch end
        // Ring 4: Right branch end
        
        // Ring 1: Lower trunk
        let lower_height = self.split_height * 0.5;
        let lower_ring = tree_structure::TreeRing {
            center: Vec3::new(0.0, lower_height, 0.0),
            radius: self.butressing * 0.9,
            orientation: Quat::IDENTITY,
            parent_index: Some(0),
            children_indices: Vec::new(),
        };
        self.tree.rings[0].children_indices.push(1);
        self.tree.rings.push(lower_ring);
        
        // Ring 2: Split point
        let split_ring = tree_structure::TreeRing {
            center: Vec3::new(0.0, self.split_height, 0.0),
            radius: self.butressing * 0.7,
            orientation: Quat::IDENTITY,
            parent_index: Some(1),
            children_indices: Vec::new(),
        };
        self.tree.rings[1].children_indices.push(2);
        self.tree.rings.push(split_ring);
        
        // Ring 3 & 4: Branch ends  
        let branch_angle = 30.0_f32.to_radians();
        let branch_length = self.trunk_height - self.split_height;
        let branch_separation = 0.4;
        
        // Ring 3: Left branch end
        let left_end = Vec3::new(
            -branch_separation - branch_length * branch_angle.sin(),
            self.split_height + branch_length * branch_angle.cos(),
            0.0
        );
        let left_branch_ring = tree_structure::TreeRing {
            center: left_end,
            radius: self.butressing * 0.4,
            orientation: Quat::from_rotation_y(-branch_angle * 0.5),
            parent_index: Some(2), // Child of split ring
            children_indices: Vec::new(),
        };
        
        // Ring 4: Right branch end
        let right_end = Vec3::new(
            branch_separation + branch_length * branch_angle.sin(),
            self.split_height + branch_length * branch_angle.cos(),
            0.0
        );
        let right_branch_ring = tree_structure::TreeRing {
            center: right_end,
            radius: self.butressing * 0.4,
            orientation: Quat::from_rotation_y(branch_angle * 0.5),
            parent_index: Some(2), // Child of split ring
            children_indices: Vec::new(),
        };
        
        // Connect split ring to both branch rings
        self.tree.rings[2].children_indices.push(3);
        self.tree.rings[2].children_indices.push(4);
        
        self.tree.rings.push(left_branch_ring);
        self.tree.rings.push(right_branch_ring);
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

