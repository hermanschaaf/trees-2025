use wasm_bindgen::prelude::*;

// ----- Quaternion -----

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Quaternion {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}

#[wasm_bindgen]
impl Quaternion {
    #[wasm_bindgen(constructor)]
    pub fn new(w: f32, x: f32, y: f32, z: f32) -> Quaternion {
        Quaternion { w, x, y, z }
    }

    #[wasm_bindgen(getter)] pub fn w(&self) -> f32 { self.w }
    #[wasm_bindgen(getter)] pub fn x(&self) -> f32 { self.x }
    #[wasm_bindgen(getter)] pub fn y(&self) -> f32 { self.y }
    #[wasm_bindgen(getter)] pub fn z(&self) -> f32 { self.z }
}

// ----- Vector3d -----

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Vector3d {
    x: f32,
    y: f32,
    z: f32,
}

#[wasm_bindgen]
impl Vector3d {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f32, y: f32, z: f32) -> Vector3d {
        Vector3d { x, y, z }
    }

    #[wasm_bindgen(getter)] pub fn x(&self) -> f32 { self.x }
    #[wasm_bindgen(getter)] pub fn y(&self) -> f32 { self.y }
    #[wasm_bindgen(getter)] pub fn z(&self) -> f32 { self.z }
}
