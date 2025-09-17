pub mod placement;

use crate::core::{TwigGenerator, TwigParams, TreeSubsystem};
use crate::structure::{TreeStructure, TwigGenerationParams, RingType};
use glam::Vec3;
use rand::{Rng, rngs::SmallRng};

pub use placement::TwigPlacer;

pub struct TwigSystem {
    placer: TwigPlacer,
}

impl TwigSystem {
    pub fn new() -> Self {
        TwigSystem {
            placer: TwigPlacer::new(),
        }
    }
}

impl TreeSubsystem for TwigSystem {
    type Params = TwigParams;
    type Output = ();
    
    fn generate(
        params: &Self::Params,
        tree: &mut TreeStructure,
        rng: &mut SmallRng,
    ) -> Self::Output {
        if !params.enable {
            return;
        }

        let placer = TwigPlacer::new();
        
        // Clear existing twigs
        tree.twigs.clear();
        
        // Create twig generation parameters
        let twig_params = TwigGenerationParams {
            density: params.density,
            scale_min: params.scale * 0.5,
            scale_max: params.scale * 1.5,
            angle_variation: params.angle_variation,
            attachment_threshold: 0.05,
        };
        
        // Find twig attachment points in all cross-sections (excluding roots)
        for (i, cross_section) in tree.cross_sections.iter().enumerate() {
            for ring in &cross_section.component_rings {
                // Skip root rings
                if matches!(ring.ring_type, RingType::Root { .. }) {
                    continue;
                }
                
                // Check if this ring should have twigs
                let density_chance = (twig_params.density * 0.3).min(1.0);
                let should_have_twigs = ring.radius <= twig_params.attachment_threshold 
                    && rng.gen_range(0.0..1.0) < density_chance;
                
                // Also add random twigs to medium-sized branches
                let is_medium_branch_with_random_twigs = ring.radius > twig_params.attachment_threshold 
                    && ring.radius <= twig_params.attachment_threshold * 2.0 
                    && rng.gen_range(0.0..1.0) < (twig_params.density * 0.1);
                
                if should_have_twigs || is_medium_branch_with_random_twigs {
                    // Get branch direction
                    let branch_direction = if i > 0 {
                        (cross_section.center - tree.cross_sections[0].center).normalize()
                    } else {
                        Vec3::Y
                    };
                    
                    // Generate twigs at this position
                    placer.generate_twigs_at_position(
                        &mut tree.twigs,
                        cross_section.center,
                        branch_direction,
                        ring.radius,
                        &twig_params,
                        rng,
                    );
                }
            }
        }
    }
}

impl TwigGenerator for TwigSystem {}

impl Default for TwigSystem {
    fn default() -> Self {
        Self::new()
    }
}