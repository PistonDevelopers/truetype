
mod hhea;

pub use self::hhea::HHEA;

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Fixed(pub i32);
