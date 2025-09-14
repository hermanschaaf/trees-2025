import * as THREE from 'three';
import * as dat from 'dat.gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';

import init, { generate } from "/static/js/tree_rs.js";

await init();
const tree = generate(123, 5.0, 1.0);
console.log("Rust generate:", tree);

// Load twig library metadata
const loadTwigLibrary = async (): Promise<TwigLibrary> => {
    try {
        const response = await fetch('/static/twigs/twigs-library.json');
        if (!response.ok) {
            throw new Error(`Failed to load twig library: ${response.statusText}`);
        }
        return await response.json();
    } catch (error) {
        console.error('Error loading twig library:', error);
        // Return default library with just procedural twigs
        return {
            version: '1.0',
            description: 'Default twig library (fallback)',
            twigs: [{
                id: 'procedural',
                name: 'Procedural Twigs',
                description: 'Generated procedurally using basic geometries',
                type: 'procedural',
                filename: null,
                defaultScale: 1.0,
                preview: {
                    thumbnail: null,
                    description: 'Simple procedural leaves and branches'
                }
            }]
        };
    }
};

// Load GLTF model with caching
const loadTwigModel = async (filename: string): Promise<THREE.Object3D | null> => {
    if (twigModelCache.has(filename)) {
        return twigModelCache.get(filename)!;
    }
    
    try {
        const gltf = await new Promise<any>((resolve, reject) => {
            gltfLoader.load(
                `/static/twigs/${filename}`,
                resolve,
                undefined,
                reject
            );
        });
        
        const model = gltf.scene.clone();
        twigModelCache.set(filename, model);
        console.log(`Loaded GLTF model: ${filename}`);
        return model;
    } catch (error) {
        console.error(`Failed to load GLTF model ${filename}:`, error);
        return null;
    }
};

console.log("trunk_height: ", tree.trunk_height);
console.log("buttressing: ", tree.butressing);
console.log("rings_count: ", tree.rings_count());

// Scene setup
const scene = new THREE.Scene();
scene.background = new THREE.Color(0xf0f0f0); // Light gray background
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setSize(window.innerWidth, window.innerHeight);
renderer.shadowMap.enabled = true;
renderer.shadowMap.type = THREE.PCFSoftShadowMap;
document.body.appendChild(renderer.domElement);

// Tree parameters
const treeParams = {
    height: tree.trunk_height,
    butressing: tree.butressing,
    splitHeight: tree.split_height, // Height at which trunk splits into branches
    segmentLength: tree.segment_length,
    branchAngleMin: tree.branch_angle_min,
    branchAngleMax: tree.branch_angle_max,
    bendAngleMin: tree.bend_angle_min,
    bendAngleMax: tree.bend_angle_max,
    branchFrequencyMin: tree.branch_frequency_min,
    branchFrequencyMax: tree.branch_frequency_max,
    maxDepth: tree.max_depth,
    radiusTaper: tree.radius_taper,
    trunkRingSpread: tree.trunk_ring_spread,
    segmentLengthVariation: 0.3, // Default value since it's not exposed yet
    trunkSize: tree.trunk_size, // Base trunk radius multiplier
    radius: 0.5,
    radialSegments: 32,
    // Root system parameters
    rootEnable: tree.root_enable,
    rootDepth: tree.root_depth,
    rootSpread: tree.root_spread,
    rootDensity: tree.root_density,
    rootSegmentLength: tree.root_segment_length,
    // Twig system parameters
    twigEnable: tree.twig_enable,
    twigDensity: tree.twig_density,
    twigScale: 1.0, // Default value, will be updated by the tree object
    twigAngleVariation: tree.twig_angle_variation
};

let ringMeshes: THREE.Mesh[] = [];
let twigMeshes: THREE.Mesh[] = [];

// Twig library system
interface TwigLibraryItem {
    id: string;
    name: string;
    description: string;
    type: 'procedural' | 'gltf';
    filename: string | null;
    defaultScale: number;
    preview: {
        thumbnail: string | null;
        description: string;
    };
}

interface TwigLibrary {
    version: string;
    description: string;
    twigs: TwigLibraryItem[];
}

let twigLibrary: TwigLibrary | null = null;
let gltfLoader: GLTFLoader;
let twigModelCache: Map<string, THREE.Object3D> = new Map();
let selectedTwigType = 'procedural';

// Bark texture system
interface BarkTextureItem {
    id: string;
    name: string;
    description: string;
    type: 'procedural' | 'texture';
    diffuse: string | null;
    normal: string | null;
    roughness: string | null;
    scale: number;
    tiling: [number, number];
    color: string;
}

interface BarkLibrary {
    version: string;
    description: string;
    textures: BarkTextureItem[];
    materialProperties: {
        metalness: number;
        roughnessScale: number;
        normalScale: number;
        envMapIntensity: number;
    };
}

let barkLibrary: BarkLibrary | null = null;
let textureLoader: THREE.TextureLoader;
let barkTextureCache: Map<string, THREE.Texture> = new Map();
let selectedBarkType = 'default';

// Initialize GLTF loader and texture loader
gltfLoader = new GLTFLoader();
textureLoader = new THREE.TextureLoader();

// Load bark library metadata
const loadBarkLibrary = async (): Promise<BarkLibrary> => {
    try {
        const response = await fetch('/static/textures/bark/bark-library.json');
        if (!response.ok) {
            throw new Error(`Failed to load bark library: ${response.statusText}`);
        }
        return await response.json();
    } catch (error) {
        console.error('Error loading bark library:', error);
        // Return default library with just procedural bark
        return {
            version: '1.0',
            description: 'Default bark library (fallback)',
            textures: [{
                id: 'default',
                name: 'Default (No Texture)',
                description: 'Solid color material without texture',
                type: 'procedural',
                diffuse: null,
                normal: null,
                roughness: null,
                scale: 1.0,
                tiling: [1.0, 1.0],
                color: '#8B4513'
            }],
            materialProperties: {
                metalness: 0.0,
                roughnessScale: 1.0,
                normalScale: 1.0,
                envMapIntensity: 0.3
            }
        };
    }
};

// Load bark texture with caching
const loadBarkTexture = async (filename: string): Promise<THREE.Texture | null> => {
    if (barkTextureCache.has(filename)) {
        return barkTextureCache.get(filename)!;
    }
    
    try {
        const texture = await new Promise<THREE.Texture>((resolve, reject) => {
            textureLoader.load(
                `/static/textures/bark/${filename}`,
                resolve,
                undefined,
                reject
            );
        });
        
        // Configure texture settings
        texture.wrapS = THREE.RepeatWrapping;
        texture.wrapT = THREE.RepeatWrapping;
        texture.generateMipmaps = true;
        texture.minFilter = THREE.LinearMipmapLinearFilter;
        texture.magFilter = THREE.LinearFilter;
        
        barkTextureCache.set(filename, texture);
        console.log(`Loaded bark texture: ${filename}`);
        return texture;
    } catch (error) {
        console.error(`Failed to load bark texture ${filename}:`, error);
        return null;
    }
};

// Initialize libraries
twigLibrary = await loadTwigLibrary();
barkLibrary = await loadBarkLibrary();
console.log('Loaded twig library:', twigLibrary);
console.log('Loaded bark library:', barkLibrary);

// Create bark material function
const createBarkMaterial = async (barkType: string): Promise<THREE.MeshStandardMaterial> => {
    if (!barkLibrary) {
        // Fallback material
        return new THREE.MeshStandardMaterial({
            color: 0x8B4513,
            roughness: 0.8,
            metalness: 0.0
        });
    }
    
    const barkTexture = barkLibrary.textures.find(t => t.id === barkType);
    if (!barkTexture) {
        // Fallback material
        return new THREE.MeshStandardMaterial({
            color: 0x8B4513,
            roughness: 0.8,
            metalness: 0.0
        });
    }
    
    const material = new THREE.MeshStandardMaterial({
        color: new THREE.Color(barkTexture.color),
        roughness: 0.8,
        metalness: barkLibrary.materialProperties.metalness
    });
    
    // Load textures if specified
    if (barkTexture.type === 'texture') {
        // Load diffuse texture
        if (barkTexture.diffuse) {
            const diffuseTexture = await loadBarkTexture(barkTexture.diffuse);
            if (diffuseTexture) {
                diffuseTexture.repeat.set(barkTexture.tiling[0], barkTexture.tiling[1]);
                material.map = diffuseTexture;
            }
        }
        
        // Load normal texture
        if (barkTexture.normal) {
            const normalTexture = await loadBarkTexture(barkTexture.normal);
            if (normalTexture) {
                normalTexture.repeat.set(barkTexture.tiling[0], barkTexture.tiling[1]);
                material.normalMap = normalTexture;
                material.normalScale = new THREE.Vector2(
                    barkLibrary.materialProperties.normalScale,
                    barkLibrary.materialProperties.normalScale
                );
            }
        }
        
        // Load roughness texture
        if (barkTexture.roughness) {
            const roughnessTexture = await loadBarkTexture(barkTexture.roughness);
            if (roughnessTexture) {
                roughnessTexture.repeat.set(barkTexture.tiling[0], barkTexture.tiling[1]);
                material.roughnessMap = roughnessTexture;
            }
        }
    }
    
    return material;
};

// Create initial materials
let ringMaterial = await createBarkMaterial(selectedBarkType);
const cylinderMaterial = new THREE.MeshStandardMaterial({ 
    color: 0x654321, // Darker brown for branches
    roughness: 0.8,
    metalness: 0.0
});

// Create materials for twigs
const leafMaterial = new THREE.MeshLambertMaterial({ 
    color: 0x228B22, // Forest green
    side: THREE.DoubleSide // Show both sides of leaves
});
const twigBranchMaterial = new THREE.MeshStandardMaterial({ 
    color: 0x8B4513, // Brown like main branches
    roughness: 0.7,
    metalness: 0.0
});
const budMaterial = new THREE.MeshStandardMaterial({ 
    color: 0x90EE90, // Light green for buds
    roughness: 0.6,
    metalness: 0.0
});

// Performance optimization: use instanced rendering for GLTF twigs
let twigInstancedMeshes: THREE.InstancedMesh[] = [];
let twigInstanceCount = 0;
const maxTwigInstances = 1000; // Reasonable limit to prevent crashes

// Create twig geometries for different twig types
const createTwigGeometry = async (twigType: string, scale: number): Promise<THREE.Object3D | null> => {
    // Check if we should use GLTF models
    if (selectedTwigType !== 'procedural' && twigLibrary) {
        const selectedTwig = twigLibrary.twigs.find(t => t.id === selectedTwigType);
        if (selectedTwig && selectedTwig.type === 'gltf' && selectedTwig.filename) {
            // Return null for GLTF models - we'll handle them with instancing
            return null;
        }
    }
    
    // Fallback to procedural generation
    switch (twigType) {
        case 'LeafCluster':
            // Create a cluster of small leaf planes
            const leafClusterGeometry = new THREE.Group();
            for (let i = 0; i < 5; i++) {
                const leafGeometry = new THREE.PlaneGeometry(0.1 * scale, 0.15 * scale);
                const leafMesh = new THREE.Mesh(leafGeometry, leafMaterial);
                
                // Random positioning within cluster
                const angle = (i / 5) * Math.PI * 2;
                leafMesh.position.set(
                    Math.cos(angle) * 0.05 * scale,
                    Math.random() * 0.1 * scale,
                    Math.sin(angle) * 0.05 * scale
                );
                leafMesh.rotation.set(
                    Math.random() * 0.5,
                    angle + Math.random() * 0.5,
                    Math.random() * 0.5
                );
                leafClusterGeometry.add(leafMesh);
            }
            return leafClusterGeometry;
            
        case 'SmallBranch':
            // Create a small branch with a few leaves
            const branchGroup = new THREE.Group();
            
            // Small branch cylinder
            const branchGeometry = new THREE.CylinderGeometry(0.01 * scale, 0.02 * scale, 0.2 * scale, 6);
            const branchMesh = new THREE.Mesh(branchGeometry, twigBranchMaterial);
            branchMesh.position.y = 0.1 * scale;
            branchGroup.add(branchMesh);
            
            // Add 2-3 leaves
            for (let i = 0; i < 3; i++) {
                const leafGeometry = new THREE.PlaneGeometry(0.08 * scale, 0.12 * scale);
                const leafMesh = new THREE.Mesh(leafGeometry, leafMaterial);
                leafMesh.position.set(
                    (Math.random() - 0.5) * 0.1 * scale,
                    0.15 * scale + Math.random() * 0.05 * scale,
                    (Math.random() - 0.5) * 0.1 * scale
                );
                leafMesh.rotation.set(
                    Math.random() * 0.3,
                    Math.random() * Math.PI,
                    Math.random() * 0.3
                );
                branchGroup.add(leafMesh);
            }
            return branchGroup;
            
        case 'BranchTip':
        default:
            // Create a simple bud at branch tip
            const budGeometry = new THREE.SphereGeometry(0.03 * scale, 6, 4);
            const budMesh = new THREE.Mesh(budGeometry, budMaterial);
            return budMesh;
    }
};

// Create instanced mesh for GLTF twigs
const createTwigInstancedMesh = async (twigCount: number): Promise<void> => {
    if (selectedTwigType === 'procedural' || !twigLibrary) return;
    
    const selectedTwig = twigLibrary.twigs.find(t => t.id === selectedTwigType);
    if (!selectedTwig || selectedTwig.type !== 'gltf' || !selectedTwig.filename) return;
    
    // Limit the number of instances to prevent crashes
    const actualInstanceCount = Math.min(twigCount, maxTwigInstances);
    
    // Load the GLTF model
    const model = await loadTwigModel(selectedTwig.filename);
    if (!model) return;
    
    // Extract geometry and materials from all meshes in the model
    const meshData: Array<{geometry: THREE.BufferGeometry, material: THREE.Material}> = [];
    
    model.traverse((child) => {
        if (child instanceof THREE.Mesh) {
            // Clone the material properly
            let clonedMaterial: THREE.Material;
            
            if (Array.isArray(child.material)) {
                // Handle multi-material case - use first material
                clonedMaterial = child.material[0].clone();
                console.log('GLTF twig has multi-material, using first:', clonedMaterial.type);
            } else {
                clonedMaterial = child.material.clone();
                console.log('GLTF twig material type:', clonedMaterial.type);
            }
            
            // Debug material properties
            if ('map' in clonedMaterial && clonedMaterial.map) {
                console.log('Material has diffuse texture:', clonedMaterial.map);
            }
            if ('normalMap' in clonedMaterial && clonedMaterial.normalMap) {
                console.log('Material has normal map');
            }
            
            // Ensure material is compatible with instanced rendering
            if ('transparent' in clonedMaterial) {
                // Force some settings that work better with instanced meshes
                clonedMaterial.transparent = true;
                clonedMaterial.alphaTest = 0.01; // Prevent z-fighting issues
            }

            meshData.push({
                geometry: child.geometry.clone(),
                material: clonedMaterial
            });
        }
    });
    
    if (meshData.length === 0) {
        console.warn('No meshes found in GLTF twig model');
        return;
    }
    
    console.log(`Found ${meshData.length} meshes in GLTF twig model`);
    
    // Respect the GLTF's original pivot point and transformations
    // Extract any transformation data from the GLTF nodes
    let modelTransform = new THREE.Matrix4();
    
    model.traverse((child) => {
        if (child instanceof THREE.Mesh) {
            // Get the mesh's world matrix which includes all parent transformations
            child.updateMatrixWorld(true);
            console.log('GLTF mesh transform:', child.matrixWorld.elements);
            
            // For now, let's try using the model as-is without offset calculations
            console.log('Mesh position:', child.position);
            console.log('Mesh scale:', child.scale);
            console.log('Mesh rotation:', child.rotation);
        }
    });
    
    // Create instanced meshes for each mesh in the GLTF model
    for (const meshInfo of meshData) {
        const instancedMesh = new THREE.InstancedMesh(meshInfo.geometry, meshInfo.material, actualInstanceCount);
        instancedMesh.castShadow = true;
        instancedMesh.receiveShadow = true;
        
        // Don't apply any custom offset - let the GLTF's original pivot be used
        instancedMesh.userData = { useOriginalPivot: true };

        scene.add(instancedMesh);
        twigInstancedMeshes.push(instancedMesh);
    }
    
    twigInstanceCount = 0;
    
    console.log(`Created ${twigInstancedMeshes.length} instanced meshes for ${actualInstanceCount} twig instances each`);
};

// Create tree visualization using generated mesh
const createTreeVisualization = async () => {
    // Clear existing meshes
    ringMeshes.forEach(mesh => {
        scene.remove(mesh);
        mesh.geometry.dispose();
    });
    ringMeshes = [];
    
    // Clear existing twigs
    twigMeshes.forEach(mesh => {
        scene.remove(mesh);
        // Dispose geometries for Groups and individual meshes
        if (mesh instanceof THREE.Group) {
            mesh.children.forEach(child => {
                if (child instanceof THREE.Mesh) {
                    child.geometry.dispose();
                }
            });
        } else {
            mesh.geometry.dispose();
        }
    });
    twigMeshes = [];
    
    // Clear existing instanced meshes
    twigInstancedMeshes.forEach(mesh => {
        scene.remove(mesh);
        mesh.dispose();
    });
    twigInstancedMeshes = [];
    twigInstanceCount = 0;
    
    // Generate mesh from Rust
    const treeMesh = tree.generate_tree_mesh(16); // 16 points per ring
    
    if (treeMesh.vertices.length > 0) {
        // Create Three.js geometry from the generated mesh
        const geometry = new THREE.BufferGeometry();
        
        // Set vertices (convert flat array to THREE.js format)
        const vertices = new Float32Array(treeMesh.vertices);
        geometry.setAttribute('position', new THREE.BufferAttribute(vertices, 3));
        
        // Set normals
        const normals = new Float32Array(treeMesh.normals);
        geometry.setAttribute('normal', new THREE.BufferAttribute(normals, 3));
        
        // Set UVs
        const uvs = new Float32Array(treeMesh.uvs);
        geometry.setAttribute('uv', new THREE.BufferAttribute(uvs, 2));
        
        // Set indices
        const indices = new Uint32Array(treeMesh.indices);
        geometry.setIndex(new THREE.BufferAttribute(indices, 1));
        
        // Create mesh
        const mesh = new THREE.Mesh(geometry, ringMaterial);
        mesh.castShadow = true;
        mesh.receiveShadow = true;
        
        ringMeshes.push(mesh);
        scene.add(mesh);
        
        console.log(`Created tree mesh with ${treeMesh.vertices.length/3} vertices and ${treeMesh.indices.length/3} triangles`);
    } else {
        console.warn('No vertices found in tree mesh');
    }
    
    // Generate twigs if enabled
    if (treeParams.twigEnable) {
        const twigCount = tree.twigs_count();
        console.log(`Generating ${twigCount} twigs`);
        
        // Use instanced rendering for GLTF models
        if (selectedTwigType !== 'procedural') {
            await createTwigInstancedMesh(twigCount);
            
            if (twigInstancedMeshes.length > 0) {
                // Set instance transforms for all instanced meshes
                const matrix = new THREE.Matrix4();
                const instanceCount = Math.min(twigCount, maxTwigInstances);
                
                for (let i = 0; i < instanceCount; i++) {
                    const position = tree.twig_position(i);
                    const scale = tree.twig_scale(i);
                    const orientationX = tree.twig_orientation_x(i);
                    const orientationY = tree.twig_orientation_y(i);
                    const orientationZ = tree.twig_orientation_z(i);
                    const orientationW = tree.twig_orientation_w(i);
                    
                    if (position && scale !== null && 
                        orientationX !== null && orientationY !== null && 
                        orientationZ !== null && orientationW !== null) {
                        
                        // Apply scale from twig library
                        const selectedTwig = twigLibrary?.twigs.find(t => t.id === selectedTwigType);
                        const finalScale = scale * (selectedTwig?.defaultScale || 1.0);
                        
                        // Create transform matrix with offset correction
                        const quaternion = new THREE.Quaternion(orientationX, orientationY, orientationZ, orientationW);
                        const fix = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1,0,0), Math.PI/2);
                        quaternion.premultiply(fix);
                        // Apply each instanced mesh - try with no offset first
                        twigInstancedMeshes.forEach(mesh => {
                            let finalPosition = new THREE.Vector3(position.x, position.y, position.z);

                            matrix.compose(
                                finalPosition,
                                quaternion,
                                new THREE.Vector3(finalScale, finalScale, finalScale)
                            );
                            
                            mesh.setMatrixAt(i, matrix);
                        });
                    }
                }
                
                // Notify Three.js that instance data changed for all meshes
                twigInstancedMeshes.forEach(mesh => {
                    mesh.instanceMatrix.needsUpdate = true;
                });
                
                console.log(`Created ${twigInstancedMeshes.length} instanced meshes with ${instanceCount} instances each (limited from ${twigCount} total)`);
            }
        } else {
            // Use individual meshes for procedural twigs (they're lightweight)
            const twigPromises: Promise<void>[] = [];
            const maxProceduralTwigs = 500; // Even procedural twigs should be limited
            const proceduralCount = Math.min(twigCount, maxProceduralTwigs);
            
            for (let i = 0; i < proceduralCount; i++) {
                const position = tree.twig_position(i);
                const scale = tree.twig_scale(i);
                const twigType = tree.twig_type(i);
                const orientationX = tree.twig_orientation_x(i);
                const orientationY = tree.twig_orientation_y(i);
                const orientationZ = tree.twig_orientation_z(i);
                const orientationW = tree.twig_orientation_w(i);
                
                if (position && scale !== null && twigType && 
                    orientationX !== null && orientationY !== null && 
                    orientationZ !== null && orientationW !== null) {
                    
                    // Create async task for each twig
                    const twigPromise = createTwigGeometry(twigType, scale).then((twigGeometry) => {
                        if (twigGeometry) {
                            // Position the twig
                            twigGeometry.position.set(position.x, position.y, position.z);
                            
                            // Apply quaternion rotation
                            const quaternion = new THREE.Quaternion(orientationX, orientationY, orientationZ, orientationW);
                            twigGeometry.setRotationFromQuaternion(quaternion);
                            
                            // Enable shadows
                            twigGeometry.traverse((child) => {
                                if (child instanceof THREE.Mesh) {
                                    child.castShadow = true;
                                    child.receiveShadow = true;
                                }
                            });
                            
                            scene.add(twigGeometry);
                            twigMeshes.push(twigGeometry as any); // Store for cleanup
                        }
                    });
                    
                    twigPromises.push(twigPromise);
                }
            }
            
            // Wait for all procedural twigs to load
            await Promise.all(twigPromises);
            
            console.log(`Created ${twigMeshes.length} procedural twig instances (limited from ${twigCount} total)`);
        }
    }
};

// Create initial tree visualization
createTreeVisualization().catch(error => {
    console.error('Error creating initial tree visualization:', error);
});

// Improved lighting setup
const ambientLight = new THREE.AmbientLight(0x909090, 0.7); // Soft ambient light
scene.add(ambientLight);

// Main directional light (sun)
const directionalLight = new THREE.DirectionalLight(0xffffff, 1.0);
directionalLight.position.set(10, 10, 5);
directionalLight.castShadow = true;
directionalLight.shadow.mapSize.width = 2048;
directionalLight.shadow.mapSize.height = 2048;
directionalLight.shadow.camera.near = 0.5;
directionalLight.shadow.camera.far = 50;
directionalLight.shadow.camera.left = -10;
directionalLight.shadow.camera.right = 10;
directionalLight.shadow.camera.top = 10;
directionalLight.shadow.camera.bottom = -10;
scene.add(directionalLight);

// Secondary light for fill lighting
const fillLight = new THREE.DirectionalLight(0xffffff, 0.3);
fillLight.position.set(-5, 3, -5);
scene.add(fillLight);

// Add a ground plane to receive shadows
const groundGeometry = new THREE.PlaneGeometry(20, 20);
const groundMaterial = new THREE.MeshPhongMaterial({ color: 0x999999 });
const ground = new THREE.Mesh(groundGeometry, groundMaterial);
ground.rotation.x = -Math.PI / 2;
ground.position.y = -0.1;
ground.receiveShadow = true;
scene.add(ground);

// Position the camera
camera.position.set(9, 10, 9);
camera.lookAt(0, 0, 0);

// Add orbit controls for mouse/trackpad interaction
const controls = new OrbitControls(camera, renderer.domElement);
controls.target.set(0, treeParams.height * 1.8, 0);
controls.enableDamping = true;
controls.dampingFactor = 0.05;

// GUI Controls
const gui = new dat.GUI({ width: 320 });
const treeFolder = gui.addFolder('Tree Parameters');

// Update bark material for all tree meshes
const updateBarkMaterial = async () => {
    try {
        const newMaterial = await createBarkMaterial(selectedBarkType);
        
        // Update all ring meshes with new material
        ringMeshes.forEach(mesh => {
            // Dispose old material
            if (mesh.material instanceof THREE.Material) {
                mesh.material.dispose();
            }
            mesh.material = newMaterial.clone();
        });
        
        // Store the updated material as the new default
        ringMaterial.dispose();
        ringMaterial = newMaterial;
        
        console.log(`Updated bark material to: ${selectedBarkType}`);
    } catch (error) {
        console.error('Error updating bark material:', error);
    }
};

function redrawTree() {
    createTreeVisualization().catch(error => {
        console.error('Error redrawing tree:', error);
    });
}

treeFolder.add(treeParams, 'height', 0.1, 10).onChange((value: number) => {
    tree.set_trunk_height(value);
    treeParams.height = value;
    redrawTree();
});

treeFolder.add(treeParams, 'butressing', 0.1, 5).onChange((value: number) => {
    tree.set_butressing(value);
    treeParams.butressing = value;
    redrawTree();
});

treeFolder.add(treeParams, 'splitHeight', 0.5, 8).onChange((value: number) => {
    tree.set_split_height(value);
    treeParams.splitHeight = value;
    redrawTree();
});

treeFolder.add(treeParams, 'trunkSize', 0.2, 2.0).name('Trunk Size').onChange((value: number) => {
    tree.set_trunk_size(value);
    treeParams.trunkSize = value;
    redrawTree();
});

// Add advanced tree generation controls
const advancedFolder = gui.addFolder('Advanced Parameters');

advancedFolder.add(treeParams, 'segmentLength', 0.1, 1.0).name('Segment Length').onChange((value: number) => {
    tree.set_segment_length(value);
    treeParams.segmentLength = value;
    redrawTree();
});

advancedFolder.add(treeParams, 'branchAngleMin', 5.0, 45.0).name('Min Branch Angle (°)').onChange((value: number) => {
    tree.set_branch_angle_range(value, treeParams.branchAngleMax);
    treeParams.branchAngleMin = value;
    redrawTree();
});

advancedFolder.add(treeParams, 'branchAngleMax', 15.0, 90.0).name('Max Branch Angle (°)').onChange((value: number) => {
    tree.set_branch_angle_range(treeParams.branchAngleMin, value);
    treeParams.branchAngleMax = value;
    redrawTree();
});

advancedFolder.add(treeParams, 'bendAngleMin', -45.0, 0.0).name('Min Bend Angle (°)').onChange((value: number) => {
    tree.set_bend_angle_range(value, treeParams.bendAngleMax);
    treeParams.bendAngleMin = value;
    redrawTree();
});

advancedFolder.add(treeParams, 'bendAngleMax', 0.0, 45.0).name('Max Bend Angle (°)').onChange((value: number) => {
    tree.set_bend_angle_range(treeParams.bendAngleMin, value);
    treeParams.bendAngleMax = value;
    redrawTree();
});

advancedFolder.add(treeParams, 'branchFrequencyMin', 1, 10).name('Min Branch Frequency').onChange((value: number) => {
    tree.set_branch_frequency_range(Math.floor(value), treeParams.branchFrequencyMax);
    treeParams.branchFrequencyMin = Math.floor(value);
    redrawTree();
});

advancedFolder.add(treeParams, 'branchFrequencyMax', 2, 15).name('Max Branch Frequency').onChange((value: number) => {
    tree.set_branch_frequency_range(treeParams.branchFrequencyMin, Math.floor(value));
    treeParams.branchFrequencyMax = Math.floor(value);
    redrawTree();
});

advancedFolder.add(treeParams, 'maxDepth', 1, 12).name('Max Depth').onChange((value: number) => {
    tree.set_max_depth(Math.floor(value));
    treeParams.maxDepth = Math.floor(value);
    redrawTree();
});

advancedFolder.add(treeParams, 'radiusTaper', 0.1, 0.8).name('Radius Taper').onChange((value: number) => {
    tree.set_radius_taper(value);
    treeParams.radiusTaper = value;
    redrawTree();
});

advancedFolder.add(treeParams, 'trunkRingSpread', 0.0, 5.0).name('Ring Spread').onChange((value: number) => {
    tree.set_trunk_ring_spread(value);
    treeParams.trunkRingSpread = value;
    redrawTree();
});

advancedFolder.add(treeParams, 'segmentLengthVariation', 0.0, 1.0).name('Segment Variation').onChange((value: number) => {
    tree.set_segment_length_variation(value);
    treeParams.segmentLengthVariation = value;
    redrawTree();
});

const wireframeControl = {
    wireframe: false
};

treeFolder.add(wireframeControl, 'wireframe').name('Wireframe Mode').onChange((value: boolean) => {
    ringMaterial.wireframe = value;
});
treeFolder.open();

// Add visualization controls
const visualFolder = gui.addFolder('Visualization');
const visualParams = {
    showRings: true,
    ringThickness: 0.05,
    color: '#8B4513'
};

visualFolder.add(visualParams, 'showRings').onChange((value: boolean) => {
    ringMeshes.forEach(mesh => {
        mesh.visible = value;
    });
});

visualFolder.add(visualParams, 'ringThickness', 0.01, 0.2).onChange((value: number) => {
    // Update ring thickness - would require recreating geometry
    redrawTree();
});

visualFolder.addColor(visualParams, 'color').onChange((value: string) => {
    const hexColor = parseInt(value.replace('#', '0x'));
    ringMaterial.color.setHex(hexColor);
});

visualFolder.open();

// Add root system controls
const rootFolder = gui.addFolder('Root System');

rootFolder.add(treeParams, 'rootEnable').name('Enable Roots').onChange((value: boolean) => {
    tree.set_root_enable(value);
    treeParams.rootEnable = value;
    redrawTree();
});

rootFolder.add(treeParams, 'rootDepth', 0.5, 3.0).name('Root Depth').onChange((value: number) => {
    tree.set_root_depth(value);
    treeParams.rootDepth = value;
    redrawTree();
});

rootFolder.add(treeParams, 'rootSpread', 0.5, 2.0).name('Root Spread').onChange((value: number) => {
    tree.set_root_spread(value);
    treeParams.rootSpread = value;
    redrawTree();
});

rootFolder.add(treeParams, 'rootDensity', 2, 8).step(1).name('Root Count').onChange((value: number) => {
    tree.set_root_density(Math.floor(value));
    treeParams.rootDensity = Math.floor(value);
    redrawTree();
});

rootFolder.add(treeParams, 'rootSegmentLength', 0.1, 0.8).name('Root Segment Length').onChange((value: number) => {
    tree.set_root_segment_length(value);
    treeParams.rootSegmentLength = value;
    redrawTree();
});

rootFolder.open();

// Add twig system controls
const twigFolder = gui.addFolder('Twig System');

twigFolder.add(treeParams, 'twigEnable').name('Enable Twigs').onChange((value: boolean) => {
    tree.set_twig_enable(value);
    treeParams.twigEnable = value;
    redrawTree();
});

twigFolder.add(treeParams, 'twigDensity', 0.1, 2.0).name('Twig Density').onChange((value: number) => {
    tree.set_twig_density(value);
    treeParams.twigDensity = value;
    redrawTree();
});

twigFolder.add(treeParams, 'twigScale', 0.1, 3.0).name('Twig Size').onChange((value: number) => {
    tree.set_twig_scale(value);
    treeParams.twigScale = value;
    redrawTree();
});

twigFolder.add(treeParams, 'twigAngleVariation', 0.0, 1.0).name('Angle Variation').onChange((value: number) => {
    tree.set_twig_angle_variation(value);
    treeParams.twigAngleVariation = value;
    redrawTree();
});

// Add twig library controls
if (twigLibrary) {
    const twigLibraryFolder = gui.addFolder('Twig Library');
    
    // Create options object for dropdown
    const twigOptions: { [key: string]: string } = {};
    twigLibrary.twigs.forEach(twig => {
        twigOptions[twig.name] = twig.id;
    });
    
    const twigLibraryParams = {
        selectedTwig: selectedTwigType
    };
    
    twigLibraryFolder.add(twigLibraryParams, 'selectedTwig', twigOptions).name('Twig Type').onChange((value: string) => {
        selectedTwigType = value;
        console.log(`Selected twig type: ${selectedTwigType}`);
        redrawTree();
    });
    
    // Add info display for selected twig
    const selectedTwigInfo = twigLibrary.twigs.find(t => t.id === selectedTwigType);
    if (selectedTwigInfo) {
        const infoParams = {
            description: selectedTwigInfo.description,
            type: selectedTwigInfo.type
        };
        
        twigLibraryFolder.add(infoParams, 'description').name('Description').listen();
        twigLibraryFolder.add(infoParams, 'type').name('Type').listen();
    }
    
    twigLibraryFolder.open();
}

twigFolder.open();

// Add bark texture controls
if (barkLibrary) {
    const barkFolder = gui.addFolder('Bark Textures');
    
    // Create options object for dropdown
    const barkOptions: { [key: string]: string } = {};
    barkLibrary.textures.forEach(texture => {
        barkOptions[texture.name] = texture.id;
    });
    
    const barkParams = {
        selectedBark: selectedBarkType
    };
    
    barkFolder.add(barkParams, 'selectedBark', barkOptions).name('Bark Type').onChange((value: string) => {
        selectedBarkType = value;
        console.log(`Selected bark type: ${selectedBarkType}`);
        updateBarkMaterial();
    });
    
    // Add info display for selected bark
    const selectedBarkInfo = barkLibrary.textures.find(t => t.id === selectedBarkType);
    if (selectedBarkInfo) {
        const barkInfoParams = {
            description: selectedBarkInfo.description,
            type: selectedBarkInfo.type
        };
        
        barkFolder.add(barkInfoParams, 'description').name('Description').listen();
        barkFolder.add(barkInfoParams, 'type').name('Type').listen();
    }
    
    barkFolder.open();
}

// Add export controls
const exportFolder = gui.addFolder('Export');

// Download GLTF function
const downloadGltf = () => {
    try {
        console.log('Exporting GLTF...');
        const gltfJson = tree.export_gltf(16); // Use same resolution as mesh generation
        console.log('GLTF exported successfully');
        
        // Create blob and download
        const blob = new Blob([gltfJson], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        
        // Create download link
        const link = document.createElement('a');
        link.href = url;
        link.download = `tree_${Date.now()}.gltf`;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        
        // Clean up object URL
        URL.revokeObjectURL(url);
        
        console.log('GLTF file downloaded');
    } catch (error) {
        console.error('Error exporting GLTF:', error);
        alert('Failed to export GLTF file. Check console for details.');
    }
};

const exportParams = {
    downloadGltf: downloadGltf
};

exportFolder.add(exportParams, 'downloadGltf').name('Download GLTF');
exportFolder.open();

// Animation loop
function animate() {
    requestAnimationFrame(animate);
    
    // Update controls for smooth damping
    controls.update();
    
    // Vary the twig rotation slightly to simulate wind. Use sinuoidal function to create a more natural effect.
    twigMeshes.forEach(mesh => {
        // TODO
    })

    renderer.render(scene, camera);
}

// Handle window resize
window.addEventListener('resize', () => {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
});

// Start the animation
animate();