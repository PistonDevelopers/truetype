
/// A bounding box type.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct BBox {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Fixed(pub i32);

/// Indicates the type of offset format used in the index to loc ('loca') table.
///
/// Taken from `indexToLocFormat` field of the `head` font table.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LocationFormat {
    Short,
    Long,
}

impl LocationFormat {
    /// Returns size in bytes of the one entry in the `loca` font table.
    #[cfg(test)]
    pub fn entry_size(&self) -> u32 {
        match *self {
            LocationFormat::Short => 2,
            LocationFormat::Long => 4,
        }
    }
}
