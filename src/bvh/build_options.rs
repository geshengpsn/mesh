#[derive(Default)]
pub struct BuildBvhOption {
    pub depth_control: DepthControl,
    pub split_method: SplitMethod,
}

#[derive(Clone, Copy)]
pub enum DepthControl {
    MaxDepth(usize),
    MinPrimitives(usize),
}

impl Default for DepthControl {
    fn default() -> Self {
        DepthControl::MaxDepth(20)
    }
}

#[derive(Clone, Copy, Default)]
pub enum SplitMethod {
    #[default]
    Mid,
    Average,
    // SAH,
}
