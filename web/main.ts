import * as THREE from 'three';
import * as dat from 'dat.gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';

import init, { generate } from "/static/js/tree_rs.js";

await init();
const tree = generate(123, 5.0, 1.0);
console.log("Rust generate:", tree);
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
    twigScale: tree.twig_scale,
    twigAngleVariation: tree.twig_angle_variation
};

let ringMeshes: THREE.Mesh[] = [];
let twigMeshes: THREE.Mesh[] = [];

// Create materials for rings
const ringMaterial = new THREE.MeshPhongMaterial({ 
    color: 0x8B4513, // Brown color
    shininess: 30,
    specular: 0x222222
});
const cylinderMaterial = new THREE.MeshPhongMaterial({ 
    color: 0x654321, // Darker brown for branches
    shininess: 20,
    specular: 0x111111
});

// Create materials for twigs
const leafMaterial = new THREE.MeshLambertMaterial({ 
    color: 0x228B22, // Forest green
    side: THREE.DoubleSide // Show both sides of leaves
});
const twigBranchMaterial = new THREE.MeshPhongMaterial({ 
    color: 0x8B4513, // Brown like main branches
    shininess: 10
});
const budMaterial = new THREE.MeshPhongMaterial({ 
    color: 0x90EE90, // Light green for buds
    shininess: 15
});

// Create twig geometries for different twig types
const createTwigGeometry = (twigType: string, scale: number) => {
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

// Create tree visualization using generated mesh
const createTreeVisualization = () => {
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
        console.log("No mesh data generated - falling back to simple visualization");

        // Fallback: Create simple ring visualization
        const ringCount = tree.rings_count();
        for (let i = 0; i < ringCount; i++) {
            const center = tree.ring_center(i);
            const radius = tree.ring_radius(i);

            if (center && radius !== null) {
                const geometry = new THREE.TorusGeometry(radius, 0.05, 8, 16);
                const mesh = new THREE.Mesh(geometry, ringMaterial);
                mesh.position.set(center.x, center.y, center.z);
                mesh.rotation.x = Math.PI / 2;
                mesh.castShadow = true;
                mesh.receiveShadow = true;
                ringMeshes.push(mesh);
                scene.add(mesh);
            }
        }
        console.log(`Created ${ringMeshes.length} fallback ring meshes`);
    }
    
    // Generate twigs if enabled
    if (treeParams.twigEnable) {
        const twigCount = tree.twigs_count();
        console.log(`Generating ${twigCount} twigs`);
        
        for (let i = 0; i < twigCount; i++) {
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
                
                // Create twig geometry
                const twigGeometry = createTwigGeometry(twigType, scale);
                
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
        }
        
        console.log(`Created ${twigMeshes.length} twig instances`);
    }
};

// Create initial tree visualization
createTreeVisualization();

// Improved lighting setup
const ambientLight = new THREE.AmbientLight(0x404040, 0.4); // Soft ambient light
scene.add(ambientLight);

// Main directional light (sun)
const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
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
camera.position.set(6, 4, 8);
camera.lookAt(0, treeParams.height / 2, 0);

// Add orbit controls for mouse/trackpad interaction
const controls = new OrbitControls(camera, renderer.domElement);
controls.target.set(0, treeParams.height / 2, 0);
controls.enableDamping = true;
controls.dampingFactor = 0.05;

// GUI Controls
const gui = new dat.GUI({ width: 320 });
const treeFolder = gui.addFolder('Tree Parameters');

const redrawTree = () => {
    createTreeVisualization();
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

advancedFolder.add(treeParams, 'radiusTaper', 0.1, 1.0).name('Radius Taper').onChange((value: number) => {
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

twigFolder.add(treeParams, 'twigScale', 0.5, 3.0).name('Twig Size').onChange((value: number) => {
    tree.set_twig_scale(value);
    treeParams.twigScale = value;
    redrawTree();
});

twigFolder.add(treeParams, 'twigAngleVariation', 0.0, 1.0).name('Angle Variation').onChange((value: number) => {
    tree.set_twig_angle_variation(value);
    treeParams.twigAngleVariation = value;
    redrawTree();
});

twigFolder.open();

// Animation loop
function animate() {
    requestAnimationFrame(animate);
    
    // Update controls for smooth damping
    controls.update();
    
    // Optional: slowly rotate the cylinder to see it better
    // cylinder.rotation.y += 0.005;
    
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