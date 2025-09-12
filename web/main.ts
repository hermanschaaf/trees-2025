import * as THREE from 'three';
import * as dat from 'dat.gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';

import init, { generate } from "/static/js/tree_rs.js";

await init();
const tree = generate(123, 1, 2.0);
console.log("Rust generate:", tree);
console.log("trunk_height: ", tree.trunk_height);
console.log("buttressing: ", tree.butressing);

// Scene setup
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Tree parameters
const treeParams = {
    height: tree.trunk_height,
    butressing: tree.butressing,
    radius: 0.5,
    radialSegments: 32
};

let cylinders: THREE.Mesh[] = [];

// Create a vertical cylinder pointing upwards from origin
let geometry = new THREE.CylinderGeometry(
    treeParams.radius, 
    treeParams.radius * treeParams.butressing, 
    treeParams.height, 
    treeParams.radialSegments
);
const material = new THREE.MeshBasicMaterial({ color: 0xffff00, wireframe: false });
const cylinder = new THREE.Mesh(geometry, material);
cylinders.push(cylinder);

// Position cylinder so it starts at origin and extends upward
cylinder.position.y = treeParams.height / 2;
scene.add(cylinder);

// Add some lighting to better see the cylinder
const ambientLight = new THREE.AmbientLight(0x404040, 0.5);
scene.add(ambientLight);
const directionalLight = new THREE.DirectionalLight(0xffffff, 0.3);
directionalLight.position.set(5, 5, 5);
scene.add(directionalLight);

// Position the camera
camera.position.set(3, 2, 5);
camera.lookAt(0, treeParams.height / 2, 0);

// Add orbit controls for mouse/trackpad interaction
const controls = new OrbitControls(camera, renderer.domElement);
controls.target.set(0, treeParams.height / 2, 0);
controls.enableDamping = true;
controls.dampingFactor = 0.05;

// GUI Controls
const gui = new dat.GUI();
const cylinderFolder = gui.addFolder('Trunk');

const redrawTree = () => {
    for (let cylinder of cylinders) {
        scene.remove(cylinder);
    }
    cylinders = [];
    
    // Create new geometry with updated height
    geometry.dispose();
    const branches = tree.branches();
    for (let branch of branches) {
        geometry = new THREE.CylinderGeometry(
            branch.start_radius,
            branch.end_radius,
            branch.length,
            treeParams.radialSegments
        );
        const cylinder = new THREE.Mesh(geometry, material);
        cylinders.push(cylinder);
        cylinder.position.set(branch.start.x, branch.start.y, branch.start.z);
        cylinder.lookAt(branch.end.x, branch.end.y, branch.end.z);
    }
    console.log(branches);

    for (let cylinder of cylinders) {
        scene.add(cylinder);
    }
}

cylinderFolder.add(treeParams, 'height', 0.1, 10).onChange((value: number) => {
    tree.set_trunk_height(value);
    tree.render();
    treeParams.height = value;

    redrawTree()

    // Update controls target to center of cylinder
    controls.target.set(0, value / 2, 0);
});

cylinderFolder.add(treeParams, 'butressing', 0.1, 10).onChange((value: number) => {
    tree.set_butressing(value);
    treeParams.butressing = value;
    redrawTree()

    // Update controls target to center of cylinder
    controls.target.set(0, value / 2, 0);
});

cylinderFolder.add(treeParams, 'radius', 0.1, 2).onChange((value: number) => {
    treeParams.radius = value;
    redrawTree()
});

cylinderFolder.add(material, 'wireframe');
cylinderFolder.open();

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