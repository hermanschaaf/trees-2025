use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use std::collections::BTreeMap;

// New modular structure
mod core;
mod trunk;
mod branching;
mod roots;
mod twigs;
mod structure;
mod wasm;


use core::{TreeParameters, generator::ModularTreeGenerator};
use structure::TwigType;

#[wasm_bindgen]
pub struct TreeObject {
    params: TreeParameters,
    tree: structure::TreeStructure,
    generator: ModularTreeGenerator,
}

#[wasm_bindgen]
pub struct TreeMesh {
    vertices: Vec<f32>,
    normals: Vec<f32>,
    uvs: Vec<f32>,
    indices: Vec<u32>,
    depths: Vec<u32>, // Depth value for each vertex
}

#[wasm_bindgen]
impl TreeMesh {
    #[wasm_bindgen(getter)]
    pub fn vertices(&self) -> Vec<f32> {
        self.vertices.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn normals(&self) -> Vec<f32> {
        self.normals.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn uvs(&self) -> Vec<f32> {
        self.uvs.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn indices(&self) -> Vec<u32> {
        self.indices.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn depths(&self) -> Vec<u32> {
        self.depths.clone()
    }
}

#[wasm_bindgen]
impl TreeObject {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, trunk_height: f32, butressing: f32) -> Result<TreeObject, JsValue> {
        use core::{GeneralParams, TrunkParams, BranchingParams, RootParams, TwigParams};
        
        let general_params = GeneralParams {
            seed,
            max_depth: 8,
        };
        
        let trunk_params = TrunkParams {
            height: trunk_height,
            buttressing: butressing,
            split_height: trunk_height * 0.6,
            segment_length: 0.3,
            size: 1.0,
            ring_spread: 0.3,
            segment_length_variation: 0.2,
        };
        
        let branching_params = BranchingParams {
            angle_min: 15.0,
            angle_max: 45.0,
            bend_angle_min: 5.0,
            bend_angle_max: 25.0,
            frequency_min: 2,
            frequency_max: 5,
            radius_taper: 0.8,
            azimuth_variation: 0.8,
            max_reach: 25.0,
        };
        
        let root_params = RootParams {
            enable: true,
            depth: 1.5,
            spread: 1.2,
            density: 4,
            segment_length: 0.4,
        };
        
        let twig_params = TwigParams {
            enable: true,
            density: 1.0,
            scale: 0.8,
            angle_variation: 0.6,
        };
        
        let params = TreeParameters {
            general: general_params,
            trunk: trunk_params,
            branching: branching_params,
            roots: root_params,
            twigs: twig_params,
        };
        
        let generator = ModularTreeGenerator::new();
        let tree = generator.generate_tree(&params);
        
        Ok(TreeObject {
            params,
            tree,
            generator,
        })
    }
    
    fn regenerate_tree(&mut self) {
        self.tree = self.generator.generate_tree(&self.params);
    }

    pub fn render(&mut self) {
        // TODO: Generate mesh data for rendering
    }

    pub fn rings_count(&self) -> usize {
        self.tree.cross_sections.len()
    }
    
    pub fn ring_center(&self, index: usize) -> Option<wasm::Vector3d> {
        self.tree.cross_sections.get(index).map(|cross_section| {
            wasm::Vector3d::new(cross_section.center.x, cross_section.center.y, cross_section.center.z)
        })
    }
    
    pub fn ring_radius(&self, index: usize) -> Option<f32> {
        self.tree.cross_sections.get(index).and_then(|cross_section| {
            cross_section.component_rings.first().map(|ring| ring.radius)
        })
    }
    
    pub fn twigs_count(&self) -> usize {
        self.tree.twigs.len()
    }
    
    pub fn twig_position(&self, index: usize) -> Option<wasm::Vector3d> {
        self.tree.twigs.get(index).map(|twig| {
            wasm::Vector3d::new(twig.position.x, twig.position.y, twig.position.z)
        })
    }
    
    pub fn twig_orientation_x(&self, index: usize) -> Option<f32> {
        self.tree.twigs.get(index).map(|twig| twig.orientation.x)
    }
    
    pub fn twig_orientation_y(&self, index: usize) -> Option<f32> {
        self.tree.twigs.get(index).map(|twig| twig.orientation.y)
    }
    
    pub fn twig_orientation_z(&self, index: usize) -> Option<f32> {
        self.tree.twigs.get(index).map(|twig| twig.orientation.z)
    }
    
    pub fn twig_orientation_w(&self, index: usize) -> Option<f32> {
        self.tree.twigs.get(index).map(|twig| twig.orientation.w)
    }
    
    pub fn twig_scale(&self, index: usize) -> Option<f32> {
        self.tree.twigs.get(index).map(|twig| twig.scale)
    }
    
    pub fn twig_type(&self, index: usize) -> Option<String> {
        self.tree.twigs.get(index).map(|twig| {
            match twig.twig_type {
                TwigType::LeafCluster => "LeafCluster".to_string(),
                TwigType::SmallBranch => "SmallBranch".to_string(),
                TwigType::BranchTip => "BranchTip".to_string(),
            }
        })
    }

    pub fn generate_tree_mesh(&self, resolution: u32) -> TreeMesh {
        let ring_mesh = self.tree.generate_mesh(resolution);
        
        // Convert Vec3 vertices to flat f32 array
        let mut vertices = Vec::with_capacity(ring_mesh.vertices.len() * 3);
        for vertex in &ring_mesh.vertices {
            vertices.push(vertex.x);
            vertices.push(vertex.y);
            vertices.push(vertex.z);
        }
        
        // Convert Vec3 normals to flat f32 array
        let mut normals = Vec::with_capacity(ring_mesh.normals.len() * 3);
        for normal in &ring_mesh.normals {
            normals.push(normal.x);
            normals.push(normal.y);
            normals.push(normal.z);
        }
        
        // Convert Vec2 UVs to flat f32 array
        let mut uvs = Vec::with_capacity(ring_mesh.uvs.len() * 2);
        for uv in &ring_mesh.uvs {
            uvs.push(uv.x);
            uvs.push(uv.y);
        }
        
        TreeMesh {
            vertices,
            normals,
            uvs,
            indices: ring_mesh.indices,
            depths: ring_mesh.depths,
        }
    }

    pub fn set_trunk_height(&mut self, height: f32) {
        self.params.trunk.height = height;
        self.regenerate_tree();
    }

    pub fn set_butressing(&mut self, butressing: f32) {
        self.params.trunk.buttressing = butressing;
        self.regenerate_tree();
    }

    pub fn set_split_height(&mut self, split_height: f32) {
        self.params.trunk.split_height = split_height;
        self.regenerate_tree();
    }

    pub fn set_segment_length(&mut self, segment_length: f32) {
        self.params.trunk.segment_length = segment_length.max(0.01);
        self.regenerate_tree();
    }

    pub fn set_branch_angle_range(&mut self, min: f32, max: f32) {
        self.params.branching.angle_min = min.min(max);
        self.params.branching.angle_max = max.max(min);
        self.regenerate_tree();
    }

    pub fn set_bend_angle_range(&mut self, min: f32, max: f32) {
        self.params.branching.bend_angle_min = min.min(max);
        self.params.branching.bend_angle_max = max.max(min);
        self.regenerate_tree();
    }

    pub fn set_branch_frequency_range(&mut self, min: u32, max: u32) {
        let validated_min = min.max(1).min(max.max(1));
        let validated_max = max.max(1).max(validated_min);
        self.params.branching.frequency_min = validated_min;
        self.params.branching.frequency_max = validated_max;
        self.regenerate_tree();
    }

    pub fn set_max_depth(&mut self, max_depth: u32) {
        self.params.general.max_depth = max_depth.max(1);
        self.regenerate_tree();
    }

    pub fn set_radius_taper(&mut self, radius_taper: f32) {
        self.params.branching.radius_taper = radius_taper.max(0.1).min(1.0);
        self.regenerate_tree();
    }

    pub fn set_trunk_ring_spread(&mut self, trunk_ring_spread: f32) {
        self.params.trunk.ring_spread = trunk_ring_spread.max(0.0).min(2.0);
        self.regenerate_tree();
    }

    pub fn set_segment_length_variation(&mut self, segment_length_variation: f32) {
        self.params.trunk.segment_length_variation = segment_length_variation.max(0.0).min(1.0);
        self.regenerate_tree();
    }

    pub fn set_trunk_size(&mut self, trunk_size: f32) {
        self.params.trunk.size = trunk_size.max(0.1).min(10.0);
        self.regenerate_tree();
    }

    pub fn set_branch_azimuth_variation(&mut self, variation: f32) {
        self.params.branching.azimuth_variation = variation.max(0.0).min(1.0);
        self.regenerate_tree();
    }

    pub fn set_max_branch_reach(&mut self, reach: f32) {
        self.params.branching.max_reach = reach.max(2.0).min(50.0);
        self.regenerate_tree();
    }

    // Root system setters
    pub fn set_root_enable(&mut self, enable: bool) {
        self.params.roots.enable = enable;
        self.regenerate_tree();
    }

    pub fn set_root_depth(&mut self, depth: f32) {
        self.params.roots.depth = depth.max(0.5).min(3.0);
        self.regenerate_tree();
    }

    pub fn set_root_spread(&mut self, spread: f32) {
        self.params.roots.spread = spread.max(0.5).min(2.0);
        self.regenerate_tree();
    }

    pub fn set_root_density(&mut self, density: u32) {
        self.params.roots.density = density.max(2).min(8);
        self.regenerate_tree();
    }

    pub fn set_root_segment_length(&mut self, segment_length: f32) {
        self.params.roots.segment_length = segment_length.max(0.1).min(0.8);
        self.regenerate_tree();
    }

    // Twig system setters
    pub fn set_twig_enable(&mut self, enable: bool) {
        self.params.twigs.enable = enable;
        self.regenerate_tree();
    }

    pub fn set_twig_density(&mut self, density: f32) {
        self.params.twigs.density = density.max(0.1).min(2.0);
        self.regenerate_tree();
    }

    pub fn set_twig_scale(&mut self, scale: f32) {
        self.params.twigs.scale = scale.max(0.1).min(3.0);
        self.regenerate_tree();
    }

    pub fn set_twig_angle_variation(&mut self, variation: f32) {
        self.params.twigs.angle_variation = variation.max(0.0).min(1.0);
        self.regenerate_tree();
    }

    /// Export the tree as a GLTF file (returns JSON as string)
    pub fn export_gltf(&self, resolution: u32) -> Result<String, JsValue> {
        let mesh = self.generate_tree_mesh(resolution);
        
        // Create GLTF JSON structure
        let mut root = gltf_json::Root::default();
        
        // Create scene
        let _scene_index = root.push(gltf_json::Scene {
            extensions: Default::default(),
            extras: Default::default(),
            nodes: vec![gltf_json::Index::new(0)],
        });
        root.scene = Some(gltf_json::Index::new(0)); // Scene index is always 0

        // Create node
        root.push(gltf_json::Node {
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            mesh: Some(gltf_json::Index::new(0)),
            rotation: None,
            scale: None,
            translation: None,
            skin: None,
            weights: None,
        });

        // Create mesh
        root.push(gltf_json::Mesh {
            extensions: Default::default(),
            extras: Default::default(),
            primitives: vec![gltf_json::mesh::Primitive {
                attributes: {
                    let mut map = BTreeMap::new();
                    map.insert(
                        gltf_json::validation::Checked::Valid(gltf_json::mesh::Semantic::Positions),
                        gltf_json::Index::new(0), // Position accessor
                    );
                    map.insert(
                        gltf_json::validation::Checked::Valid(gltf_json::mesh::Semantic::Normals),
                        gltf_json::Index::new(1), // Normal accessor
                    );
                    map.insert(
                        gltf_json::validation::Checked::Valid(gltf_json::mesh::Semantic::TexCoords(0)),
                        gltf_json::Index::new(2), // UV accessor
                    );
                    map
                },
                extensions: Default::default(),
                extras: Default::default(),
                indices: Some(gltf_json::Index::new(3)), // Index accessor
                material: None,
                mode: gltf_json::validation::Checked::Valid(gltf_json::mesh::Mode::Triangles),
                targets: None,
            }],
            weights: None,
        });

        // Create buffer data
        let vertices_bytes = mesh.vertices.iter()
            .flat_map(|f| f.to_le_bytes().to_vec())
            .collect::<Vec<u8>>();
        
        let normals_bytes = mesh.normals.iter()
            .flat_map(|f| f.to_le_bytes().to_vec())
            .collect::<Vec<u8>>();
            
        let uvs_bytes = mesh.uvs.iter()
            .flat_map(|f| f.to_le_bytes().to_vec())
            .collect::<Vec<u8>>();
            
        let indices_bytes = mesh.indices.iter()
            .flat_map(|i| i.to_le_bytes().to_vec())
            .collect::<Vec<u8>>();

        // Combine all data into single buffer
        let mut buffer_data = Vec::new();
        let vertices_offset = 0;
        let normals_offset = vertices_bytes.len();
        let uvs_offset = normals_offset + normals_bytes.len();
        let indices_offset = uvs_offset + uvs_bytes.len();
        
        buffer_data.extend(vertices_bytes);
        buffer_data.extend(normals_bytes);
        buffer_data.extend(uvs_bytes);
        buffer_data.extend(indices_bytes);

        // Create buffer
        use base64::Engine as _;
        let buffer_data_base64 = base64::engine::general_purpose::STANDARD.encode(&buffer_data);
        let buffer_uri = format!("data:application/octet-stream;base64,{}", buffer_data_base64);
        
        root.push(gltf_json::Buffer {
            byte_length: gltf_json::validation::USize64::from(buffer_data.len()),
            extensions: Default::default(),
            extras: Default::default(),
            uri: Some(buffer_uri),
        });

        // Create buffer view for vertices
        root.push(gltf_json::buffer::View {
            buffer: gltf_json::Index::new(0),
            byte_length: gltf_json::validation::USize64::from(mesh.vertices.len() * 4),
            byte_offset: Some(gltf_json::validation::USize64::from(vertices_offset as usize)),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: Some(gltf_json::validation::Checked::Valid(gltf_json::buffer::Target::ArrayBuffer)),
        });

        // Create buffer view for normals
        root.push(gltf_json::buffer::View {
            buffer: gltf_json::Index::new(0),
            byte_length: gltf_json::validation::USize64::from(mesh.normals.len() * 4),
            byte_offset: Some(gltf_json::validation::USize64::from(normals_offset as usize)),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: Some(gltf_json::validation::Checked::Valid(gltf_json::buffer::Target::ArrayBuffer)),
        });

        // Create buffer view for UVs
        root.push(gltf_json::buffer::View {
            buffer: gltf_json::Index::new(0),
            byte_length: gltf_json::validation::USize64::from(mesh.uvs.len() * 4),
            byte_offset: Some(gltf_json::validation::USize64::from(uvs_offset as usize)),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: Some(gltf_json::validation::Checked::Valid(gltf_json::buffer::Target::ArrayBuffer)),
        });

        // Create buffer view for indices
        root.push(gltf_json::buffer::View {
            buffer: gltf_json::Index::new(0),
            byte_length: gltf_json::validation::USize64::from(mesh.indices.len() * 4),
            byte_offset: Some(gltf_json::validation::USize64::from(indices_offset as usize)),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: Some(gltf_json::validation::Checked::Valid(gltf_json::buffer::Target::ElementArrayBuffer)),
        });

        // Create accessors
        // Position accessor
        root.push(gltf_json::Accessor {
            buffer_view: Some(gltf_json::Index::new(0)),
            byte_offset: Some(gltf_json::validation::USize64::from(0_usize)),
            component_type: gltf_json::validation::Checked::Valid(gltf_json::accessor::GenericComponentType(gltf_json::accessor::ComponentType::F32)),
            count: gltf_json::validation::USize64::from(mesh.vertices.len() / 3),
            extensions: Default::default(),
            extras: Default::default(),
            type_: gltf_json::validation::Checked::Valid(gltf_json::accessor::Type::Vec3),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });

        // Normal accessor
        root.push(gltf_json::Accessor {
            buffer_view: Some(gltf_json::Index::new(1)),
            byte_offset: Some(gltf_json::validation::USize64::from(0_usize)),
            component_type: gltf_json::validation::Checked::Valid(gltf_json::accessor::GenericComponentType(gltf_json::accessor::ComponentType::F32)),
            count: gltf_json::validation::USize64::from(mesh.normals.len() / 3),
            extensions: Default::default(),
            extras: Default::default(),
            type_: gltf_json::validation::Checked::Valid(gltf_json::accessor::Type::Vec3),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });

        // UV accessor
        root.push(gltf_json::Accessor {
            buffer_view: Some(gltf_json::Index::new(2)),
            byte_offset: Some(gltf_json::validation::USize64::from(0_usize)),
            component_type: gltf_json::validation::Checked::Valid(gltf_json::accessor::GenericComponentType(gltf_json::accessor::ComponentType::F32)),
            count: gltf_json::validation::USize64::from(mesh.uvs.len() / 2),
            extensions: Default::default(),
            extras: Default::default(),
            type_: gltf_json::validation::Checked::Valid(gltf_json::accessor::Type::Vec2),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });

        // Index accessor
        root.push(gltf_json::Accessor {
            buffer_view: Some(gltf_json::Index::new(3)),
            byte_offset: Some(gltf_json::validation::USize64::from(0_usize)),
            component_type: gltf_json::validation::Checked::Valid(gltf_json::accessor::GenericComponentType(gltf_json::accessor::ComponentType::U32)),
            count: gltf_json::validation::USize64::from(mesh.indices.len()),
            extensions: Default::default(),
            extras: Default::default(),
            type_: gltf_json::validation::Checked::Valid(gltf_json::accessor::Type::Scalar),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });

        // Serialize to JSON
        match serde_json::to_string_pretty(&root) {
            Ok(json) => Ok(json),
            Err(e) => Err(JsValue::from_str(&format!("Failed to serialize GLTF: {}", e))),
        }
    }
}

// Public API: generate a Tree
#[wasm_bindgen]
pub fn generate(seed: u32, trunk_height: f32, butressing: f32) -> Result<TreeObject, JsValue> {
    TreeObject::new(seed, trunk_height, butressing)
}

