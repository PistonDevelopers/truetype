
/// A bounding box type.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct BBox {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}
