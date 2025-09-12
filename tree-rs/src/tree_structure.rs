use glam::{Vec2, Vec3, Quat};

#[derive(Debug)]
pub struct TreeStructure {
    pub rings: Vec<TreeRing>,
    pub root_index: usize,      // Usually 0, but flexible

    // Metadata
    pub species_params: TreeSpecies,
    pub age: f32,
    pub overall_health: f32,
}

#[derive(Debug, Clone)]
pub struct TreeRing {
    // Geometric properties
    pub center: Vec3,           // Position in 3D space
    pub radius: f32,            // Ring radius
    pub orientation: Quat,      // Ring orientation (normal direction)

    // Connectivity
    pub parent_index: Option<usize>,  // Index of parent ring
    pub children_indices: Vec<usize>, // Indices of child rings
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
}

#[derive(Debug, Clone)]
pub struct RingGeometry {
    pub points: Vec<Vec3>,      // Points around the ring circumference
    pub normals: Vec<Vec3>,     // Normal at each point
    pub tangents: Vec<Vec3>,    // For bark texture orientation
    pub ring_normal: Vec3,      // Overall ring direction
}

impl TreeStructure {
    pub fn new(tree_species: TreeSpecies) -> TreeStructure {
        TreeStructure{
            rings: Vec::new(),
            root_index: 0,
            species_params: tree_species.clone(),
            age: 0.0,
            overall_health: 0.0,
        }
    }

    pub fn add_ring(&mut self, parent_idx: usize, ring: TreeRing) -> usize {
        let new_idx = self.rings.len();
        self.rings[parent_idx].children_indices.push(new_idx);

        let mut new_ring = ring;
        new_ring.parent_index = Some(parent_idx);
        self.rings.push(new_ring);

        new_idx
    }

    pub fn generate_mesh(&self, ring_resolution: u32) -> RingMesh {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        // Generate geometry for each ring (just perimeter points, no faces)
        let ring_geometries: Vec<RingGeometry> = self.rings
            .iter()
            .map(|ring| ring.generate_geometry(ring_resolution))
            .collect();

        // Connect parent rings to children with tubular surfaces
        // This creates the actual tree mesh by connecting ring perimeters
        for (ring_idx, ring) in self.rings.iter().enumerate() {
            for &child_idx in &ring.children_indices {
                self.connect_ring_perimeters(
                    &ring_geometries[ring_idx],
                    &ring_geometries[child_idx],
                    &mut vertices,
                    &mut normals,
                    &mut uvs,
                    &mut indices,
                );
            }
        }

        RingMesh { vertices, normals, uvs, indices }
    }


    fn connect_ring_perimeters(
        &self,
        parent_geo: &RingGeometry,
        child_geo: &RingGeometry,
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

    fn handle_branch_split(
        &self,
        _parent_geo: &RingGeometry,
        _children_geo: &[RingGeometry],
        _branch_angles: &[f32], // Where each child branches off
    ) {
        // TODO: Create transition geometry that smoothly splits the parent ring
        // into multiple child rings
    }

    fn connect_ring_segment(
        &self,
        parent_geo: &RingGeometry,
        child_geo: &RingGeometry,
        start_angle: f32,
        end_angle: f32,
    ) {
        // This is a placeholder for segment connection logic
        // In a full implementation, this would create the mesh data
        // for connecting a portion of parent ring to child ring
        let _parent_resolution = parent_geo.points.len();
        let _child_resolution = child_geo.points.len();
        
        // TODO: Implement segment connection logic
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

impl TreeRing {
    pub fn generate_geometry(&self, resolution: u32) -> RingGeometry {
        let mut points = Vec::with_capacity(resolution as usize);
        let mut normals = Vec::with_capacity(resolution as usize);
        let mut tangents = Vec::with_capacity(resolution as usize);

        // Create points around the ring
        for i in 0..resolution {
            let angle = (i as f32 / resolution as f32) * 2.0 * std::f32::consts::PI;

            // Local ring coordinates in XZ plane (horizontal ring)
            let local_x = angle.cos() * self.radius;
            let local_z = angle.sin() * self.radius;
            let local_point = Vec3::new(local_x, 0.0, local_z);

            // Transform to world space using ring orientation
            let world_point = self.center + self.orientation * local_point;
            let world_normal = self.orientation * local_point.normalize();
            let world_tangent = self.orientation * Vec3::new(-local_z, 0.0, local_x).normalize();

            points.push(world_point);
            normals.push(world_normal);
            tangents.push(world_tangent);
        }

        RingGeometry {
            points,
            normals,
            tangents,
            ring_normal: self.orientation * Vec3::Y, // Y is up, so ring normal is Y
        }
    }
}