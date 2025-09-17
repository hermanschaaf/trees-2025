use crate::structure::{Twig, TwigType, TwigGenerationParams};
use glam::{Vec3, Quat};
use rand::{Rng, rngs::SmallRng};

pub struct TwigPlacer;

impl TwigPlacer {
    pub fn new() -> Self {
        TwigPlacer
    }

    pub fn generate_twigs_at_position(
        &self,
        twigs: &mut Vec<Twig>,
        position: Vec3,
        branch_direction: Vec3,
        branch_radius: f32,
        twig_params: &TwigGenerationParams,
        rng: &mut SmallRng,
    ) {
        use std::f32::consts::PI;
        
        // Determine number of twigs based on density and branch size
        let base_twig_count = twig_params.density * 2.0;
        let twig_count = rng.gen_range((base_twig_count * 0.5)..=(base_twig_count * 1.5)) as u32;
        let twig_count = twig_count.max(1).min(12);
        
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
                TwigType::LeafCluster
            } else if branch_radius < 0.04 {
                TwigType::BranchTip
            } else {
                TwigType::SmallBranch
            };
            
            // Create twig
            let twig = Twig {
                position: position + twig_direction * (branch_radius * 0.5),
                orientation: Quat::from_rotation_arc(Vec3::Y, twig_direction),
                scale,
                twig_type,
            };
            
            twigs.push(twig);
        }
    }
}