use glam::{Vec2, Vec3, Quat};

#[derive(Debug)]
pub struct TreeStructure {
    pub cross_sections: Vec<BranchCrossSection>,
    pub twigs: Vec<Twig>,       // Collection of all twigs in the tree

    // Metadata
    pub species_params: TreeSpecies,
    pub age: f32,
    pub overall_health: f32,
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
    pub parent_index: Option<usize>,  // Index of parent cross-section
    pub children_indices: Vec<usize>, // Indices of child cross-sections
}

#[derive(Debug, Clone)]
pub struct ComponentRing {
    // Local geometric properties
    pub offset: Vec2,           // Offset from cross-section center (local coordinates)
    pub radius: f32,            // Ring radius
    pub ring_type: RingType,    // What type of branch/root this ring represents

    // Ring level connectivity
    pub parent_ring_id: Option<RingId>,     // Parent ring (may be in different cross-section)
    pub children_ring_ids: Vec<RingId>,     // Child rings (may be in different cross-sections)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RingId {
    pub cross_section_index: usize,
    pub ring_index: usize,
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
pub struct TreeSpecies {
    pub branching_angle_range: (f32, f32),
    pub ring_spacing: f32,
    pub taper_rate: f32,        // How quickly branches thin out
    pub max_branch_depth: u32,
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
    pub normals: Vec<Vec3>,     // Normal at each point
    pub tangents: Vec<Vec3>,    // For bark texture orientation
    pub section_normal: Vec3,   // Overall cross-section direction
}

#[derive(Debug, Clone)]
pub struct RingGeometry {
    pub points: Vec<Vec3>,      // Points around the ring circumference
    pub normals: Vec<Vec3>,     // Normal at each point
    pub tangents: Vec<Vec3>,    // For bark texture orientation
    pub ring_normal: Vec3,      // Overall ring direction
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
    pub fn new(tree_species: TreeSpecies) -> TreeStructure {
        TreeStructure{
            cross_sections: Vec::new(),
            twigs: Vec::new(),
            species_params: tree_species.clone(),
            age: 0.0,
            overall_health: 0.0,
        }
    }

    pub fn add_cross_section(&mut self, parent_idx: Option<usize>, cross_section: BranchCrossSection) -> usize {
        let new_idx = self.cross_sections.len();
        
        if let Some(parent) = parent_idx {
            self.cross_sections[parent].children_indices.push(new_idx);
        }

        let mut new_cross_section = cross_section;
        new_cross_section.parent_index = parent_idx;
        self.cross_sections.push(new_cross_section);

        new_idx
    }

    pub fn add_component_ring(&mut self, cross_section_idx: usize, ring: ComponentRing) -> RingId {
        let ring_idx = self.cross_sections[cross_section_idx].component_rings.len();
        self.cross_sections[cross_section_idx].component_rings.push(ring);
        
        RingId {
            cross_section_index: cross_section_idx,
            ring_index: ring_idx,
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
    
    fn connect_cross_section_perimeters(
        &self,
        parent_geo: &CrossSectionGeometry,
        child_geo: &CrossSectionGeometry,
        vertices: &mut Vec<Vec3>,
        normals: &mut Vec<Vec3>,
        uvs: &mut Vec<Vec2>,
        indices: &mut Vec<u32>,
    ) {
        let base_vertex_idx = vertices.len() as u32;
        let resolution = parent_geo.points.len().min(child_geo.points.len());

        // Add vertices from both ring perimeters
        for i in 0..resolution {
            vertices.push(parent_geo.points[i]);
            vertices.push(child_geo.points[i]);
        }

        // Calculate surface normals for tubular connection
        for i in 0..resolution {
            let next_i = (i + 1) % resolution;
            
            // Get quad vertices
            let p1 = parent_geo.points[i];
            let p2 = child_geo.points[i];
            let p3 = child_geo.points[next_i];
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
            let p3 = child_geo.points[next_i];
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

    fn angle_range_to_indices(
        &self,
        start_angle: f32,
        end_angle: f32,
        resolution: usize,
    ) -> Vec<usize> {
        let mut indices = Vec::new();

        // Normalize angles to [0, 2π]
        let start = start_angle.rem_euclid(2.0 * std::f32::consts::PI);
        let end = end_angle.rem_euclid(2.0 * std::f32::consts::PI);

        // Calculate how many points we need in this segment
        let angle_span = if end > start {
            end - start
        } else {
            (2.0 * std::f32::consts::PI - start) + end // Handle wrap-around
        };

        let points_in_segment = ((angle_span / (2.0 * std::f32::consts::PI))
            * resolution as f32).max(3.0) as usize; // Minimum 3 points

        // Generate indices for the segment
        for i in 0..points_in_segment {
            let t = i as f32 / (points_in_segment - 1) as f32;
            let angle = start + t * angle_span;
            let index = ((angle / (2.0 * std::f32::consts::PI)) * resolution as f32) as usize % resolution;
            indices.push(index);
        }

        indices
    }

    fn connect_point_segments(
        &self,
        parent_geo: &RingGeometry,
        child_geo: &RingGeometry,
        parent_indices: &[usize],
        child_indices: &[usize],
        vertices: &mut Vec<Vec3>,
        normals: &mut Vec<Vec3>,
        uvs: &mut Vec<Vec2>,
        indices: &mut Vec<u32>,
    ) {
        let base_vertex_idx = vertices.len() as u32;

        // Add vertices from both segments
        for &idx in parent_indices {
            vertices.push(parent_geo.points[idx]);
            normals.push(parent_geo.normals[idx]);
        }

        for &idx in child_indices {
            vertices.push(child_geo.points[idx]);
            normals.push(child_geo.normals[idx]);
        }

        // Generate UVs
        for (i, _) in parent_indices.iter().enumerate() {
            let u = i as f32 / (parent_indices.len() - 1) as f32;
            uvs.push(Vec2::new(u, 0.0));
        }

        for (i, _) in child_indices.iter().enumerate() {
            let u = i as f32 / (child_indices.len() - 1) as f32;
            uvs.push(Vec2::new(u, 1.0));
        }

        // Create triangular mesh between the two segments
        self.triangulate_between_segments(
            parent_indices.len(),
            child_indices.len(),
            base_vertex_idx,
            indices,
        );
    }

    fn triangulate_between_segments(
        &self,
        parent_count: usize,
        child_count: usize,
        base_vertex_idx: u32,
        indices: &mut Vec<u32>,
    ) {
        // This is the tricky part - connecting two segments that may have
        // different numbers of points. We use a "marching" algorithm.

        let mut parent_i = 0;
        let mut child_i = 0;

        while parent_i < parent_count - 1 || child_i < child_count - 1 {
            let p1 = base_vertex_idx + parent_i as u32;
            let p2 = base_vertex_idx + (parent_i + 1).min(parent_count - 1) as u32;
            let c1 = base_vertex_idx + parent_count as u32 + child_i as u32;
            let c2 = base_vertex_idx + parent_count as u32 + (child_i + 1).min(child_count - 1) as u32;

            // Decide which triangle(s) to create based on relative progress
            let parent_progress = parent_i as f32 / (parent_count - 1) as f32;
            let child_progress = child_i as f32 / (child_count - 1) as f32;

            if parent_progress <= child_progress {
                // Advance parent
                if child_i < child_count - 1 {
                    // Create triangle: p1, p2, c1
                    indices.extend_from_slice(&[p1, p2, c1]);
                }
                parent_i += 1;
            } else {
                // Advance child
                if parent_i < parent_count - 1 {
                    // Create triangle: p1, c2, c1
                    indices.extend_from_slice(&[p1, c2, c1]);
                }
                child_i += 1;
            }

            // Sometimes we need a quad (two triangles)
            if parent_i < parent_count - 1 && child_i < child_count - 1 {
                let next_parent_progress = (parent_i + 1) as f32 / (parent_count - 1) as f32;
                let next_child_progress = (child_i + 1) as f32 / (child_count - 1) as f32;

                if (next_parent_progress - child_progress).abs() < 0.1 &&
                    (parent_progress - next_child_progress).abs() < 0.1 {
                    // Create quad: p1, p2, c2, c1
                    indices.extend_from_slice(&[p1, p2, c2]);
                    indices.extend_from_slice(&[p1, c2, c1]);
                    parent_i += 1;
                    child_i += 1;
                }
            }
        }
    }

    fn calculate_child_radius(parent_radius: f32, species: &TreeSpecies, depth: u32) -> f32 {
        parent_radius * species.taper_rate.powf(depth as f32)
    }
}

impl BranchCrossSection {
    pub fn generate_unified_geometry(&self, resolution: u32) -> CrossSectionGeometry {
        let mut points = Vec::new();
        let mut normals = Vec::new();
        let mut tangents = Vec::new();

        // For now, simple approach: if multiple rings, sample around the union of circles
        // TODO: Later implement proper CSG operations for complex shapes
        
        if self.component_rings.is_empty() {
            // No rings - return empty geometry
            return CrossSectionGeometry {
                points,
                normals,
                tangents,
                section_normal: self.orientation * Vec3::Y,
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
            normals,
            tangents,
            section_normal: self.orientation * Vec3::Y,
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
            normals,
            tangents,
            section_normal: self.orientation * Vec3::Y,
        }
    }
}

