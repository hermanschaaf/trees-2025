use glam::{Vec2, Vec3, Quat};

#[derive(Debug)]
pub struct TreeStructure {
    pub cross_sections: Vec<BranchCrossSection>,
    pub twigs: Vec<Twig>,       // Collection of all twigs in the tree
}

#[derive(Debug, Clone)]
pub struct BranchCrossSection {
    // Geometric properties
    pub center: Vec3,           // Position in 3D space
    pub orientation: Quat,      // Cross-section orientation
    pub depth: u32,             // Hierarchical depth (0=trunk, 1=main branches, etc.)

    // Component rings at this position
    pub component_rings: Vec<ComponentRing>,

    // Cross-section level connectivity
    pub children_indices: Vec<usize>, // Indices of child cross-sections
}

#[derive(Debug, Clone)]
pub struct ComponentRing {
    // Local geometric properties
    pub offset: Vec2,           // Offset from cross-section center (local coordinates)
    pub radius: f32,            // Ring radius
    pub ring_type: RingType,    // What type of branch/root this ring represents
}

#[derive(Debug, Clone, PartialEq)]
pub enum RingType {
    MainTrunk,
    SideBranch,
    Root { root_type: RootType },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RootType {
    TapRoot,        // Deep central root
    LateralRoot,    // Horizontal spreading roots
    FeederRoot,     // Small surface roots
}

#[derive(Debug, Clone)]
pub struct RingMesh {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u32>,
    pub depths: Vec<u32>, // Depth value for each vertex
}

#[derive(Debug, Clone)]
pub struct CrossSectionGeometry {
    pub points: Vec<Vec3>,      // Points around the unified perimeter
}

#[derive(Debug, Clone)]
pub struct Twig {
    pub position: Vec3,         // World position of twig attachment
    pub orientation: Quat,      // Twig orientation in world space
    pub scale: f32,             // Size scaling factor (0.5-3.0)
    pub twig_type: TwigType,    // Type of twig for rendering
}

#[derive(Debug, Clone, PartialEq)]
pub enum TwigType {
    LeafCluster,                // Dense cluster of leaves
    SmallBranch,                // Small woody branch with sparse leaves
    BranchTip,                  // Terminal branch tip with buds
}

#[derive(Debug, Clone)]
pub struct TwigGenerationParams {
    pub density: f32,           // Twigs per branch tip (0.1-2.0)
    pub scale_min: f32,         // Minimum twig scale
    pub scale_max: f32,         // Maximum twig scale
    pub angle_variation: f32,   // Angular spread variation (0.0-1.0)
    pub attachment_threshold: f32, // Branch radius threshold for twig attachment
}

impl TreeStructure {
    pub fn new() -> TreeStructure {
        TreeStructure{
            cross_sections: Vec::new(),
            twigs: Vec::new(),
        }
    }

    pub fn generate_mesh(&self, ring_resolution: u32) -> RingMesh {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();
        let mut depths = Vec::new();

        // Generate geometry for each cross-section (unified perimeter from multiple rings)
        let cross_section_geometries: Vec<CrossSectionGeometry> = self.cross_sections
            .iter()
            .map(|cross_section| cross_section.generate_unified_geometry(ring_resolution))
            .collect();

        // Connect parent cross-sections to children with tubular surfaces
        for (cs_idx, cross_section) in self.cross_sections.iter().enumerate() {
            for &child_idx in &cross_section.children_indices {
                self.connect_cross_section_perimeters_with_depth(
                    &cross_section_geometries[cs_idx],
                    &cross_section_geometries[child_idx],
                    cross_section.depth,
                    self.cross_sections[child_idx].depth,
                    &mut vertices,
                    &mut normals,
                    &mut uvs,
                    &mut indices,
                    &mut depths,
                );
            }
        }

        RingMesh { vertices, normals, uvs, indices, depths }
    }

    fn connect_cross_section_perimeters_with_depth(
        &self,
        parent_geo: &CrossSectionGeometry,
        child_geo: &CrossSectionGeometry,
        parent_depth: u32,
        child_depth: u32,
        vertices: &mut Vec<Vec3>,
        normals: &mut Vec<Vec3>,
        uvs: &mut Vec<Vec2>,
        indices: &mut Vec<u32>,
        depths: &mut Vec<u32>,
    ) {
        let base_vertex_idx = vertices.len() as u32;
        let resolution = parent_geo.points.len().min(child_geo.points.len());

        // Add vertices from both ring perimeters with depth information
        for i in 0..resolution {
            vertices.push(parent_geo.points[i]);
            vertices.push(child_geo.points[i]);
            depths.push(parent_depth);
            depths.push(child_depth);
        }

        // Calculate surface normals for tubular connection
        for i in 0..resolution {
            let next_i = (i + 1) % resolution;
            
            // Get quad vertices
            let p1 = parent_geo.points[i];
            let p2 = child_geo.points[i];
            let _p3 = child_geo.points[next_i];
            let p4 = parent_geo.points[next_i];
            
            // Calculate face normal for this section of tube
            let edge1 = p2 - p1;
            let edge2 = p4 - p1;
            let face_normal = edge1.cross(edge2).normalize();
            
            // Use outward-pointing normals
            normals.push(face_normal);
            normals.push(face_normal);
        }

        // Generate UVs (u = around circumference, v = along branch)
        for i in 0..resolution {
            let u = i as f32 / resolution as f32;
            uvs.push(Vec2::new(u, 0.0)); // Parent ring
            uvs.push(Vec2::new(u, 1.0)); // Child ring
        }

        // Create quad faces between ring perimeters (tubular surface)
        for i in 0..resolution {
            let next_i = (i + 1) % resolution;

            // Indices for the quad connecting perimeter points
            let p1 = base_vertex_idx + (i * 2) as u32;       // parent current
            let p2 = base_vertex_idx + (i * 2 + 1) as u32;   // child current  
            let p3 = base_vertex_idx + (next_i * 2 + 1) as u32; // child next
            let p4 = base_vertex_idx + (next_i * 2) as u32;  // parent next

            // Two triangles per quad forming the tube surface
            indices.extend_from_slice(&[p1, p2, p3, p1, p3, p4]);
        }
    }
}

impl BranchCrossSection {
    pub fn generate_unified_geometry(&self, resolution: u32) -> CrossSectionGeometry {
        let points = Vec::new();

        // For now, simple approach: if multiple rings, sample around the union of circles
        // TODO: Later implement proper CSG operations for complex shapes
        
        if self.component_rings.is_empty() {
            // No rings - return empty geometry
            return CrossSectionGeometry {
                points,
            };
        }

        if self.component_rings.len() == 1 {
            // Single ring - use traditional circular geometry
            let ring = &self.component_rings[0];
            return self.generate_single_ring_geometry(ring, resolution);
        }

        // Multiple rings - generate unified perimeter
        self.generate_multi_ring_geometry(resolution)
    }

    fn generate_single_ring_geometry(&self, ring: &ComponentRing, resolution: u32) -> CrossSectionGeometry {
        let mut points = Vec::with_capacity(resolution as usize);
        let mut normals = Vec::with_capacity(resolution as usize);
        let mut tangents = Vec::with_capacity(resolution as usize);

        // Create points around the ring
        for i in 0..resolution {
            let angle = (i as f32 / resolution as f32) * 2.0 * std::f32::consts::PI;

            // Local ring coordinates in XZ plane (horizontal ring)
            let local_x = angle.cos() * ring.radius + ring.offset.x;
            let local_z = angle.sin() * ring.radius + ring.offset.y; // offset.y maps to local Z
            let local_point = Vec3::new(local_x, 0.0, local_z);

            // Transform to world space using cross-section orientation
            let world_point = self.center + self.orientation * local_point;
            let world_normal = self.orientation * local_point.normalize();
            let world_tangent = self.orientation * Vec3::new(-local_z, 0.0, local_x).normalize();

            points.push(world_point);
            normals.push(world_normal);
            tangents.push(world_tangent);
        }

        CrossSectionGeometry {
            points,
        }
    }

    fn generate_multi_ring_geometry(&self, resolution: u32) -> CrossSectionGeometry {
        // Create a unified perimeter from multiple overlapping circles
        let mut points = Vec::new();
        let mut normals = Vec::new();
        let mut tangents = Vec::new();
        
        // Sample points at regular angular intervals
        for i in 0..resolution {
            let angle = (i as f32 / resolution as f32) * 2.0 * std::f32::consts::PI;
            let direction = Vec2::new(angle.cos(), angle.sin());
            
            // Find the maximum distance in this direction across all rings
            let mut max_distance = 0.0f32;
            
            for ring in &self.component_rings {
                // Project the ring center onto the direction vector
                let center_projection = ring.offset.dot(direction);
                
                // Calculate the maximum reach of this ring in this direction
                let ring_reach = center_projection + ring.radius;
                
                // Also check if the direction intersects the ring from the other side
                let distance_to_center = (ring.offset - direction * center_projection).length();
                if distance_to_center <= ring.radius {
                    // The ray intersects this ring
                    let intersection_distance = center_projection + (ring.radius * ring.radius - distance_to_center * distance_to_center).sqrt();
                    max_distance = max_distance.max(intersection_distance);
                } else {
                    // The ray doesn't intersect, use the closest point on the ring
                    max_distance = max_distance.max(ring_reach);
                }
            }
            
            // Ensure we have at least some minimum radius
            max_distance = max_distance.max(0.1);
            
            // Create the point at this distance in the direction
            let local_x = direction.x * max_distance;
            let local_z = direction.y * max_distance;
            let local_point = Vec3::new(local_x, 0.0, local_z);
            
            // Transform to world space
            let world_point = self.center + self.orientation * local_point;
            let world_normal = self.orientation * local_point.normalize();
            let world_tangent = self.orientation * Vec3::new(-local_z, 0.0, local_x).normalize();
            
            points.push(world_point);
            normals.push(world_normal);
            tangents.push(world_tangent);
        }
        
        CrossSectionGeometry {
            points,
        }
    }
}

