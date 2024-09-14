#[derive(Clone, Copy, Debug)]
pub enum BlendOp {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}
impl BlendOp {
    pub fn to_gl_enum(&self) -> u32 {
        match self {
            BlendOp::Add => gl::FUNC_ADD,
            BlendOp::Subtract => gl::FUNC_SUBTRACT,
            BlendOp::ReverseSubtract => gl::FUNC_REVERSE_SUBTRACT,
            BlendOp::Min => gl::MIN,
            BlendOp::Max => gl::MAX,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
    SrcAlphaSaturate,
    Src1Color,
    OneMinusSrc1Color,
    Src1Alpha,
    OneMinusSrc1Alpha,
}
impl BlendFactor {
    pub fn to_gl_enum(&self) -> u32 {
        match self {
            BlendFactor::Zero => gl::ZERO,
            BlendFactor::One => gl::ONE,
            BlendFactor::SrcColor => gl::SRC_COLOR,
            BlendFactor::OneMinusSrcColor => gl::ONE_MINUS_SRC_COLOR,
            BlendFactor::DstColor => gl::DST_COLOR,
            BlendFactor::OneMinusDstColor => gl::ONE_MINUS_DST_COLOR,
            BlendFactor::SrcAlpha => gl::SRC_ALPHA,
            BlendFactor::OneMinusSrcAlpha => gl::ONE_MINUS_SRC_ALPHA,
            BlendFactor::DstAlpha => gl::DST_ALPHA,
            BlendFactor::OneMinusDstAlpha => gl::ONE_MINUS_DST_ALPHA,
            BlendFactor::ConstantColor => gl::CONSTANT_COLOR,
            BlendFactor::OneMinusConstantColor => gl::ONE_MINUS_CONSTANT_COLOR,
            BlendFactor::ConstantAlpha => gl::CONSTANT_ALPHA,
            BlendFactor::OneMinusConstantAlpha => gl::ONE_MINUS_CONSTANT_ALPHA,
            BlendFactor::SrcAlphaSaturate => gl::SRC_ALPHA_SATURATE,
            BlendFactor::Src1Color => gl::SRC1_COLOR,
            BlendFactor::OneMinusSrc1Color => gl::ONE_MINUS_SRC1_COLOR,
            BlendFactor::Src1Alpha => gl::SRC1_ALPHA,
            BlendFactor::OneMinusSrc1Alpha => gl::ONE_MINUS_SRC1_ALPHA,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlendMask(u32);

impl BlendMask {
    pub const NONE: Self = Self(0);
    pub const RED: Self = Self(1);
    pub const GREEN: Self = Self(2);
    pub const BLUE: Self = Self(4);
    pub const ALPHA: Self = Self(8);
    pub const RGB: Self = Self(Self::RED.0 | Self::GREEN.0 | Self::BLUE.0);
    pub const RGBA: Self = Self(Self::RGB.0 | Self::ALPHA.0);
}
#[derive(Debug, Clone, Copy)]
pub struct BlendMode {
    pub color_op: BlendOp,
    pub color_src: BlendFactor,
    pub color_dst: BlendFactor,
    pub alpha_op: BlendOp,
    pub alpha_src: BlendFactor,
    pub alpha_dst: BlendFactor,
    pub mask: BlendMask,
}

impl BlendMode {
    pub const fn new_separate(
        color_op: BlendOp,
        color_src: BlendFactor,
        color_dst: BlendFactor,
        alpha_op: BlendOp,
        alpha_src: BlendFactor,
        alpha_dst: BlendFactor,
    ) -> Self {
        Self {
            color_op,
            color_src,
            color_dst,
            alpha_op,
            alpha_src,
            alpha_dst,
            mask: BlendMask::RGBA,
        }
    }
    pub const fn new(blend_op: BlendOp, src: BlendFactor, dst: BlendFactor) -> Self {
        Self {
            color_op: blend_op,
            color_src: src,
            color_dst: dst,
            alpha_op: blend_op,
            alpha_src: src,
            alpha_dst: dst,
            mask: BlendMask::RGBA,
        }
    }
}

pub const NORMAL: BlendMode = BlendMode::new_separate(
    BlendOp::Add,
    BlendFactor::SrcAlpha,
    BlendFactor::OneMinusSrcAlpha,
    BlendOp::Add,
    BlendFactor::SrcAlpha,
    BlendFactor::OneMinusSrcAlpha,
);
pub const ADDITIVE: BlendMode = BlendMode::new_separate(
    BlendOp::Add,
    BlendFactor::SrcAlpha,
    BlendFactor::One,
    BlendOp::Add,
    BlendFactor::SrcAlpha,
    BlendFactor::One,
);
pub const SUBSTRACT: BlendMode = BlendMode::new_separate(
    BlendOp::ReverseSubtract,
    BlendFactor::One,
    BlendFactor::One,
    BlendOp::Add,
    BlendFactor::One,
    BlendFactor::One,
);
