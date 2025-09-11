use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

mod wasm_math;
mod tree;

#[wasm_bindgen]
pub struct TreeObject {
    pub seed: u32,
    pub trunk_height: f32,
    pub butressing: f32,
    tree: tree::Tree,
}

#[wasm_bindgen]
pub struct Branch {
    pub length: f32,
    pub radius: f32,
    pub depth: u32,
    pub direction: wasm_math::Quaternion,
    pub start: wasm_math::Vector3d,
    pub end: wasm_math::Vector3d,
}

#[wasm_bindgen]
impl TreeObject {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, trunk_height: f32, butressing: f32) -> Result<TreeObject, JsValue> {
        let tree = tree::Tree::new(seed, trunk_height, butressing);
        Ok(TreeObject { seed: tree.seed, trunk_height, butressing, tree })
    }

    pub fn set_trunk_height(&mut self, height: f32) {
        self.tree.trunk_height = height;
        self.trunk_height = height;
    }

    pub fn set_butressing(&mut self, butressing: f32) {
        self.tree.butressing = butressing;
        self.butressing = butressing;
    }

    pub fn grow(&mut self) {
        self.tree.trunk_height += 1.0;
        self.butressing += 1.0;
        // self.tree.grow();
    }

    pub fn branches(&mut self) -> Vec<Branch> {
        // Precompute starts and ends first (no borrow conflicts)
        let mut starts = Vec::with_capacity(self.tree.branches.len());
        let mut ends = Vec::with_capacity(self.tree.branches.len());
        for idx in 0..self.tree.branches.len() {
            starts.push(self.tree.branch_start(idx));
            ends.push(self.tree.branch_end(idx)); // needs &mut self
        }
    
        // Now iterate immutably
        self.tree.branches.iter()
            .enumerate()
            .filter(|(_, branch)| !branch.pruned)
            .map(|(idx, branch)| {
                let start = starts[idx];
                let end = ends[idx];
                Branch {
                    depth: branch.depth,
                    length: branch.length,
                    radius: branch.radius,
                    direction: wasm_math::Quaternion::new(
                        branch.direction.w(),
                        branch.direction.x(),
                        branch.direction.y(),
                        branch.direction.z(),
                    ),
                    start: wasm_math::Vector3d::new(start.x, start.y, start.z),
                    end: wasm_math::Vector3d::new(end.x, end.y, end.z),
                }
            })
            .collect()
    }    
}

// Public API: generate a Tree
#[wasm_bindgen]
pub fn generate(seed: u32, trunk_height: f32, butressing: f32) -> Result<TreeObject, JsValue> {
    let tree = tree::Tree::new(seed, trunk_height, butressing);
    Ok(TreeObject { seed: tree.seed, trunk_height, butressing, tree })
}

