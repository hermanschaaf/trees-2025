#[derive(Debug, Clone)]
pub struct TreeParameters {
    pub general: GeneralParams,
    pub trunk: TrunkParams,
    pub branching: BranchingParams,
    pub roots: RootParams,
    pub twigs: TwigParams,
}

#[derive(Debug, Clone)]
pub struct GeneralParams {
    pub seed: u32,
    pub max_depth: u32,
}

#[derive(Debug, Clone)]
pub struct TrunkParams {
    pub height: f32,
    pub buttressing: f32,
    pub split_height: f32,
    pub segment_length: f32,
    pub size: f32,
    pub ring_spread: f32,
    pub segment_length_variation: f32,
}

#[derive(Debug, Clone)]
pub struct BranchingParams {
    pub angle_min: f32,
    pub angle_max: f32,
    pub bend_angle_min: f32,
    pub bend_angle_max: f32,
    pub frequency_min: u32,
    pub frequency_max: u32,
    pub radius_taper: f32,
    pub azimuth_variation: f32,
    pub max_reach: f32,
}

#[derive(Debug, Clone)]
pub struct RootParams {
    pub enable: bool,
    pub depth: f32,
    pub spread: f32,
    pub density: u32,
    pub segment_length: f32,
}

#[derive(Debug, Clone)]
pub struct TwigParams {
    pub enable: bool,
    pub density: f32,
    pub scale: f32,
    pub angle_variation: f32,
}

impl Default for TreeParameters {
    fn default() -> Self {
        TreeParameters {
            general: GeneralParams::default(),
            trunk: TrunkParams::default(),
            branching: BranchingParams::default(),
            roots: RootParams::default(),
            twigs: TwigParams::default(),
        }
    }
}

impl Default for GeneralParams {
    fn default() -> Self {
        GeneralParams {
            seed: 123,
            max_depth: 20,
        }
    }
}

impl Default for TrunkParams {
    fn default() -> Self {
        TrunkParams {
            height: 5.0,
            buttressing: 1.0,
            split_height: 3.0, // 60% of default height
            segment_length: 0.3,
            size: 3.0,
            ring_spread: 0.5,
            segment_length_variation: 0.3,
        }
    }
}

impl Default for BranchingParams {
    fn default() -> Self {
        BranchingParams {
            angle_min: 25.0,
            angle_max: 45.0,
            bend_angle_min: -15.0,
            bend_angle_max: 15.0,
            frequency_min: 2,
            frequency_max: 4,
            radius_taper: 0.8,
            azimuth_variation: 0.5,
            max_reach: 50.0,
        }
    }
}

impl Default for RootParams {
    fn default() -> Self {
        RootParams {
            enable: true,
            depth: 3.0,
            spread: 2.5,
            density: 6,
            segment_length: 0.25,
        }
    }
}

impl Default for TwigParams {
    fn default() -> Self {
        TwigParams {
            enable: true,
            density: 1.0,
            scale: 1.0,
            angle_variation: 0.5,
        }
    }
}

// Legacy compatibility layer removed - no longer needed after refactoring