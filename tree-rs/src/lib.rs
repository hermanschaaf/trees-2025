use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;

mod wasm_math;
mod tree;

#[wasm_bindgen]
pub struct TreeObject {
    pub seed: u32,
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
    pub fn new(seed: u32, segment_params: JsValue, straightness_params: JsValue) -> Result<TreeObject, JsValue> {
        let segment_params: DistributionParams = from_value(segment_params)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse segment_params: {}", e)))?;
        let straightness_params: DistributionParams = from_value(straightness_params)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse straightness_params: {}", e)))?;

        let segment_dist = convert_to_distribution(segment_params)?;
        let straightness_dist = convert_to_distribution(straightness_params)?;

        let tree = tree::Tree::new(seed, segment_dist, straightness_dist);
        Ok(TreeObject { seed: tree.seed, tree })
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

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
interface DistributionParams {
    dist_type: 'normal' | 'uniform' | 'poisson';
    loc: number;
    scale: number;
}
"#;

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct DistributionParams {
    dist_type: String,
    loc: f32,
    scale: f32,
}

#[wasm_bindgen]
impl DistributionParams {
    #[wasm_bindgen(constructor)]
    pub fn new(dist_type: String, loc: f32, scale: f32) -> Self {
        Self { dist_type, loc, scale }
    }
}

// Public API: generate a Tree
#[wasm_bindgen]
pub fn generate(seed: u32, segment_params: JsValue, straightness_params: JsValue) -> Result<TreeObject, JsValue> {
    let segment_params: DistributionParams = from_value(segment_params)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse segment_params: {}", e)))?;
    let straightness_params: DistributionParams = from_value(straightness_params)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse straightness_params: {}", e)))?;

    let segment_dist = convert_to_distribution(segment_params)?;
    let straightness_dist = convert_to_distribution(straightness_params)?;

    let tree = tree::Tree::new(seed, segment_dist, straightness_dist);
    Ok(TreeObject { seed: tree.seed, tree })
}

fn convert_to_distribution(params: DistributionParams) -> Result<tree::Distribution, JsValue> {
    use tree::DistributionFamily;
    
    let family = match params.dist_type.as_str() {
        "normal" => DistributionFamily::Normal,
        "uniform" => DistributionFamily::Uniform,
        "poisson" => DistributionFamily::Poisson,
        _ => return Err(JsValue::from_str(&format!("Unknown distribution type: {}", params.dist_type))),
    };
    
    Ok(tree::Distribution {
        family,
        location: params.loc,
        scale: params.scale,
    })
}


#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works() {
        // Create test distribution parameters
        let segment_params = DistributionParams {
            dist_type: "normal".to_string(),
            loc: 0.3,
            scale: 0.1,
        };
        let straightness_params = DistributionParams {
            dist_type: "normal".to_string(),
            loc: 2.0,
            scale: 0.5,
        };
        
        // Convert to JsValue for the test
        let segment_js = JsValue::from_serde(&segment_params).unwrap();
        let straightness_js = JsValue::from_serde(&straightness_params).unwrap();
        
        let t = generate(123, segment_js, straightness_js).unwrap();
        assert_eq!(t.seed, 123);
    }
}
