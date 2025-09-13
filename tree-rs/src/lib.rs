use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use glam::Vec2;

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
    pub trunk_ring_spread: f32, // 0.0-2.0: how spread out trunk rings are
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
            trunk_ring_spread: 0.5,   // Default moderate spread
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
        // Calculate segment length based on trunk height to control number of trunk segments
        let target_trunk_segments = (self.trunk_height * 3.0).max(3.0) as u32; // ~3 segments per unit height, minimum 3
        let trunk_segment_length = self.trunk_height / target_trunk_segments as f32;
        
        let branch_angle_range = (self.branch_angle_min, self.branch_angle_max);
        let bend_angle_range = (self.bend_angle_min, self.bend_angle_max);
        let branch_frequency_range = (self.branch_frequency_min, self.branch_frequency_max);
        let max_depth = self.max_depth;
        let radius_taper = self.radius_taper;
        
        // Clear existing cross-sections
        self.tree.cross_sections.clear();
        
        let mut rng = SmallRng::seed_from_u64(self.seed as u64);
        
        // Create root cross-section with multiple rings based on buttressing
        let trunk_rings = self.generate_trunk_rings();
        
        // Calculate initial trunk radius before moving trunk_rings
        let initial_trunk_radius = if !trunk_rings.is_empty() {
            trunk_rings[0].radius
        } else {
            0.5 // fallback
        };
        
        let root_cross_section = tree_structure::BranchCrossSection {
            center: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            component_rings: trunk_rings,
            parent_index: None,
            children_indices: Vec::new(),
        };
        self.tree.cross_sections.push(root_cross_section);
        
        // Start recursive generation from ALL root rings
        for ring_index in 0..self.tree.cross_sections[0].component_rings.len() {
            let root_ring_id = tree_structure::RingId {
                cross_section_index: 0,
                ring_index,
            };
            
            let ring_radius = self.tree.cross_sections[0].component_rings[ring_index].radius;
            
            Self::generate_branch_recursive_static(
                &mut self.tree.cross_sections,
                root_ring_id,         // parent_ring_id for THIS specific ring
                Vec3::new(0.0, 1.0, 0.0), // growth_direction (up)
                ring_radius,          // Use THIS ring's radius
                0,                    // depth
                0,                    // segments_since_branch
                &mut rng,
                trunk_segment_length, // Use calculated trunk segment length
                branch_angle_range,
                bend_angle_range,
                branch_frequency_range,
                max_depth,
                radius_taper,
                self.trunk_height,    // Pass trunk height for branching logic
            );
        }
    }
    
    fn generate_trunk_rings(&self) -> Vec<tree_structure::ComponentRing> {
        use std::f32::consts::PI;
        
        // Calculate number of rings based on buttressing value
        // buttressing of 0.5 = 1 ring, 1.0 = 3 rings, 1.5 = 4 rings, 2.0 = 5 rings, etc.
        let ring_count = if self.butressing < 0.7 {
            1
        } else {
            ((self.butressing - 0.5) * 3.0 + 1.0).max(1.0) as u32
        };
        let mut rings = Vec::new();
        
        // Fixed base trunk radius - overall segment should maintain this cross-sectional area
        let base_trunk_radius = 0.5;
        let target_area = PI * base_trunk_radius * base_trunk_radius;
        
        if ring_count == 1 {
            // Single central ring - maintains full area
            rings.push(tree_structure::ComponentRing {
                offset: Vec2::ZERO,
                radius: base_trunk_radius,
                ring_type: tree_structure::RingType::MainTrunk,
                parent_ring_id: None,
                children_ring_ids: Vec::new(),
            });
        } else {
            // Multiple rings - adjust radii to maintain total cross-sectional area
            // Each ring gets equal share of the total area
            let area_per_ring = target_area / ring_count as f32;
            let individual_ring_radius = (area_per_ring / PI).sqrt();
            
            // Central ring
            rings.push(tree_structure::ComponentRing {
                offset: Vec2::ZERO,
                radius: individual_ring_radius,
                ring_type: tree_structure::RingType::MainTrunk,
                parent_ring_id: None,
                children_ring_ids: Vec::new(),
            });
            
            // Surrounding rings - all same size to maintain equal area
            let outer_rings = ring_count - 1;
            let spread_distance = base_trunk_radius * self.trunk_ring_spread;
            
            for i in 0..outer_rings {
                let angle = (i as f32 / outer_rings as f32) * 2.0 * PI;
                let offset = Vec2::new(
                    angle.cos() * spread_distance,
                    angle.sin() * spread_distance,
                );
                
                rings.push(tree_structure::ComponentRing {
                    offset,
                    radius: individual_ring_radius, // Same radius for all rings
                    ring_type: tree_structure::RingType::SideBranch,
                    parent_ring_id: None,
                    children_ring_ids: Vec::new(),
                });
            }
        }
        
        rings
    }
    
    fn create_child_ring_from_parent(
        parent_ring: &tree_structure::ComponentRing,
        parent_ring_id: tree_structure::RingId,
        taper_ratio: f32,
    ) -> tree_structure::ComponentRing {
        // Child ring inherits parent position exactly and only applies taper to radius
        tree_structure::ComponentRing {
            offset: parent_ring.offset, // Inherit exact position
            radius: parent_ring.radius * taper_ratio, // Apply only taper
            ring_type: parent_ring.ring_type.clone(),
            parent_ring_id: Some(parent_ring_id), // Track specific parent
            children_ring_ids: Vec::new(),
        }
    }
    
    fn create_branching_rings_in_same_section(
        parent_rings: &[tree_structure::ComponentRing],
    ) -> Vec<tree_structure::ComponentRing> {
        let mut branching_rings = Vec::new();
        
        // For each parent ring, create TWO rings in close proximity but slightly offset
        // This creates the "branching preparation" where rings split before actual branching
        for parent_ring in parent_rings {
            // Create two rings slightly offset from the parent position
            let offset_distance = parent_ring.radius * 0.15; // Small offset proportional to radius
            
            // First ring (trunk continuation) - slight offset in one direction
            branching_rings.push(tree_structure::ComponentRing {
                offset: parent_ring.offset + Vec2::new(offset_distance, 0.0),
                radius: parent_ring.radius, // Keep exact parent radius
                ring_type: parent_ring.ring_type.clone(),
                parent_ring_id: None,
                children_ring_ids: Vec::new(),
            });
            
            // Second ring (branch preparation) - slight offset in opposite direction
            branching_rings.push(tree_structure::ComponentRing {
                offset: parent_ring.offset + Vec2::new(-offset_distance, 0.0),
                radius: parent_ring.radius, // Keep exact parent radius
                ring_type: tree_structure::RingType::SideBranch,
                parent_ring_id: None,
                children_ring_ids: Vec::new(),
            });
        }
        
        branching_rings
    }
    
    fn split_rings_for_branching(
        parent_rings: &[tree_structure::ComponentRing],
        current_height: f32,
        trunk_height: f32,
        radius_taper: f32,
    ) -> (Vec<tree_structure::ComponentRing>, Vec<tree_structure::ComponentRing>) {
        let mut trunk_continuation_rings = Vec::new();
        let mut side_branch_rings = Vec::new();
        
        // Split the rings based on their type and position
        // Rings that are marked as trunk continuation go to trunk, others to branch
        for (i, ring) in parent_rings.iter().enumerate() {
            if ring.ring_type == tree_structure::RingType::MainTrunk || i % 2 == 0 {
                // Trunk rings and even-indexed rings continue as trunk
                trunk_continuation_rings.push(tree_structure::ComponentRing {
                    offset: ring.offset,
                    radius: ring.radius,
                    ring_type: tree_structure::RingType::MainTrunk,
                    parent_ring_id: None,
                    children_ring_ids: Vec::new(),
                });
            } else {
                // Odd-indexed rings become branches
                side_branch_rings.push(tree_structure::ComponentRing {
                    offset: Vec2::ZERO, // Branches start from center of new cross-section
                    radius: ring.radius,
                    ring_type: tree_structure::RingType::SideBranch,
                    parent_ring_id: None,
                    children_ring_ids: Vec::new(),
                });
            }
        }
        
        // Ensure we have at least one ring in each group
        if trunk_continuation_rings.is_empty() && !parent_rings.is_empty() {
            let parent_ring = &parent_rings[0];
            trunk_continuation_rings.push(tree_structure::ComponentRing {
                offset: parent_ring.offset,
                radius: parent_ring.radius,
                ring_type: tree_structure::RingType::MainTrunk,
                parent_ring_id: None,
                children_ring_ids: Vec::new(),
            });
        }
        
        if side_branch_rings.is_empty() && parent_rings.len() > 1 {
            let parent_ring = &parent_rings[parent_rings.len() - 1];
            side_branch_rings.push(tree_structure::ComponentRing {
                offset: Vec2::ZERO,
                radius: parent_ring.radius, // Keep exact parent radius
                ring_type: tree_structure::RingType::SideBranch,
                parent_ring_id: None,
                children_ring_ids: Vec::new(),
            });
        }
        
        (trunk_continuation_rings, side_branch_rings)
    }
    
    fn generate_branch_recursive_static(
        cross_sections: &mut Vec<tree_structure::BranchCrossSection>,
        parent_ring_id: tree_structure::RingId,
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
        trunk_height: f32,
    ) {
        use rand::Rng;
        use glam::{Vec3, Quat};
        
        // Stop recursion if too deep or radius too small
        if depth >= max_depth || current_radius < 0.05 {
            return;
        }
        
        // Get the specific parent ring we're extending
        let parent_ring = &cross_sections[parent_ring_id.cross_section_index].component_rings[parent_ring_id.ring_index];
        let parent_center = cross_sections[parent_ring_id.cross_section_index].center;
        
        // Add some bend to the growth direction for natural curves
        let bend_min = bend_angle_range.0.min(bend_angle_range.1);
        let bend_max = bend_angle_range.1.max(bend_angle_range.0);
        let bend_angle = rng.gen_range(bend_min..=bend_max).to_radians();
        let bend_axis = Vec3::new(rng.gen_range(-1.0..=1.0), 0.0, rng.gen_range(-1.0..=1.0)).normalize();
        let bend_rotation = Quat::from_axis_angle(bend_axis, bend_angle);
        let bent_direction = (bend_rotation * growth_direction).normalize();
        
        let next_center = parent_center + bent_direction * segment_length;
        
        // Apply gentle tapering based on radius_taper parameter
        let segment_taper = 1.0 - (1.0 - radius_taper) * 0.1;
        
        // Create child ring from THIS specific parent ring
        let child_ring = Self::create_child_ring_from_parent(parent_ring, parent_ring_id, segment_taper);
        let child_radius = child_ring.radius; // Store radius before moving
        
        // Find or create the cross-section at this position
        let (child_cross_section_index, child_ring_index) = Self::find_or_create_cross_section(
            cross_sections,
            next_center,
            bent_direction,
            parent_ring_id.cross_section_index,
            child_ring,
        );
        
        let child_ring_id = tree_structure::RingId {
            cross_section_index: child_cross_section_index,
            ring_index: child_ring_index,
        };
        
        // Calculate current height for branching decisions
        let current_height = next_center.y;
        
        // Determine if branching should occur
        let freq_min = branch_frequency_range.0.max(1);
        let freq_max = branch_frequency_range.1.max(freq_min);
        let segment_branch_ready = segments_since_branch >= rng.gen_range(freq_min..=freq_max);
        
        // Only start branching after reaching a minimum height (60% of trunk height)
        let min_branching_height = trunk_height * 0.6;
        let height_allows_branching = current_height >= min_branching_height;
        
        let should_branch = segment_branch_ready && height_allows_branching && depth < max_depth - 1;
        
        if should_branch {
            // Create branch from this ring
            let angle_min = branch_angle_range.0.min(branch_angle_range.1);
            let angle_max = branch_angle_range.1.max(branch_angle_range.0);
            let branch_angle = rng.gen_range(angle_min..=angle_max).to_radians();
            
            // Generate branch direction
            let up_component = Vec3::Y;
            let perpendicular = if bent_direction.cross(up_component).length() > 0.1 {
                bent_direction.cross(up_component).normalize()
            } else {
                Vec3::X
            };
            
            let branch_rotation = Quat::from_axis_angle(perpendicular, branch_angle);
            let branch_direction = (branch_rotation * bent_direction).normalize();
            
            // Continue main trunk with this ring
            Self::generate_branch_recursive_static(
                cross_sections,
                child_ring_id,
                bent_direction, // Main direction
                child_radius,
                depth + 1,
                0, // Reset segment counter after branching
                rng,
                segment_length,
                branch_angle_range,
                bend_angle_range,
                branch_frequency_range,
                max_depth,
                radius_taper,
                trunk_height,
            );
            
            // Create side branch from this ring
            Self::generate_branch_recursive_static(
                cross_sections,
                child_ring_id,
                branch_direction, // Branch direction
                child_radius, // Same radius as parent
                depth + 1,
                0, // Reset segment counter
                rng,
                segment_length,
                branch_angle_range,
                bend_angle_range,
                branch_frequency_range,
                max_depth,
                radius_taper,
                trunk_height,
            );
        } else {
            // Continue growing this ring lineage
            Self::generate_branch_recursive_static(
                cross_sections,
                child_ring_id,
                bent_direction,
                child_radius,
                depth,
                segments_since_branch + 1,
                rng,
                segment_length,
                branch_angle_range,
                bend_angle_range,
                branch_frequency_range,
                max_depth,
                radius_taper,
                trunk_height,
            );
        }
    }
    
    fn find_or_create_cross_section(
        cross_sections: &mut Vec<tree_structure::BranchCrossSection>,
        center: glam::Vec3,
        orientation_direction: glam::Vec3,
        parent_index: usize,
        ring: tree_structure::ComponentRing,
    ) -> (usize, usize) {
        use glam::Quat;
        
        // For now, always create a new cross-section
        // In the future, we might want to merge rings that are close in space
        let new_cross_section = tree_structure::BranchCrossSection {
            center,
            orientation: Quat::from_rotation_arc(glam::Vec3::Y, orientation_direction),
            component_rings: vec![ring],
            parent_index: Some(parent_index),
            children_indices: Vec::new(),
        };
        
        let cross_section_index = cross_sections.len();
        cross_sections[parent_index].children_indices.push(cross_section_index);
        cross_sections.push(new_cross_section);
        
        (cross_section_index, 0) // Ring is at index 0 in the new cross-section
    }

    pub fn render(&mut self) {
        // TODO: Generate mesh data for rendering
    }

    pub fn rings_count(&self) -> usize {
        self.tree.cross_sections.len()
    }
    
    pub fn ring_center(&self, index: usize) -> Option<wasm_math::Vector3d> {
        self.tree.cross_sections.get(index).map(|cross_section| {
            wasm_math::Vector3d::new(cross_section.center.x, cross_section.center.y, cross_section.center.z)
        })
    }
    
    pub fn ring_radius(&self, index: usize) -> Option<f32> {
        self.tree.cross_sections.get(index).and_then(|cross_section| {
            cross_section.component_rings.first().map(|ring| ring.radius)
        })
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
        self.segment_length = segment_length.max(0.01); // Minimum segment length
        self.regenerate_tree();
    }

    pub fn set_branch_angle_range(&mut self, min: f32, max: f32) {
        self.branch_angle_min = min.min(max);
        self.branch_angle_max = max.max(min);
        self.regenerate_tree();
    }

    pub fn set_bend_angle_range(&mut self, min: f32, max: f32) {
        self.bend_angle_min = min.min(max);
        self.bend_angle_max = max.max(min);
        self.regenerate_tree();
    }

    pub fn set_branch_frequency_range(&mut self, min: u32, max: u32) {
        // Ensure minimum values to prevent division by zero or invalid ranges
        let validated_min = min.max(1).min(max.max(1));
        let validated_max = max.max(1).max(validated_min);
        
        self.branch_frequency_min = validated_min;
        self.branch_frequency_max = validated_max;
        self.regenerate_tree();
    }

    pub fn set_max_depth(&mut self, max_depth: u32) {
        self.max_depth = max_depth.max(1); // Minimum depth of 1
        self.regenerate_tree();
    }

    pub fn set_radius_taper(&mut self, radius_taper: f32) {
        self.radius_taper = radius_taper.max(0.1).min(1.0); // Clamp between 0.1 and 1.0
        self.regenerate_tree();
    }

    pub fn set_trunk_ring_spread(&mut self, trunk_ring_spread: f32) {
        self.trunk_ring_spread = trunk_ring_spread.max(0.0).min(2.0); // Clamp between 0.0 and 2.0
        self.regenerate_tree();
    }
}

// Public API: generate a Tree
#[wasm_bindgen]
pub fn generate(seed: u32, trunk_height: f32, butressing: f32) -> Result<TreeObject, JsValue> {
    TreeObject::new(seed, trunk_height, butressing)
}

