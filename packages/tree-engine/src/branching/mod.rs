pub mod patterns;

use crate::core::{BranchGenerator, BranchingParams, TreeSubsystem, GeneralParams, TrunkParams};
use crate::structure::{TreeStructure, BranchCrossSection, ComponentRing};
use crate::trunk::RingGenerator;
use glam::{Vec3, Quat};
use rand::{Rng, rngs::SmallRng};

pub use patterns::BranchingPatterns;

pub struct BranchingSystem {
    patterns: BranchingPatterns,
}

impl BranchingSystem {
    pub fn new() -> Self {
        BranchingSystem {
            patterns: BranchingPatterns::new(),
        }
    }

    pub fn generate_branches(
        &self,
        branching_params: &BranchingParams,
        trunk_params: &TrunkParams,
        general_params: &GeneralParams,
        tree: &mut TreeStructure,
        rng: &mut SmallRng,
    ) {
        // Start coordinated generation from root cross-section (trunk)
        if !tree.cross_sections.is_empty() {
            Self::generate_coordinated_recursive(
                &mut tree.cross_sections,
                0,                        // Start from root cross-section
                Vec3::new(0.0, 1.0, 0.0), // growth_direction (up)
                0,                        // depth
                0,                        // segments_since_branch
                0,                        // segments_at_current_depth
                rng,
                branching_params,
                trunk_params,
                general_params,
            );
        }
    }

    fn generate_coordinated_recursive(
        cross_sections: &mut Vec<BranchCrossSection>,
        current_cross_section_index: usize,
        growth_direction: Vec3,
        depth: u32,
        segments_since_branch: u32,
        segments_at_current_depth: u32,
        rng: &mut SmallRng,
        branching_params: &BranchingParams,
        trunk_params: &TrunkParams,
        general_params: &GeneralParams,
    ) {
        // Stop recursion if too deep
        if depth >= general_params.max_depth {
            return;
        }
        
        // Stop if branch has extended too many segments at current depth
        let max_segments_at_depth = match depth {
            0..=2 => {
                let min_segments_for_height = (trunk_params.height / trunk_params.segment_length).ceil() as u32;
                (min_segments_for_height + 10).max(20)
            },
            3..=5 => 8,
            6..=8 => 4,
            9..=12 => 3,
            13..=16 => 2,
            _ => 1,
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
        
        // Stop if trunk is too small
        let total_area: f32 = current_rings.iter().map(|r| std::f32::consts::PI * r.radius * r.radius).sum();
        let effective_trunk_radius = (total_area / std::f32::consts::PI).sqrt();
        if effective_trunk_radius < 0.005 {
            return;
        }
        
        // Stop if branch has extended extremely far from trunk center
        let distance_from_trunk = current_center.length();
        if distance_from_trunk > branching_params.max_reach * 1.5 {
            return;
        }
        
        // Add some bend to the growth direction for natural curves
        let bend_reduction_factor = match depth {
            0..=1 => 0.1,
            2..=3 => 0.3,
            4..=5 => 0.6,
            _ => 1.0,
        };
        
        let bend_min = branching_params.bend_angle_min.min(branching_params.bend_angle_max) * bend_reduction_factor;
        let bend_max = branching_params.bend_angle_max.max(branching_params.bend_angle_min) * bend_reduction_factor;
        let bend_angle = rng.gen_range(bend_min..=bend_max).to_radians();
        let bend_axis = Vec3::new(rng.gen_range(-1.0..=1.0), 0.0, rng.gen_range(-1.0..=1.0)).normalize();
        let bend_rotation = Quat::from_axis_angle(bend_axis, bend_angle);
        let bent_direction = (bend_rotation * growth_direction).normalize();
        
        // Apply segment length variation
        let variation_factor = 1.0 + (rng.gen_range(-1.0..=1.0) * trunk_params.segment_length_variation);
        let varied_segment_length = trunk_params.segment_length * variation_factor.max(0.1);
        
        let next_center = current_center + bent_direction * varied_segment_length;
        let next_height = next_center.y;
        
        // Make coordinated branching decision
        let freq_min = branching_params.frequency_min.max(1);
        let freq_max = branching_params.frequency_max.max(freq_min);
        let segment_branch_ready = segments_since_branch >= rng.gen_range(freq_min..=freq_max);
        let height_allows_branching = next_height >= trunk_params.split_height;
        let should_branch = segment_branch_ready && height_allows_branching && depth < general_params.max_depth - 1;
        
        if should_branch {
            Self::create_coordinated_branches(
                cross_sections,
                current_cross_section_index,
                &current_rings,
                next_center,
                bent_direction,
                depth,
                rng,
                branching_params,
                trunk_params,
                general_params,
            );
        } else {
            // Continue as trunk - create next cross-section with all rings
            let ring_count = current_rings.len() as f32;
            let buttressing_factor = if ring_count > 2.0 { 0.5 } else { 1.0 };
            let base_taper_factor = if next_height < trunk_params.height { 0.05 } else { 0.4 };
            let segment_taper_factor = base_taper_factor * buttressing_factor;
            let segment_taper = 1.0 - (1.0 - branching_params.radius_taper) * segment_taper_factor;
            
            // Create child rings for ALL parent rings
            let ring_generator = RingGenerator::new();
            let mut child_rings = Vec::new();
            for parent_ring in &current_rings {
                let mut child_ring = ring_generator.create_child_ring_from_parent(
                    parent_ring, 
                    segment_taper
                );
                
                // Apply trunk flaring: reduce ring spread as we go higher
                if matches!(child_ring.ring_type, crate::structure::RingType::MainTrunk | crate::structure::RingType::SideBranch) 
                   && depth == 0 {
                    let height_factor = if trunk_params.height > 0.0 {
                        let normalized_height = (next_center.y / trunk_params.height).clamp(0.0, 1.0);
                        1.0 - (normalized_height * 0.75)
                    } else {
                        1.0
                    };
                    
                    child_ring.offset = child_ring.offset * height_factor;
                }
                
                child_rings.push(child_ring);
            }
            
            // Create next cross-section with all child rings
            let new_cross_section = BranchCrossSection {
                center: next_center,
                orientation: Quat::from_rotation_arc(Vec3::Y, bent_direction),
                depth,
                component_rings: child_rings,
                children_indices: Vec::new(),
            };
            
            let new_cross_section_index = cross_sections.len();
            cross_sections[current_cross_section_index].children_indices.push(new_cross_section_index);
            cross_sections.push(new_cross_section);
            
            // Continue recursively with the new cross-section
            Self::generate_coordinated_recursive(
                cross_sections,
                new_cross_section_index,
                bent_direction,
                depth,
                segments_since_branch + 1,
                segments_at_current_depth + 1,
                rng,
                branching_params,
                trunk_params,
                general_params,
            );
        }
    }
    
    fn create_coordinated_branches(
        cross_sections: &mut Vec<BranchCrossSection>,
        parent_cross_section_index: usize,
        parent_rings: &[ComponentRing],
        center: Vec3,
        main_direction: Vec3,
        depth: u32,
        rng: &mut SmallRng,
        branching_params: &BranchingParams,
        trunk_params: &TrunkParams,
        general_params: &GeneralParams,
    ) {
        // Create branch direction with 3D spherical branching
        let angle_min = branching_params.angle_min.min(branching_params.angle_max);
        let angle_max = branching_params.angle_max.max(branching_params.angle_min);
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
        let branch_direction = if branching_params.azimuth_variation > 0.0 {
            let azimuth_angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI) * branching_params.azimuth_variation;
            let azimuth_rotation = Quat::from_axis_angle(main_direction, azimuth_angle);
            (azimuth_rotation * planar_direction).normalize()
        } else {
            planar_direction
        };
        
        // Create trunk continuation and branch cross-sections
        let segment_taper_factor = 0.15;
        let segment_taper = 1.0 - (1.0 - branching_params.radius_taper) * segment_taper_factor;
        
        let ring_generator = RingGenerator::new();
        let mut trunk_rings = Vec::new();
        let mut branch_rings = Vec::new();
        
        // Split rings: some continue as trunk, one becomes branch
        for (i, parent_ring) in parent_rings.iter().enumerate() {
            if parent_rings.len() == 1 {
                // Single ring: trunk gets the ring, branch gets a smaller copy
                let trunk_child = ring_generator.create_child_ring_from_parent(
                    parent_ring,
                    segment_taper * 0.95
                );
                let branch_child = ring_generator.create_child_ring_from_parent(
                    parent_ring,
                    segment_taper * 0.8
                );
                trunk_rings.push(trunk_child);
                branch_rings.push(branch_child);
            } else {
                // Multiple rings: split them between trunk and branch
                if i == parent_rings.len() - 1 {
                    // Last ring becomes the branch
                    let branch_child = ring_generator.create_child_ring_from_parent(
                        parent_ring,
                        segment_taper
                    );
                    branch_rings.push(branch_child);
                } else {
                    // Other rings continue as trunk
                    let trunk_child = ring_generator.create_child_ring_from_parent(
                        parent_ring,
                        segment_taper * 0.95
                    );
                    trunk_rings.push(trunk_child);
                }
            }
        }
        
        // Create trunk continuation cross-section
        let trunk_cross_section = BranchCrossSection {
            center: center + main_direction * trunk_params.segment_length,
            orientation: Quat::from_rotation_arc(Vec3::Y, main_direction),
            depth: depth + 1,
            component_rings: trunk_rings,
            children_indices: Vec::new(),
        };
        
        // Create branch cross-section
        let branch_cross_section = BranchCrossSection {
            center: center + branch_direction * trunk_params.segment_length,
            orientation: Quat::from_rotation_arc(Vec3::Y, branch_direction),
            depth: depth + 1,
            component_rings: branch_rings,
            children_indices: Vec::new(),
        };
        
        let trunk_cs_idx = cross_sections.len();
        let branch_cs_idx = trunk_cs_idx + 1;
        
        cross_sections[parent_cross_section_index].children_indices.push(trunk_cs_idx);
        cross_sections[parent_cross_section_index].children_indices.push(branch_cs_idx);
        
        cross_sections.push(trunk_cross_section);
        cross_sections.push(branch_cross_section);
        
        // Continue both trunk and branch recursively
        Self::generate_coordinated_recursive(
            cross_sections,
            trunk_cs_idx,
            main_direction,
            depth + 1,
            0,
            0,
            rng,
            branching_params,
            trunk_params,
            general_params,
        );
        
        Self::generate_coordinated_recursive(
            cross_sections,
            branch_cs_idx,
            branch_direction,
            depth + 1,
            0,
            0,
            rng,
            branching_params,
            trunk_params,
            general_params,
        );
    }
}

impl TreeSubsystem for BranchingSystem {
    type Params = BranchingParams;
    type Output = ();
    
    fn generate(
        _params: &Self::Params,
        _tree: &mut TreeStructure,
        _rng: &mut SmallRng,
    ) -> Self::Output {
        // Note: Branching is complex and will be handled by the orchestrator
        // This trait implementation is mainly for interface consistency
    }
}

impl BranchGenerator for BranchingSystem {}

impl Default for BranchingSystem {
    fn default() -> Self {
        Self::new()
    }
}