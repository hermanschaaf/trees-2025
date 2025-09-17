use crate::core::{TreeParameters, GenerationContext, TreeSubsystem};
use crate::structure::TreeStructure;
use crate::trunk::TrunkSystem;
use crate::roots::RootSystem;
use crate::twigs::TwigSystem;
use crate::branching::BranchingSystem;

pub struct ModularTreeGenerator {
    trunk_system: TrunkSystem,
    branching_system: BranchingSystem,
    root_system: RootSystem,
    twig_system: TwigSystem,
}

impl ModularTreeGenerator {
    pub fn new() -> Self {
        ModularTreeGenerator {
            trunk_system: TrunkSystem::new(),
            branching_system: BranchingSystem::new(),
            root_system: RootSystem::new(),
            twig_system: TwigSystem::new(),
        }
    }

    pub fn generate_tree(&self, params: &TreeParameters) -> TreeStructure {
        let mut tree = TreeStructure::new();
        let mut context = GenerationContext::new(&params.general);
        
        // Step 1: Generate trunk base
        TrunkSystem::generate(&params.trunk, &mut tree, &mut context.rng);
        
        // Step 2: Generate branching structure (complex recursive process)
        self.branching_system.generate_branches(
            &params.branching,
            &params.trunk,
            &params.general,
            &mut tree, 
            &mut context.rng
        );
        
        // Step 3: Generate root system
        RootSystem::generate(&params.roots, &mut tree, &mut context.rng);
        
        // Step 4: Generate twigs
        TwigSystem::generate(&params.twigs, &mut tree, &mut context.rng);
        
        tree
    }
}

impl Default for ModularTreeGenerator {
    fn default() -> Self {
        Self::new()
    }
}