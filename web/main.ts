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

// Cylinder parameters
const cylinderParams = {
    height: tree.trunk_height,
    butressing: tree.butressing,
    radius: 0.5,
    radialSegments: 32
};

// Create a vertical cylinder pointing upwards from origin
let geometry = new THREE.CylinderGeometry(
    cylinderParams.radius, 
    cylinderParams.radius * cylinderParams.butressing, 
    cylinderParams.height, 
    cylinderParams.radialSegments
);
const material = new THREE.MeshBasicMaterial({ color: 0xffff00, wireframe: false });
const cylinder = new THREE.Mesh(geometry, material);

// Position cylinder so it starts at origin and extends upward
cylinder.position.y = cylinderParams.height / 2;
scene.add(cylinder);

// Add some lighting to better see the cylinder
const ambientLight = new THREE.AmbientLight(0x404040, 0.5);
scene.add(ambientLight);
const directionalLight = new THREE.DirectionalLight(0xffffff, 0.3);
directionalLight.position.set(5, 5, 5);
scene.add(directionalLight);

// Position the camera
camera.position.set(3, 2, 5);
camera.lookAt(0, cylinderParams.height / 2, 0);

// Add orbit controls for mouse/trackpad interaction
const controls = new OrbitControls(camera, renderer.domElement);
controls.target.set(0, cylinderParams.height / 2, 0);
controls.enableDamping = true;
controls.dampingFactor = 0.05;

// GUI Controls
const gui = new dat.GUI();
const cylinderFolder = gui.addFolder('Trunk');

const redrawCylinder = () => {
    // Create new geometry with updated height
    geometry.dispose();
    geometry = new THREE.CylinderGeometry(
        cylinderParams.radius,
        cylinderParams.radius * cylinderParams.butressing,
        cylinderParams.height,
        cylinderParams.radialSegments
    );

    // Update the mesh
    cylinder.geometry = geometry;
    cylinder.position.y = cylinderParams.height / 2;

    // Add back to scene
    scene.add(cylinder);
}

cylinderFolder.add(cylinderParams, 'height', 0.1, 10).onChange((value: number) => {
    // Remove old cylinder
    scene.remove(cylinder);
    
    tree.set_trunk_height(value);
    cylinderParams.height = value;

    redrawCylinder()

    // Update controls target to center of cylinder
    controls.target.set(0, value / 2, 0);
});

cylinderFolder.add(cylinderParams, 'butressing', 0.1, 10).onChange((value: number) => {
    // Remove old cylinder
    scene.remove(cylinder);
    
    tree.set_butressing(value);
    cylinderParams.butressing = value;
    redrawCylinder()

    // Update controls target to center of cylinder
    controls.target.set(0, value / 2, 0);
});

cylinderFolder.add(cylinderParams, 'radius', 0.1, 2).onChange((value: number) => {
    // Remove old cylinder
    scene.remove(cylinder);
    
    cylinderParams.radius = value;
    redrawCylinder()

    // Add back to scene
    scene.add(cylinder);
});

cylinderFolder.add(material, 'wireframe');
cylinderFolder.open();

// Animation loop
function animate() {
    requestAnimationFrame(animate);
    
    // Update controls for smooth damping
    controls.update();
    
    // Optional: slowly rotate the cylinder to see it better
    cylinder.rotation.y += 0.005;
    
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