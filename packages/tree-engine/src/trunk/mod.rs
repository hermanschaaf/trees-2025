pub mod rings;

use crate::core::{TrunkGenerator, TrunkParams, TreeSubsystem};
use crate::structure::{TreeStructure, BranchCrossSection};
use glam::{Vec3, Quat};
use rand::rngs::SmallRng;

pub use rings::RingGenerator;

pub struct TrunkSystem {
    ring_generator: RingGenerator,
}

impl TrunkSystem {
    pub fn new() -> Self {
        TrunkSystem {
            ring_generator: RingGenerator::new(),
        }
    }
}

impl TreeSubsystem for TrunkSystem {
    type Params = TrunkParams;
    type Output = ();
    
    fn generate(
        params: &Self::Params,
        tree: &mut TreeStructure,
        _rng: &mut SmallRng,
    ) -> Self::Output {
        let trunk_system = TrunkSystem::new();
        
        // Clear existing cross-sections
        tree.cross_sections.clear();
        
        // Create root cross-section with multiple rings based on buttressing
        let trunk_rings = trunk_system.ring_generator.generate_trunk_rings(params, 0.0);
        
        let root_cross_section = BranchCrossSection {
            center: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            depth: 0,
            component_rings: trunk_rings,
            children_indices: Vec::new(),
        };
        
        tree.cross_sections.push(root_cross_section);
    }
}

impl TrunkGenerator for TrunkSystem {}

impl Default for TrunkSystem {
    fn default() -> Self {
        Self::new()
    }
}