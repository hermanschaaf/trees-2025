import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';

let animationId: number | null = null;

export const initializeSimpleTreeViewer = async (container: HTMLElement, initialTreeType: 'fir' | 'birch' | 'ivy' = 'fir'): Promise<{ cleanup: () => void; downloadGltf: () => void; regenerateScene: (newTreeType?: 'fir' | 'birch' | 'ivy') => void } | void> => {
  let currentTreeType = initialTreeType;
  // Import WASM module
  const { default: init, generate } = await eval(`import("/static/js/tree_rs.js")`);
  
  await init();
  const tree = generate(123, 5.0, 1.0);
  console.log("Rust generate:", tree);

  // Scene setup with simple gradient background
  const scene = new THREE.Scene();
  
  // Simple gradient background
  scene.background = new THREE.Color(0x87CEEB); // Sky blue background
  
  // Camera setup
  const camera = new THREE.PerspectiveCamera(75, container.clientWidth / container.clientHeight, 0.1, 1000);
  camera.position.set(5, 5, 10);
  
  // Renderer setup
  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(container.clientWidth, container.clientHeight);
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  container.appendChild(renderer.domElement);
  
  // Controls
  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.dampingFactor = 0.05;
  
  // Enhanced lighting system
  // Ambient light for overall illumination (like sky light)
  const ambientLight = new THREE.AmbientLight(0x87CEEB, 0.4);
  scene.add(ambientLight);
  
  // Main sun light
  const sunLight = new THREE.DirectionalLight(0xffffff, 1.0);
  sunLight.position.set(50, 100, 50); // High in the sky
  sunLight.castShadow = true;
  sunLight.shadow.mapSize.width = 4096;
  sunLight.shadow.mapSize.height = 4096;
  sunLight.shadow.camera.near = 0.5;
  sunLight.shadow.camera.far = 500;
  sunLight.shadow.camera.left = -50;
  sunLight.shadow.camera.right = 50;
  sunLight.shadow.camera.top = 50;
  sunLight.shadow.camera.bottom = -50;
  sunLight.shadow.bias = -0.0001;
  scene.add(sunLight);
  
  // Secondary fill light (bounce light simulation)
  const fillLight = new THREE.DirectionalLight(0x87CEEB, 0.3);
  fillLight.position.set(-30, 20, -30);
  scene.add(fillLight);
  
  // Warm rim light for depth
  const rimLight = new THREE.DirectionalLight(0xfff4e6, 0.2);
  rimLight.position.set(-10, 5, 10);
  scene.add(rimLight);
  
  // Ground plane with better material
  const groundGeometry = new THREE.PlaneGeometry(100, 100, 10, 10);
  const groundMaterial = new THREE.MeshLambertMaterial({ 
    color: 0xffffff, // White
    transparent: true,
    opacity: 0.8
  });
  const ground = new THREE.Mesh(groundGeometry, groundMaterial);
  ground.rotation.x = -Math.PI / 2;
  ground.receiveShadow = true;
  ground.position.y = -0.1; // Slightly below origin
  scene.add(ground);

  // Load ivy leaf model
  const loader = new GLTFLoader();
  let ivyLeafModel: THREE.Object3D | null = null;
  
  const loadIvyLeaf = async () => {
    try {
      const gltf = await new Promise<any>((resolve, reject) => {
        loader.load('/static/leaves/ivy/leaf.glb', resolve, undefined, reject);
      });
      ivyLeafModel = gltf.scene.clone();
      console.log('Ivy leaf model loaded successfully');
      console.log('Model structure:', ivyLeafModel);
      console.log('Model children:', ivyLeafModel.children);
      console.log('Model bounding box:');
      
      // Calculate bounding box to understand scale
      const box = new THREE.Box3().setFromObject(ivyLeafModel);
      console.log('Bounding box size:', box.getSize(new THREE.Vector3()));
      
      // Ensure materials are set up properly
      ivyLeafModel.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          console.log('Found mesh:', child.name, 'geometry:', child.geometry, 'material:', child.material);
          child.castShadow = true;
          child.receiveShadow = true;
          
          // Ensure material is visible
          if (child.material) {
            if (Array.isArray(child.material)) {
              child.material.forEach(mat => {
                mat.transparent = false;
                mat.opacity = 1.0;
              });
            } else {
              child.material.transparent = false;
              child.material.opacity = 1.0;
            }
          }
        }
      });
    } catch (error) {
      console.error('Failed to load ivy leaf model:', error);
    }
  };

  // Generate and display tree or ivy scene based on type
  const generateScene = (newTreeType?: 'fir' | 'birch' | 'ivy') => {
    if (newTreeType) {
      currentTreeType = newTreeType;
    }
    console.log('Generating scene for tree type:', currentTreeType);
    
    // Remove existing tree and any ivy leaves
    const existingTree = scene.getObjectByName('tree');
    if (existingTree) {
      scene.remove(existingTree);
    }
    
    // Remove any existing ivy leaves and debug cubes that were added directly to scene
    const objectsToRemove: THREE.Object3D[] = [];
    scene.traverse((child) => {
      if (child.name === 'ivyLeaf' || child.name === 'debugCube') {
        objectsToRemove.push(child);
      }
    });
    objectsToRemove.forEach(obj => scene.remove(obj));

    const treeGroup = new THREE.Group();
    treeGroup.name = 'tree';

    if (currentTreeType === 'ivy') {
      console.log('Creating ivy scene, ivyLeafModel:', ivyLeafModel);
      console.log('ivyLeafModel exists?', !!ivyLeafModel);
      // Create ivy scene - just display a single leaf like the test that worked
      if (ivyLeafModel) {
        const testLeaf = ivyLeafModel.clone();
        testLeaf.name = 'ivyLeaf';
        testLeaf.position.set(0, 5, 5); // Right in front of camera (camera is at 5,5,10)
        testLeaf.scale.setScalar(1.0); // Normal size first
        testLeaf.rotation.set(0, 0, 0); // No rotation
        testLeaf.visible = true;
        
        // Check bounding box to see actual size
        const box = new THREE.Box3().setFromObject(testLeaf);
        const size = box.getSize(new THREE.Vector3());
        console.log('Leaf bounding box size:', size);
        console.log('Leaf position after scaling:', testLeaf.position);
        
        // Force material to be visible
        testLeaf.traverse((child) => {
          if (child instanceof THREE.Mesh) {
            console.log('Leaf mesh found:', child);
            if (child.material) {
              if (Array.isArray(child.material)) {
                child.material.forEach(mat => {
                  mat.transparent = false;
                  mat.opacity = 1.0;
                  mat.side = THREE.DoubleSide; // Visible from both sides
                });
              } else {
                child.material.transparent = false;
                child.material.opacity = 1.0;
                child.material.side = THREE.DoubleSide;
              }
            }
          }
        });
        
        scene.add(testLeaf);
        console.log('Ivy leaf added to scene');
      } else {
        console.warn('ivyLeafModel is null, cannot create ivy scene');
        console.log('Attempting to load ivy model again...');
        loadIvyLeaf().then(() => {
          if (ivyLeafModel) {
            console.log('Ivy model loaded on retry, adding leaf');
            const testLeaf = ivyLeafModel.clone();
            testLeaf.name = 'ivyLeaf';
            testLeaf.position.set(0, 1, 0);
            testLeaf.scale.setScalar(2.0);
            testLeaf.visible = true;
            scene.add(testLeaf);
          }
        });
      }
    } else {
      // Generate regular tree for fir and birch (for now, same tree)
      const newTree = generate(123, 5.0, 1.0);
      
      if (newTree && newTree.rings) {
        newTree.rings.forEach((ring: any, index: number) => {
          const radius = ring.radius || 0.1;
          const height = 0.2;
          
          const geometry = new THREE.CylinderGeometry(radius, radius * 0.8, height, 8);
          const material = new THREE.MeshLambertMaterial({ 
            color: new THREE.Color().setHSL(0.1, 0.7, 0.3 + Math.random() * 0.2)
          });
          
          const segment = new THREE.Mesh(geometry, material);
          segment.position.set(ring.center?.x || 0, ring.center?.y || index * 0.2, ring.center?.z || 0);
          segment.castShadow = true;
          
          treeGroup.add(segment);
        });
      }
      
      // Add green debug cube only for Fir and Birch scenes
      const debugCube = new THREE.Mesh(
        new THREE.BoxGeometry(2, 2, 2),
        new THREE.MeshLambertMaterial({ color: 0x00ff00 })
      );
      debugCube.position.set(-3, 1, 0);
      debugCube.name = 'debugCube'; // So it gets removed when switching scenes
      scene.add(debugCube);
    }
    
    scene.add(treeGroup);
  };

  // Load ivy leaf model first, then generate scene
  await loadIvyLeaf();
  generateScene();
  
  // Resize handler
  const handleResize = () => {
    camera.aspect = container.clientWidth / container.clientHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(container.clientWidth, container.clientHeight);
  };
  
  window.addEventListener('resize', handleResize);
  
  // Animation loop
  const animate = () => {
    animationId = requestAnimationFrame(animate);
    controls.update();
    renderer.render(scene, camera);
  };
  
  animate();
  
  // GLTF export function
  const downloadGltf = () => {
    import('three/examples/jsm/exporters/GLTFExporter.js').then(({ GLTFExporter }) => {
      const exporter = new GLTFExporter();
      const treeObject = scene.getObjectByName('tree');
      
      if (treeObject) {
        exporter.parse(
          treeObject,
          (gltf) => {
            const blob = new Blob([JSON.stringify(gltf)], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const link = document.createElement('a');
            link.href = url;
            link.download = 'tree.gltf';
            link.click();
            URL.revokeObjectURL(url);
          },
          (error) => {
            console.error('GLTF export error:', error);
          }
        );
      }
    });
  };
  
  // Cleanup function
  const cleanup = () => {
    if (animationId) {
      cancelAnimationFrame(animationId);
      animationId = null;
    }
    
    window.removeEventListener('resize', handleResize);
    
    if (container && renderer.domElement) {
      container.removeChild(renderer.domElement);
    }
    
    controls.dispose();
    renderer.dispose();
    
    // Clean up scene
    scene.traverse((object) => {
      if (object instanceof THREE.Mesh) {
        object.geometry.dispose();
        if (Array.isArray(object.material)) {
          object.material.forEach(material => material.dispose());
        } else {
          object.material.dispose();
        }
      }
    });
  };
  
  return { cleanup, downloadGltf, regenerateScene: generateScene };
};