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
    radius: 0.5,
    radialSegments: 32
};

let ringMeshes: THREE.Mesh[] = [];

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

// Function to create ring visualization
const createRingMesh = (center: {x: number, y: number, z: number}, radius: number) => {
    const geometry = new THREE.TorusGeometry(radius, 0.05, 8, 16);
    const mesh = new THREE.Mesh(geometry, ringMaterial);
    mesh.position.set(center.x, center.y, center.z);
    // Rotate the torus to be vertical (perpendicular to Y axis)
    mesh.rotation.x = Math.PI / 2;
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    return mesh;
};

// Create rings from TreeObject
const createTreeVisualization = () => {
    // Clear existing meshes
    ringMeshes.forEach(mesh => {
        scene.remove(mesh);
        mesh.geometry.dispose();
    });
    ringMeshes = [];
    
    // Create ring meshes
    const ringCount = tree.rings_count();
    for (let i = 0; i < ringCount; i++) {
        const center = tree.ring_center(i);
        const radius = tree.ring_radius(i);
        
        if (center && radius !== null) {
            const ringMesh = createRingMesh(center, radius);
            ringMeshes.push(ringMesh);
            scene.add(ringMesh);
        }
    }
    
    console.log(`Created ${ringMeshes.length} ring meshes`);
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
const gui = new dat.GUI();
const treeFolder = gui.addFolder('Tree Parameters');

const redrawTree = () => {
    createTreeVisualization();
}

treeFolder.add(treeParams, 'height', 0.1, 10).onChange((value: number) => {
    tree.set_trunk_height(value);
    treeParams.height = value;
    redrawTree();
    // Update controls target to center of tree
    controls.target.set(0, value / 2, 0);
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