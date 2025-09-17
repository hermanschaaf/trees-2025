pub mod parameters;
pub mod generator;

use crate::structure::TreeStructure;
use rand::rngs::SmallRng;

pub use parameters::*;

/// Core trait for tree generation subsystems
pub trait TreeSubsystem {
    type Params;
    type Output;
    
    fn generate(
        params: &Self::Params,
        tree: &mut TreeStructure,
        rng: &mut SmallRng,
    ) -> Self::Output;
}

/// Trait for trunk generation systems
pub trait TrunkGenerator: TreeSubsystem<Params = TrunkParams> {}

/// Trait for branching generation systems  
pub trait BranchGenerator: TreeSubsystem<Params = BranchingParams> {}

/// Trait for root generation systems
pub trait RootGenerator: TreeSubsystem<Params = RootParams> {}

/// Trait for twig generation systems
pub trait TwigGenerator: TreeSubsystem<Params = TwigParams> {}

/// Main tree generation context
pub struct GenerationContext {
    pub general: GeneralParams,
    pub rng: SmallRng,
}

impl GenerationContext {
    pub fn new(params: &GeneralParams) -> Self {
        use rand::SeedableRng;
        
        GenerationContext {
            general: params.clone(),
            rng: SmallRng::seed_from_u64(params.seed as u64),
        }
    }
}