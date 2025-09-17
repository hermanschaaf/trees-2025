use crate::core::TrunkParams;
use crate::structure::{ComponentRing, RingType};
use glam::Vec2;

pub struct RingGenerator;

impl RingGenerator {
    pub fn new() -> Self {
        RingGenerator
    }

    pub fn generate_trunk_rings(&self, params: &TrunkParams, height: f32) -> Vec<ComponentRing> {
        use std::f32::consts::PI;
        
        // Calculate number of rings based on buttressing value
        let ring_count = if params.buttressing < 0.7 {
            1
        } else {
            ((params.buttressing - 0.5) * 3.0 + 1.0).max(1.0) as u32
        };
        let mut rings = Vec::new();
        
        // Base trunk radius controlled by trunk_size parameter
        let base_trunk_radius = 0.5 * params.size;
        let target_area = PI * base_trunk_radius * base_trunk_radius;
        
        if ring_count == 1 {
            // Single central ring - maintains full area
            rings.push(ComponentRing {
                offset: Vec2::ZERO,
                radius: base_trunk_radius,
                ring_type: RingType::MainTrunk,
            });
        } else {
            // Multiple rings - adjust radii to maintain total cross-sectional area
            let area_per_ring = target_area / ring_count as f32;
            let individual_ring_radius = (area_per_ring / PI).sqrt();
            
            // Central ring
            rings.push(ComponentRing {
                offset: Vec2::ZERO,
                radius: individual_ring_radius,
                ring_type: RingType::MainTrunk,
            });
            
            // Surrounding rings
            let outer_rings = ring_count - 1;
            
            // Calculate height-dependent trunk flaring
            let height_factor = if params.height > 0.0 {
                let normalized_height = (height / params.height).clamp(0.0, 1.0);
                1.0 - (normalized_height * 0.75)
            } else {
                1.0
            };
            
            let spread_distance = base_trunk_radius * params.ring_spread * height_factor;
            
            for i in 0..outer_rings {
                let angle = (i as f32 / outer_rings as f32) * 2.0 * PI;
                let offset = Vec2::new(
                    angle.cos() * spread_distance,
                    angle.sin() * spread_distance,
                );
                
                rings.push(ComponentRing {
                    offset,
                    radius: individual_ring_radius,
                    ring_type: RingType::SideBranch,
                });
            }
        }
        
        rings
    }

    pub fn create_child_ring_from_parent(
        &self,
        parent_ring: &ComponentRing,
        taper_ratio: f32,
    ) -> ComponentRing {
        ComponentRing {
            offset: parent_ring.offset,
            radius: parent_ring.radius * taper_ratio,
            ring_type: parent_ring.ring_type.clone(),
        }
    }
}