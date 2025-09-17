use crate::core::{RootGenerator, RootParams, TreeSubsystem};
use crate::structure::{TreeStructure, BranchCrossSection, ComponentRing, RingType};
use glam::{Vec3, Quat};
use rand::rngs::SmallRng;

pub struct RootSystem;

impl RootSystem {
    pub fn new() -> Self {
        RootSystem
    }
}

impl TreeSubsystem for RootSystem {
    type Params = RootParams;
    type Output = ();
    
    fn generate(
        params: &Self::Params,
        tree: &mut TreeStructure,
        _rng: &mut SmallRng,
    ) -> Self::Output {
        if !params.enable {
            return;
        }

        let base_center = tree.cross_sections[0].center;
        let base_rings = tree.cross_sections[0].component_rings.clone();
        let root_segments = 3; // TODO: Make this configurable based on root_depth
        
        for segment in 0..root_segments {
            let segment_height = (segment + 1) as f32 * params.segment_length;
            let root_center = base_center - Vec3::new(0.0, segment_height, 0.0);
            
            // Create root rings that taper slightly as we go deeper
            let taper_factor = 1.0 - (segment as f32 * 0.1);
            let root_rings: Vec<ComponentRing> = base_rings.iter().map(|trunk_ring| {
                ComponentRing {
                    offset: trunk_ring.offset,
                    radius: trunk_ring.radius * taper_factor,
                    ring_type: RingType::MainTrunk, // TODO: Use Root type when implemented
                }
            }).collect();
            
            // Create root cross-section that extends trunk downward
            let root_cross_section = BranchCrossSection {
                center: root_center,
                orientation: Quat::IDENTITY,
                depth: 0,
                component_rings: root_rings,
                children_indices: Vec::new(),
            };
            
            let root_index = tree.cross_sections.len();
            if segment == 0 {
                tree.cross_sections[0].children_indices.push(root_index);
            } else {
                tree.cross_sections[root_index - 1].children_indices.push(root_index);
            }
            tree.cross_sections.push(root_cross_section);
        }
    }
}

impl RootGenerator for RootSystem {}

impl Default for RootSystem {
    fn default() -> Self {
        Self::new()
    }
}