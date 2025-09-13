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
    pub segment_length_variation: f32, // 0.0-1.0: how much segment lengths vary
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
            max_depth: 12, // Increase to allow longer trunk growth
            radius_taper: 0.8,
            trunk_ring_spread: 0.5,   // Default moderate spread
            segment_length_variation: 0.3, // Default moderate variation
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
        // Use the actual segment_length parameter
        let trunk_segment_length = self.segment_length;
        
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
        
        // Start coordinated generation from root cross-section
        Self::generate_coordinated_recursive_static(
            &mut self.tree.cross_sections,
            0,                        // Start from root cross-section
            Vec3::new(0.0, 1.0, 0.0), // growth_direction (up)
            0,                        // depth
            0,                        // segments_since_branch
            &mut rng,
            trunk_segment_length,     // Use calculated trunk segment length
            branch_angle_range,
            bend_angle_range,
            branch_frequency_range,
            max_depth,
            radius_taper,
            self.trunk_height,        // Pass trunk height for branching logic
            self.segment_length_variation, // Pass segment variation
        );
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
    
    fn generate_coordinated_recursive_static(
        cross_sections: &mut Vec<tree_structure::BranchCrossSection>,
        current_cross_section_index: usize,
        growth_direction: glam::Vec3,
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
        segment_length_variation: f32,
    ) {
        use rand::Rng;
        use glam::{Vec3, Quat};
        
        // Stop recursion if too deep
        if depth >= max_depth {
            return;
        }
        
        let current_cross_section = &cross_sections[current_cross_section_index];
        let current_center = current_cross_section.center;
        let current_rings = current_cross_section.component_rings.clone();
        
        // Stop if no rings left
        if current_rings.is_empty() {
            return;
        }
        
        // Stop if trunk is too small - use effective trunk radius (total area)
        let total_area: f32 = current_rings.iter().map(|r| std::f32::consts::PI * r.radius * r.radius).sum();
        let effective_trunk_radius = (total_area / std::f32::consts::PI).sqrt();
        if effective_trunk_radius < 0.005 { // Much lower threshold for trunk termination
            return;
        }
        
        // Add some bend to the growth direction for natural curves
        let bend_min = bend_angle_range.0.min(bend_angle_range.1);
        let bend_max = bend_angle_range.1.max(bend_angle_range.0);
        let bend_angle = rng.gen_range(bend_min..=bend_max).to_radians();
        let bend_axis = Vec3::new(rng.gen_range(-1.0..=1.0), 0.0, rng.gen_range(-1.0..=1.0)).normalize();
        let bend_rotation = Quat::from_axis_angle(bend_axis, bend_angle);
        let bent_direction = (bend_rotation * growth_direction).normalize();
        
        // Apply segment length variation
        let base_segment_length = segment_length;
        let variation_factor = 1.0 + (rng.gen_range(-1.0..=1.0) * segment_length_variation);
        let varied_segment_length = base_segment_length * variation_factor.max(0.1); // Ensure positive
        
        let next_center = current_center + bent_direction * varied_segment_length;
        let next_height = next_center.y;
        
        // Make coordinated branching decision for ALL rings in this cross-section
        let freq_min = branch_frequency_range.0.max(1);
        let freq_max = branch_frequency_range.1.max(freq_min);
        let segment_branch_ready = segments_since_branch >= rng.gen_range(freq_min..=freq_max);
        let height_allows_branching = next_height >= trunk_height;
        let should_branch = segment_branch_ready && height_allows_branching && depth < max_depth - 1;
        
        if should_branch {
            // ALL rings in this cross-section will branch together
            Self::create_coordinated_branches(
                cross_sections,
                current_cross_section_index,
                &current_rings,
                next_center,
                bent_direction,
                depth,
                rng,
                segment_length,
                branch_angle_range,
                bend_angle_range,
                branch_frequency_range,
                max_depth,
                radius_taper,
                trunk_height,
                segment_length_variation,
            );
        } else {
            // ALL rings continue as trunk - create next cross-section with all rings
            // Reduce tapering for buttressed trunks (many rings) to preserve thickness longer
            let ring_count = current_rings.len() as f32;
            let buttressing_factor = if ring_count > 2.0 { 0.5 } else { 1.0 }; // Less taper for buttressed trunks
            let base_taper_factor = if next_height < trunk_height { 0.05 } else { 0.4 };
            let segment_taper_factor = base_taper_factor * buttressing_factor;
            let segment_taper = 1.0 - (1.0 - radius_taper) * segment_taper_factor;
            
            // Create child rings for ALL parent rings
            let mut child_rings = Vec::new();
            for parent_ring in &current_rings {
                let child_ring = Self::create_child_ring_from_parent(
                    parent_ring, 
                    tree_structure::RingId { cross_section_index: current_cross_section_index, ring_index: 0 },
                    segment_taper
                );
                child_rings.push(child_ring);
            }
            
            // Create next cross-section with all child rings
            let new_cross_section = tree_structure::BranchCrossSection {
                center: next_center,
                orientation: Quat::from_rotation_arc(Vec3::Y, bent_direction),
                component_rings: child_rings,
                parent_index: Some(current_cross_section_index),
                children_indices: Vec::new(),
            };
            
            let new_cross_section_index = cross_sections.len();
            cross_sections[current_cross_section_index].children_indices.push(new_cross_section_index);
            cross_sections.push(new_cross_section);
            
            // Continue recursively with the new cross-section
            Self::generate_coordinated_recursive_static(
                cross_sections,
                new_cross_section_index,
                bent_direction,
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
                segment_length_variation,
            );
        }
    }
    
    fn create_coordinated_branches(
        cross_sections: &mut Vec<tree_structure::BranchCrossSection>,
        parent_cross_section_index: usize,
        parent_rings: &[tree_structure::ComponentRing],
        center: glam::Vec3,
        main_direction: glam::Vec3,
        depth: u32,
        rng: &mut rand::rngs::SmallRng,
        segment_length: f32,
        branch_angle_range: (f32, f32),
        bend_angle_range: (f32, f32),
        branch_frequency_range: (u32, u32),
        max_depth: u32,
        radius_taper: f32,
        trunk_height: f32,
        segment_length_variation: f32,
    ) {
        use rand::Rng;
        use glam::{Vec3, Quat};
        
        // Create branch direction
        let angle_min = branch_angle_range.0.min(branch_angle_range.1);
        let angle_max = branch_angle_range.1.max(branch_angle_range.0);
        let branch_angle = rng.gen_range(angle_min..=angle_max).to_radians();
        
        let up_component = Vec3::Y;
        let perpendicular = if main_direction.cross(up_component).length() > 0.1 {
            main_direction.cross(up_component).normalize()
        } else {
            Vec3::X
        };
        
        let branch_rotation = Quat::from_axis_angle(perpendicular, branch_angle);
        let branch_direction = (branch_rotation * main_direction).normalize();
        
        // Create trunk continuation cross-section - SPLIT rings between trunk and branch
        let segment_taper_factor = 0.5; // More aggressive tapering for branches
        let segment_taper = 1.0 - (1.0 - radius_taper) * segment_taper_factor;
        
        let mut trunk_rings = Vec::new();
        let mut branch_rings = Vec::new();
        
        // Split rings: some continue as trunk, one becomes branch
        for (i, parent_ring) in parent_rings.iter().enumerate() {
            if parent_rings.len() == 1 {
                // Single ring: trunk gets the ring, branch gets a smaller copy
                let trunk_child = Self::create_child_ring_from_parent(
                    parent_ring,
                    tree_structure::RingId { cross_section_index: parent_cross_section_index, ring_index: 0 },
                    segment_taper * 0.9 // Slightly taper trunk when branching
                );
                let branch_child = Self::create_child_ring_from_parent(
                    parent_ring,
                    tree_structure::RingId { cross_section_index: parent_cross_section_index, ring_index: 0 },
                    segment_taper * 0.6 // Much smaller branch
                );
                trunk_rings.push(trunk_child);
                branch_rings.push(branch_child);
            } else {
                // Multiple rings: split them between trunk and branch
                if i == parent_rings.len() - 1 {
                    // Last ring becomes the branch
                    let branch_child = Self::create_child_ring_from_parent(
                        parent_ring,
                        tree_structure::RingId { cross_section_index: parent_cross_section_index, ring_index: i },
                        segment_taper
                    );
                    branch_rings.push(branch_child);
                } else {
                    // Other rings continue as trunk
                    let trunk_child = Self::create_child_ring_from_parent(
                        parent_ring,
                        tree_structure::RingId { cross_section_index: parent_cross_section_index, ring_index: i },
                        segment_taper * 0.95 // Very light taper for trunk continuation
                    );
                    trunk_rings.push(trunk_child);
                }
            }
        }
        
        // Create trunk continuation cross-section
        let trunk_cross_section = tree_structure::BranchCrossSection {
            center: center + main_direction * segment_length,
            orientation: Quat::from_rotation_arc(Vec3::Y, main_direction),
            component_rings: trunk_rings,
            parent_index: Some(parent_cross_section_index),
            children_indices: Vec::new(),
        };
        
        // Create branch cross-section
        let branch_cross_section = tree_structure::BranchCrossSection {
            center: center + branch_direction * segment_length,
            orientation: Quat::from_rotation_arc(Vec3::Y, branch_direction),
            component_rings: branch_rings,
            parent_index: Some(parent_cross_section_index),
            children_indices: Vec::new(),
        };
        
        let trunk_cs_idx = cross_sections.len();
        let branch_cs_idx = trunk_cs_idx + 1;
        
        cross_sections[parent_cross_section_index].children_indices.push(trunk_cs_idx);
        cross_sections[parent_cross_section_index].children_indices.push(branch_cs_idx);
        
        cross_sections.push(trunk_cross_section);
        cross_sections.push(branch_cross_section);
        
        // Continue both trunk and branch recursively
        Self::generate_coordinated_recursive_static(
            cross_sections,
            trunk_cs_idx,
            main_direction,
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
            segment_length_variation,
        );
        
        Self::generate_coordinated_recursive_static(
            cross_sections,
            branch_cs_idx,
            branch_direction,
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
            segment_length_variation,
        );
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
        let next_height = next_center.y;
        
        // Apply gentle tapering based on radius_taper parameter
        // Less taper for trunk segments, more for branches
        let trunk_taper_factor = if next_height < trunk_height { 0.02 } else { 0.1 }; // Much less taper in trunk
        let segment_taper = 1.0 - (1.0 - radius_taper) * trunk_taper_factor;
        
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
        
        // Use the already calculated height for branching decisions
        let current_height = next_height;
        
        // Determine if branching should occur
        let freq_min = branch_frequency_range.0.max(1);
        let freq_max = branch_frequency_range.1.max(freq_min);
        let segment_branch_ready = segments_since_branch >= rng.gen_range(freq_min..=freq_max);
        
        // Only start branching after reaching the desired trunk height
        // Height parameter controls how much trunk we want before branching starts
        let min_branching_height = trunk_height;
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

    pub fn set_segment_length_variation(&mut self, segment_length_variation: f32) {
        self.segment_length_variation = segment_length_variation.max(0.0).min(1.0); // Clamp between 0.0 and 1.0
        self.regenerate_tree();
    }
}

// Public API: generate a Tree
#[wasm_bindgen]
pub fn generate(seed: u32, trunk_height: f32, butressing: f32) -> Result<TreeObject, JsValue> {
    TreeObject::new(seed, trunk_height, butressing)
}

