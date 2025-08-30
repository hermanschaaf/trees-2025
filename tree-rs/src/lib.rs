use wasm_bindgen::prelude::*;

mod wasm_math;
mod tree;

#[wasm_bindgen]
pub struct TreeObject {
    pub seed: u32,
    pub age: f32,
    tree: tree::Tree,
}

#[wasm_bindgen]
pub struct Branch {
    pub length: f32,
    pub radius: f32,
    pub direction: wasm_math::Quaternion,
    pub start: wasm_math::Vector3d,
    pub end: wasm_math::Vector3d,
}

#[wasm_bindgen]
impl TreeObject {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, age: f32) -> TreeObject {
        TreeObject { seed, age, tree: tree::Tree::new(seed, age) }
    }

    pub fn grow(&mut self) {
        self.tree.grow(5.0);
    }

    pub fn branches(&self) -> Vec<Branch> {
        self.tree.branches.iter().map(|branch| {
            let start = self.tree.branch_start(branch.index);
            let end = self.tree.branch_end(branch.index);
            Branch {
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
        }).collect()
    }
}

// Public API: generate a Tree
#[wasm_bindgen]
pub fn generate(seed: u32, age: f32) -> TreeObject {
    let tree = tree::Tree::new(seed, age);
    TreeObject { seed: tree.seed, age: tree.age, tree }
}


#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works() {
        let t = generate(123, 1.0);
        assert_eq!(t.seed, 123);
        assert_eq!(t.age, 1.0);
    }
}
