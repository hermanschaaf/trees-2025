use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use glam::Vec2;
use std::collections::BTreeMap;

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
    pub trunk_size: f32, // 0.2-2.0: base trunk radius multiplier
    pub branch_azimuth_variation: f32, // 0.0-1.0: 3D branching spread
    pub max_branch_reach: f32, // 2.0-10.0: max distance branches can extend from trunk center
    // Root system parameters
    pub root_enable: bool, // Whether to generate roots
    pub root_depth: f32, // How deep roots go (0.5-3.0)
    pub root_spread: f32, // How wide roots spread (0.5-2.0)
    pub root_density: u32, // Number of main roots (2-8)
    pub root_segment_length: f32, // Length of root segments
    
    // Twig system parameters
    pub twig_enable: bool, // Whether to generate twigs
    pub twig_density: f32, // Twigs per branch tip (0.1-2.0)
    pub twig_scale: f32, // Size scaling for twigs (0.5-3.0)
    pub twig_angle_variation: f32, // Angular spread of twigs (0.0-1.0)
    tree: tree_structure::TreeStructure,
}

#[wasm_bindgen]
pub struct TreeMesh {
    vertices: Vec<f32>,
    normals: Vec<f32>,
    uvs: Vec<f32>,
    indices: Vec<u32>,
    depths: Vec<u32>, // Depth value for each vertex
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
    
    #[wasm_bindgen(getter)]
    pub fn depths(&self) -> Vec<u32> {
        self.depths.clone()
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
            max_depth: 20, // Increase to allow more branching
            radius_taper: 0.8,
            trunk_ring_spread: 0.5,   // Default moderate spread
            segment_length_variation: 0.3, // Default moderate variation
            trunk_size: 3.0, // Default trunk size (0.5 base radius)
            branch_azimuth_variation: 0.5, // Default moderate 3D spread
            max_branch_reach: 50.0, // Default generous branch reach
            // Root system defaults
            root_enable: true,
            root_depth: 1.5,
            root_spread: 1.2,
            root_density: 4,
            root_segment_length: 0.3,
            
            // Twig system defaults
            twig_enable: true,
            twig_density: 1.0,
            twig_scale: 1.0,
            twig_angle_variation: 0.5,
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
            depth: 0, // Root/trunk starts at depth 0
            component_rings: trunk_rings,
            parent_index: None,
            children_indices: Vec::new(),
        };
        self.tree.cross_sections.push(root_cross_section);
        
        // Start coordinated generation from root cross-section (trunk)
        Self::generate_coordinated_recursive_static(
            &mut self.tree.cross_sections,
            0,                        // Start from root cross-section
            Vec3::new(0.0, 1.0, 0.0), // growth_direction (up)
            0,                        // depth
            0,                        // segments_since_branch
            0,                        // segments_at_current_depth (start at 0)
            &mut rng,
            trunk_segment_length,     // Use calculated trunk segment length
            branch_angle_range,
            bend_angle_range,
            branch_frequency_range,
            max_depth,
            radius_taper,
            self.split_height,        // Pass split height for branching logic
            self.segment_length_variation, // Pass segment variation
            self.branch_azimuth_variation, // Pass 3D branching parameter
            self.max_branch_reach,    // Pass max branch reach parameter
        );
        
        // Generate root system if enabled
        if self.root_enable {
            Self::generate_root_system(
                &mut self.tree.cross_sections,
                0, // Start from same root cross-section as trunk
                &mut rng,
                self.root_depth,
                self.root_spread,
                self.root_density,
                self.root_segment_length,
                radius_taper,
                self.segment_length_variation,
            );
        }
        
        // Generate twigs if enabled
        if self.twig_enable {
            // Clear existing twigs
            self.tree.twigs.clear();
            
            // Create twig generation parameters
            let twig_params = tree_structure::TwigGenerationParams {
                density: self.twig_density,
                scale_min: self.twig_scale * 0.5,
                scale_max: self.twig_scale * 1.5,
                angle_variation: self.twig_angle_variation,
                attachment_threshold: 0.05, // Attach twigs to branches with radius < 0.05
            };
            
            // Find twig attachment points in all cross-sections (excluding roots)
            for (i, cross_section) in self.tree.cross_sections.iter().enumerate() {
                for ring in &cross_section.component_rings {
                    // Skip root rings - don't generate twigs on roots
                    if matches!(ring.ring_type, tree_structure::RingType::Root { .. }) {
                        continue;
                    }
                    
                    // Check if this ring should have twigs - make density affect all branches
                    let density_chance = (twig_params.density * 0.3).min(1.0); // Scale 0.1-2.0 to 0.03-0.6, capped at 1.0
                    let should_have_twigs = ring.radius <= twig_params.attachment_threshold 
                        && rng.gen_range(0.0..1.0) < density_chance;
                    
                    // Also add random twigs to medium-sized branches for more natural distribution
                    let is_medium_branch_with_random_twigs = ring.radius > twig_params.attachment_threshold 
                        && ring.radius <= twig_params.attachment_threshold * 2.0 
                        && rng.gen_range(0.0..1.0) < (twig_params.density * 0.1); // Reduced impact
                    
                    if should_have_twigs || is_medium_branch_with_random_twigs {
                        // Get branch direction (approximate from parent if available)
                        let branch_direction = if i > 0 {
                            (cross_section.center - self.tree.cross_sections[0].center).normalize()
                        } else {
                            Vec3::Y // Default upward direction for trunk
                        };
                        
                        // Generate twigs at this position
                        Self::generate_twigs_at_position(
                            &mut self.tree.twigs,
                            cross_section.center,
                            branch_direction,
                            ring.radius,
                            &twig_params,
                            &mut rng,
                        );
                    }
                }
            }
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
        
        // Base trunk radius controlled by trunk_size parameter
        let base_trunk_radius = 0.5 * self.trunk_size;
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
        segments_at_current_depth: u32,
        rng: &mut rand::rngs::SmallRng,
        segment_length: f32,
        branch_angle_range: (f32, f32),
        bend_angle_range: (f32, f32),
        branch_frequency_range: (u32, u32),
        max_depth: u32,
        radius_taper: f32,
        trunk_height: f32,
        segment_length_variation: f32,
        branch_azimuth_variation: f32,
        max_branch_reach: f32,
    ) {
        use rand::Rng;
        use glam::{Vec3, Quat};
        
        // Stop recursion if too deep
        if depth >= max_depth {
            // This is a termination point - generate twigs if enabled
            return;
        }
        
        // Stop if branch has extended too many segments at current depth (prevents long tendrils)
        // For trunk (depth 0-2), allow enough segments to reach target height
        let max_segments_at_depth = match depth {
            0..=2 => {
                // Calculate minimum segments needed to reach split height
                let min_segments_for_height = (trunk_height / segment_length).ceil() as u32;
                (min_segments_for_height + 10).max(20)  // Trunk: ensure we can reach target height + some extra
            },
            3..=5 => 8,   // Secondary branches medium length
            6..=8 => 4,   // Tertiary branches short
            9..=12 => 3,  // Deep branches short
            13..=16 => 2, // Very deep branches very short
            _ => 1,       // Extremely deep branches minimal
        };
        if segments_at_current_depth >= max_segments_at_depth {
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
        
        // Stop if branch has extended extremely far from trunk center (fallback safety check)
        let distance_from_trunk = current_center.length(); // Distance from origin (trunk base)
        if distance_from_trunk > max_branch_reach * 1.5 { // Much more generous limit, mostly for safety
            return;
        }
        
        // Add some bend to the growth direction for natural curves
        // Reduce bending for trunk segments to keep them straighter
        let bend_reduction_factor = match depth {
            0..=1 => 0.1,   // Trunk: only 10% of bend (very straight)
            2..=3 => 0.3,   // Main branches: 30% of bend
            4..=5 => 0.6,   // Secondary branches: 60% of bend
            _ => 1.0,       // Small branches: full bend (natural curves)
        };
        
        let bend_min = bend_angle_range.0.min(bend_angle_range.1) * bend_reduction_factor;
        let bend_max = bend_angle_range.1.max(bend_angle_range.0) * bend_reduction_factor;
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
                branch_azimuth_variation,
                max_branch_reach,
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
                depth,
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
                segments_at_current_depth + 1, // Increment segment count at current depth
                rng,
                segment_length,
                branch_angle_range,
                bend_angle_range,
                branch_frequency_range,
                max_depth,
                radius_taper,
                trunk_height,
                segment_length_variation,
                branch_azimuth_variation,
                max_branch_reach,
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
        branch_azimuth_variation: f32,
        max_branch_reach: f32,
    ) {
        use rand::Rng;
        use glam::{Vec3, Quat};
        
        // Create branch direction with 3D spherical branching
        let angle_min = branch_angle_range.0.min(branch_angle_range.1);
        let angle_max = branch_angle_range.1.max(branch_angle_range.0);
        let branch_angle = rng.gen_range(angle_min..=angle_max).to_radians();
        
        // Traditional 2D branching (base method)
        let up_component = Vec3::Y;
        let perpendicular = if main_direction.cross(up_component).length() > 0.1 {
            main_direction.cross(up_component).normalize()
        } else {
            Vec3::X
        };
        let planar_rotation = Quat::from_axis_angle(perpendicular, branch_angle);
        let planar_direction = (planar_rotation * main_direction).normalize();
        
        // Apply 3D spherical branching based on azimuth variation parameter
        let branch_direction = if branch_azimuth_variation > 0.0 {
            // Random azimuthal rotation around the main growth direction for 3D spread
            let azimuth_angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI) * branch_azimuth_variation;
            let azimuth_rotation = Quat::from_axis_angle(main_direction, azimuth_angle);
            (azimuth_rotation * planar_direction).normalize()
        } else {
            planar_direction
        };
        
        // Create trunk continuation cross-section - SPLIT rings between trunk and branch
        // Use gentler tapering similar to normal trunk growth
        let segment_taper_factor = 0.15; // Gentler tapering for branches
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
                    segment_taper * 0.95 // Very light additional taper for trunk when branching
                );
                let branch_child = Self::create_child_ring_from_parent(
                    parent_ring,
                    tree_structure::RingId { cross_section_index: parent_cross_section_index, ring_index: 0 },
                    segment_taper * 0.8 // Moderately smaller branch (was 0.6)
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
            depth: depth + 1,
            component_rings: trunk_rings,
            parent_index: Some(parent_cross_section_index),
            children_indices: Vec::new(),
        };
        
        // Create branch cross-section
        let branch_cross_section = tree_structure::BranchCrossSection {
            center: center + branch_direction * segment_length,
            orientation: Quat::from_rotation_arc(Vec3::Y, branch_direction),
            depth: depth + 1,
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
            0, // Reset segments at depth counter for new branch
            rng,
            segment_length,
            branch_angle_range,
            bend_angle_range,
            branch_frequency_range,
            max_depth,
            radius_taper,
            trunk_height,
            segment_length_variation,
            branch_azimuth_variation,
            max_branch_reach,
        );
        
        Self::generate_coordinated_recursive_static(
            cross_sections,
            branch_cs_idx,
            branch_direction,
            depth + 1,
            0, // Reset segment counter
            0, // Reset segments at depth counter for new branch
            rng,
            segment_length,
            branch_angle_range,
            bend_angle_range,
            branch_frequency_range,
            max_depth,
            radius_taper,
            trunk_height,
            segment_length_variation,
            branch_azimuth_variation,
            max_branch_reach,
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
        if depth >= max_depth || current_radius < 0.01 {
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
            depth,
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
        
        // Only start branching after reaching the split height
        // Split height controls how much trunk we want before branching starts
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
        depth: u32,
    ) -> (usize, usize) {
        use glam::Quat;
        
        // For now, always create a new cross-section
        // In the future, we might want to merge rings that are close in space
        let new_cross_section = tree_structure::BranchCrossSection {
            center,
            orientation: Quat::from_rotation_arc(glam::Vec3::Y, orientation_direction),
            depth,
            component_rings: vec![ring],
            parent_index: Some(parent_index),
            children_indices: Vec::new(),
        };
        
        let cross_section_index = cross_sections.len();
        cross_sections[parent_index].children_indices.push(cross_section_index);
        cross_sections.push(new_cross_section);
        
        (cross_section_index, 0) // Ring is at index 0 in the new cross-section
    }
    
    fn generate_twigs_at_position(
        twigs: &mut Vec<tree_structure::Twig>,
        position: glam::Vec3,
        branch_direction: glam::Vec3,
        branch_radius: f32,
        twig_params: &tree_structure::TwigGenerationParams,
        rng: &mut rand::rngs::SmallRng,
    ) {
        use rand::Rng;
        use glam::{Vec3, Quat};
        use std::f32::consts::PI;
        
        // Determine number of twigs based on density and branch size
        let base_twig_count = twig_params.density * 2.0; // More moderate scaling with density
        let twig_count = rng.gen_range((base_twig_count * 0.5)..=(base_twig_count * 1.5)) as u32;
        let twig_count = twig_count.max(1).min(12); // Allow up to 12 twigs per position
        
        for i in 0..twig_count {
            // Random angle around the branch
            let angle = if twig_count == 1 {
                rng.gen_range(0.0..2.0 * PI)
            } else {
                (i as f32 / twig_count as f32) * 2.0 * PI + rng.gen_range(-0.5..0.5)
            };
            
            // Create perpendicular vectors to branch direction
            let up = if branch_direction.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
            let right = branch_direction.cross(up).normalize();
            let forward = right.cross(branch_direction).normalize();
            
            // Random twig direction with variation
            let variation = twig_params.angle_variation * rng.gen_range(-1.0..1.0);
            let twig_angle = 45.0f32.to_radians() + variation * 30.0f32.to_radians();
            
            let twig_direction = (
                branch_direction * twig_angle.cos() +
                (right * angle.cos() + forward * angle.sin()) * twig_angle.sin()
            ).normalize();
            
            // Random twig scale
            let scale = rng.gen_range(twig_params.scale_min..=twig_params.scale_max);
            
            // Choose twig type based on branch radius
            let twig_type = if branch_radius < 0.02 {
                tree_structure::TwigType::LeafCluster
            } else if branch_radius < 0.04 {
                tree_structure::TwigType::BranchTip
            } else {
                tree_structure::TwigType::SmallBranch
            };
            
            // Create twig
            let twig = tree_structure::Twig {
                position: position + twig_direction * (branch_radius * 0.5), // Slight offset from branch center
                orientation: Quat::from_rotation_arc(Vec3::Y, twig_direction),
                scale,
                twig_type,
            };
            
            twigs.push(twig);
        }
    }
    
    fn generate_root_system(
        cross_sections: &mut Vec<tree_structure::BranchCrossSection>,
        root_cross_section_index: usize,
        rng: &mut rand::rngs::SmallRng,
        root_depth: f32,
        root_spread: f32,
        root_density: u32,
        root_segment_length: f32,
        radius_taper: f32,
        segment_length_variation: f32,
    ) {
        use rand::Rng;
        use glam::{Vec3, Quat, Vec2};
        use std::f32::consts::PI;
        
        // Clone the data we need before mutating the vector
        let (root_center, parent_rings) = {
            let root_cross_section = &cross_sections[root_cross_section_index];
            (root_cross_section.center, root_cross_section.component_rings.clone())
        };
        
        // Distribute roots among available parent rings
        let rings_per_root = (parent_rings.len() as f32 / root_density as f32).max(1.0);
        
        // Create main roots radiating outward and downward
        for i in 0..root_density {
            let angle = (i as f32 / root_density as f32) * 2.0 * PI;
            let horizontal_direction = Vec3::new(angle.cos(), 0.0, angle.sin());
            
            // Root direction: mostly down but angled outward, influenced by root_spread
            let base_downward_angle = 45.0f32.to_radians(); // Base 45 degrees from vertical
            let spread_influence = (root_spread - 1.0) * 15.0; // spread affects angle by ±15 degrees
            let downward_angle = (base_downward_angle + spread_influence.to_radians()).clamp(20.0f32.to_radians(), 70.0f32.to_radians());
            
            let root_direction = Vec3::new(
                horizontal_direction.x * downward_angle.sin(),
                -downward_angle.cos(), // Negative Y = downward
                horizontal_direction.z * downward_angle.sin(),
            ).normalize();
            
            // Select parent ring for this root (distribute evenly)
            let parent_ring_index = ((i as f32 * rings_per_root) as usize).min(parent_rings.len() - 1);
            let parent_ring = &parent_rings[parent_ring_index];
            
            // Position root connection point on the edge of parent ring, in root direction
            let connection_offset = Vec2::new(
                horizontal_direction.x * parent_ring.radius * root_spread,
                horizontal_direction.z * parent_ring.radius * root_spread,
            );
            
            // Calculate initial root radius with tapering from parent
            let parent_radius = parent_ring.radius;
            let root_radius = parent_radius * 0.4; // Roots start smaller than their parent
            
            // Create root starting ring
            let root_ring = tree_structure::ComponentRing {
                offset: connection_offset,
                radius: root_radius,
                ring_type: tree_structure::RingType::Root { 
                    root_type: tree_structure::RootType::LateralRoot 
                },
                parent_ring_id: None,
                children_ring_ids: Vec::new(),
            };
            
            // Create first root cross-section
            let root_start_center = root_center + root_direction * root_segment_length;
            let first_root_cross_section = tree_structure::BranchCrossSection {
                center: root_start_center,
                orientation: Quat::from_rotation_arc(Vec3::Y, root_direction),
                depth: 1, // Roots start at depth 1
                component_rings: vec![root_ring],
                parent_index: Some(root_cross_section_index),
                children_indices: Vec::new(),
            };
            
            let root_cs_index = cross_sections.len();
            cross_sections[root_cross_section_index].children_indices.push(root_cs_index);
            cross_sections.push(first_root_cross_section);
            
            // Generate root growth recursively
            Self::generate_root_recursive_static(
                cross_sections,
                root_cs_index,
                root_direction,
                1, // Start at depth 1
                rng,
                root_segment_length,
                root_depth,
                root_spread,
                radius_taper,
                segment_length_variation,
                root_radius, // Pass initial radius for tapering
            );
        }
    }
    
    fn generate_root_recursive_static(
        cross_sections: &mut Vec<tree_structure::BranchCrossSection>,
        current_cross_section_index: usize,
        growth_direction: glam::Vec3,
        depth: u32,
        rng: &mut rand::rngs::SmallRng,
        segment_length: f32,
        max_root_depth: f32,
        root_spread: f32,
        radius_taper: f32,
        segment_length_variation: f32,
        current_radius: f32,
    ) {
        use rand::Rng;
        use glam::{Vec3, Quat, Vec2};
        
        // Stop if too deep, too small, or reached max depth
        if depth > 25 || current_radius < 0.01 {
            return;
        }
        
        let current_cross_section = &cross_sections[current_cross_section_index];
        let current_center = current_cross_section.center;
        
        // Stop if we've gone too deep below ground
        if current_center.y < -max_root_depth {
            return;
        }
        
        // Add some bend and variation to root growth
        let bend_angle = rng.gen_range(-20.0f32..=20.0f32).to_radians();
        let bend_axis = Vec3::new(rng.gen_range(-1.0..=1.0), 0.0, rng.gen_range(-1.0..=1.0)).normalize();
        let bend_rotation = Quat::from_axis_angle(bend_axis, bend_angle);
        let bent_direction = (bend_rotation * growth_direction).normalize();
        
        // Apply segment length variation
        let variation_factor = 1.0 + (rng.gen_range(-1.0..=1.0) * segment_length_variation);
        let varied_segment_length = segment_length * variation_factor.max(0.1);
        
        let next_center = current_center + bent_direction * varied_segment_length;
        
        // Apply aggressive tapering for roots
        let next_radius = current_radius * radius_taper * 0.9; // Extra aggressive tapering for roots
        
        // Create next root segment
        let next_ring = tree_structure::ComponentRing {
            offset: Vec2::ZERO, // Keep simple for root segments
            radius: next_radius,
            ring_type: tree_structure::RingType::Root { 
                root_type: tree_structure::RootType::LateralRoot 
            },
            parent_ring_id: None,
            children_ring_ids: Vec::new(),
        };
        
        let next_cross_section = tree_structure::BranchCrossSection {
            center: next_center,
            orientation: glam::Quat::from_rotation_arc(Vec3::Y, bent_direction),
            depth: depth + 1,
            component_rings: vec![next_ring],
            parent_index: Some(current_cross_section_index),
            children_indices: Vec::new(),
        };
        
        let next_cs_index = cross_sections.len();
        cross_sections[current_cross_section_index].children_indices.push(next_cs_index);
        cross_sections.push(next_cross_section);
        
        // Continue root growth
        Self::generate_root_recursive_static(
            cross_sections,
            next_cs_index,
            bent_direction,
            depth + 1,
            rng,
            segment_length,
            max_root_depth,
            root_spread,
            radius_taper,
            segment_length_variation,
            next_radius,
        );
        
        // Occasionally create side root branches
        let should_branch = depth > 2 && rng.gen_range(0.0..=1.0) < 0.3; // 30% chance to branch
        
        if should_branch && depth < 15 {
            // Create a side root branch
            let branch_angle = rng.gen_range(30.0f32..90.0f32).to_radians();
            let branch_axis = Vec3::new(rng.gen_range(-1.0..=1.0), 0.2, rng.gen_range(-1.0..=1.0)).normalize();
            let branch_rotation = Quat::from_axis_angle(branch_axis, branch_angle);
            let branch_direction = (branch_rotation * bent_direction).normalize();
            
            // Branch starts smaller than main root
            let branch_radius = current_radius * 0.6;
            
            Self::generate_root_recursive_static(
                cross_sections,
                next_cs_index,
                branch_direction,
                depth + 1,
                rng,
                segment_length * 0.8, // Shorter segments for branches
                max_root_depth,
                root_spread,
                radius_taper,
                segment_length_variation,
                branch_radius,
            );
        }
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
    
    pub fn twigs_count(&self) -> usize {
        self.tree.twigs.len()
    }
    
    pub fn twig_position(&self, index: usize) -> Option<wasm_math::Vector3d> {
        self.tree.twigs.get(index).map(|twig| {
            wasm_math::Vector3d::new(twig.position.x, twig.position.y, twig.position.z)
        })
    }
    
    pub fn twig_orientation_x(&self, index: usize) -> Option<f32> {
        self.tree.twigs.get(index).map(|twig| twig.orientation.x)
    }
    
    pub fn twig_orientation_y(&self, index: usize) -> Option<f32> {
        self.tree.twigs.get(index).map(|twig| twig.orientation.y)
    }
    
    pub fn twig_orientation_z(&self, index: usize) -> Option<f32> {
        self.tree.twigs.get(index).map(|twig| twig.orientation.z)
    }
    
    pub fn twig_orientation_w(&self, index: usize) -> Option<f32> {
        self.tree.twigs.get(index).map(|twig| twig.orientation.w)
    }
    
    pub fn twig_scale(&self, index: usize) -> Option<f32> {
        self.tree.twigs.get(index).map(|twig| twig.scale)
    }
    
    pub fn twig_type(&self, index: usize) -> Option<String> {
        self.tree.twigs.get(index).map(|twig| {
            match twig.twig_type {
                tree_structure::TwigType::LeafCluster => "LeafCluster".to_string(),
                tree_structure::TwigType::SmallBranch => "SmallBranch".to_string(),
                tree_structure::TwigType::BranchTip => "BranchTip".to_string(),
            }
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
            depths: ring_mesh.depths,
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

    pub fn set_trunk_size(&mut self, trunk_size: f32) {
        self.trunk_size = trunk_size.max(0.1).min(10.0); // Clamp between 0.1 and 10.0
        self.regenerate_tree();
    }

    pub fn set_branch_azimuth_variation(&mut self, variation: f32) {
        self.branch_azimuth_variation = variation.max(0.0).min(1.0); // Clamp between 0.0 and 1.0
        self.regenerate_tree();
    }

    pub fn set_max_branch_reach(&mut self, reach: f32) {
        self.max_branch_reach = reach.max(2.0).min(50.0); // Clamp between 2.0 and 50.0
        self.regenerate_tree();
    }

    // Root system setters
    pub fn set_root_enable(&mut self, enable: bool) {
        self.root_enable = enable;
        self.regenerate_tree();
    }

    pub fn set_root_depth(&mut self, depth: f32) {
        self.root_depth = depth.max(0.5).min(3.0); // Clamp between 0.5 and 3.0
        self.regenerate_tree();
    }

    pub fn set_root_spread(&mut self, spread: f32) {
        self.root_spread = spread.max(0.5).min(2.0); // Clamp between 0.5 and 2.0
        self.regenerate_tree();
    }

    pub fn set_root_density(&mut self, density: u32) {
        self.root_density = density.max(2).min(8); // Clamp between 2 and 8
        self.regenerate_tree();
    }

    pub fn set_root_segment_length(&mut self, segment_length: f32) {
        self.root_segment_length = segment_length.max(0.1).min(0.8); // Clamp between 0.1 and 0.8
        self.regenerate_tree();
    }

    // Twig system setters
    pub fn set_twig_enable(&mut self, enable: bool) {
        self.twig_enable = enable;
        self.regenerate_tree();
    }

    pub fn set_twig_density(&mut self, density: f32) {
        self.twig_density = density.max(0.1).min(2.0); // Clamp between 0.1 and 2.0
        self.regenerate_tree();
    }

    pub fn set_twig_scale(&mut self, scale: f32) {
        self.twig_scale = scale.max(0.1).min(3.0); // Clamp between 0.5 and 3.0
        self.regenerate_tree();
    }

    pub fn set_twig_angle_variation(&mut self, variation: f32) {
        self.twig_angle_variation = variation.max(0.0).min(1.0); // Clamp between 0.0 and 1.0
        self.regenerate_tree();
    }

    /// Export the tree as a GLTF file (returns JSON as string)
    pub fn export_gltf(&self, resolution: u32) -> Result<String, JsValue> {
        let mesh = self.generate_tree_mesh(resolution);
        
        // Create GLTF JSON structure
        let mut root = gltf_json::Root::default();
        
        // Create scene
        let scene_index = root.push(gltf_json::Scene {
            extensions: Default::default(),
            extras: Default::default(),
            nodes: vec![gltf_json::Index::new(0)],
        });
        root.scene = Some(gltf_json::Index::new(0)); // Scene index is always 0

        // Create node
        root.push(gltf_json::Node {
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            mesh: Some(gltf_json::Index::new(0)),
            rotation: None,
            scale: None,
            translation: None,
            skin: None,
            weights: None,
        });

        // Create mesh
        root.push(gltf_json::Mesh {
            extensions: Default::default(),
            extras: Default::default(),
            primitives: vec![gltf_json::mesh::Primitive {
                attributes: {
                    let mut map = BTreeMap::new();
                    map.insert(
                        gltf_json::validation::Checked::Valid(gltf_json::mesh::Semantic::Positions),
                        gltf_json::Index::new(0), // Position accessor
                    );
                    map.insert(
                        gltf_json::validation::Checked::Valid(gltf_json::mesh::Semantic::Normals),
                        gltf_json::Index::new(1), // Normal accessor
                    );
                    map.insert(
                        gltf_json::validation::Checked::Valid(gltf_json::mesh::Semantic::TexCoords(0)),
                        gltf_json::Index::new(2), // UV accessor
                    );
                    map
                },
                extensions: Default::default(),
                extras: Default::default(),
                indices: Some(gltf_json::Index::new(3)), // Index accessor
                material: None,
                mode: gltf_json::validation::Checked::Valid(gltf_json::mesh::Mode::Triangles),
                targets: None,
            }],
            weights: None,
        });

        // Create buffer data
        let vertices_bytes = mesh.vertices.iter()
            .flat_map(|f| f.to_le_bytes().to_vec())
            .collect::<Vec<u8>>();
        
        let normals_bytes = mesh.normals.iter()
            .flat_map(|f| f.to_le_bytes().to_vec())
            .collect::<Vec<u8>>();
            
        let uvs_bytes = mesh.uvs.iter()
            .flat_map(|f| f.to_le_bytes().to_vec())
            .collect::<Vec<u8>>();
            
        let indices_bytes = mesh.indices.iter()
            .flat_map(|i| i.to_le_bytes().to_vec())
            .collect::<Vec<u8>>();

        // Combine all data into single buffer
        let mut buffer_data = Vec::new();
        let vertices_offset = 0;
        let normals_offset = vertices_bytes.len();
        let uvs_offset = normals_offset + normals_bytes.len();
        let indices_offset = uvs_offset + uvs_bytes.len();
        
        buffer_data.extend(vertices_bytes);
        buffer_data.extend(normals_bytes);
        buffer_data.extend(uvs_bytes);
        buffer_data.extend(indices_bytes);

        // Create buffer
        use base64::Engine as _;
        let buffer_data_base64 = base64::engine::general_purpose::STANDARD.encode(&buffer_data);
        let buffer_uri = format!("data:application/octet-stream;base64,{}", buffer_data_base64);
        
        root.push(gltf_json::Buffer {
            byte_length: gltf_json::validation::USize64::from(buffer_data.len()),
            extensions: Default::default(),
            extras: Default::default(),
            uri: Some(buffer_uri),
        });

        // Create buffer view for vertices
        root.push(gltf_json::buffer::View {
            buffer: gltf_json::Index::new(0),
            byte_length: gltf_json::validation::USize64::from(mesh.vertices.len() * 4),
            byte_offset: Some(gltf_json::validation::USize64::from(vertices_offset as usize)),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: Some(gltf_json::validation::Checked::Valid(gltf_json::buffer::Target::ArrayBuffer)),
        });

        // Create buffer view for normals
        root.push(gltf_json::buffer::View {
            buffer: gltf_json::Index::new(0),
            byte_length: gltf_json::validation::USize64::from(mesh.normals.len() * 4),
            byte_offset: Some(gltf_json::validation::USize64::from(normals_offset as usize)),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: Some(gltf_json::validation::Checked::Valid(gltf_json::buffer::Target::ArrayBuffer)),
        });

        // Create buffer view for UVs
        root.push(gltf_json::buffer::View {
            buffer: gltf_json::Index::new(0),
            byte_length: gltf_json::validation::USize64::from(mesh.uvs.len() * 4),
            byte_offset: Some(gltf_json::validation::USize64::from(uvs_offset as usize)),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: Some(gltf_json::validation::Checked::Valid(gltf_json::buffer::Target::ArrayBuffer)),
        });

        // Create buffer view for indices
        root.push(gltf_json::buffer::View {
            buffer: gltf_json::Index::new(0),
            byte_length: gltf_json::validation::USize64::from(mesh.indices.len() * 4),
            byte_offset: Some(gltf_json::validation::USize64::from(indices_offset as usize)),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: Some(gltf_json::validation::Checked::Valid(gltf_json::buffer::Target::ElementArrayBuffer)),
        });

        // Create accessors
        // Position accessor
        root.push(gltf_json::Accessor {
            buffer_view: Some(gltf_json::Index::new(0)),
            byte_offset: Some(gltf_json::validation::USize64::from(0_usize)),
            component_type: gltf_json::validation::Checked::Valid(gltf_json::accessor::GenericComponentType(gltf_json::accessor::ComponentType::F32)),
            count: gltf_json::validation::USize64::from(mesh.vertices.len() / 3),
            extensions: Default::default(),
            extras: Default::default(),
            type_: gltf_json::validation::Checked::Valid(gltf_json::accessor::Type::Vec3),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });

        // Normal accessor
        root.push(gltf_json::Accessor {
            buffer_view: Some(gltf_json::Index::new(1)),
            byte_offset: Some(gltf_json::validation::USize64::from(0_usize)),
            component_type: gltf_json::validation::Checked::Valid(gltf_json::accessor::GenericComponentType(gltf_json::accessor::ComponentType::F32)),
            count: gltf_json::validation::USize64::from(mesh.normals.len() / 3),
            extensions: Default::default(),
            extras: Default::default(),
            type_: gltf_json::validation::Checked::Valid(gltf_json::accessor::Type::Vec3),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });

        // UV accessor
        root.push(gltf_json::Accessor {
            buffer_view: Some(gltf_json::Index::new(2)),
            byte_offset: Some(gltf_json::validation::USize64::from(0_usize)),
            component_type: gltf_json::validation::Checked::Valid(gltf_json::accessor::GenericComponentType(gltf_json::accessor::ComponentType::F32)),
            count: gltf_json::validation::USize64::from(mesh.uvs.len() / 2),
            extensions: Default::default(),
            extras: Default::default(),
            type_: gltf_json::validation::Checked::Valid(gltf_json::accessor::Type::Vec2),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });

        // Index accessor
        root.push(gltf_json::Accessor {
            buffer_view: Some(gltf_json::Index::new(3)),
            byte_offset: Some(gltf_json::validation::USize64::from(0_usize)),
            component_type: gltf_json::validation::Checked::Valid(gltf_json::accessor::GenericComponentType(gltf_json::accessor::ComponentType::U32)),
            count: gltf_json::validation::USize64::from(mesh.indices.len()),
            extensions: Default::default(),
            extras: Default::default(),
            type_: gltf_json::validation::Checked::Valid(gltf_json::accessor::Type::Scalar),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });

        // Serialize to JSON
        match serde_json::to_string_pretty(&root) {
            Ok(json) => Ok(json),
            Err(e) => Err(JsValue::from_str(&format!("Failed to serialize GLTF: {}", e))),
        }
    }
}

// Public API: generate a Tree
#[wasm_bindgen]
pub fn generate(seed: u32, trunk_height: f32, butressing: f32) -> Result<TreeObject, JsValue> {
    TreeObject::new(seed, trunk_height, butressing)
}

