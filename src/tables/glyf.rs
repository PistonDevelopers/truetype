
use Error;
use Result;
use types::BBox;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

#[derive(Debug)]
pub struct GLYF {
    bytes: Vec<u8>,
}

impl GLYF {
    pub fn from_data(data: &[u8], offset: usize, size: usize) -> Result<Self> {
        if offset + size > data.len() {
            return Err(Error::Malformed);
        }

        Ok(GLYF {
            bytes: data[offset..offset + size].to_owned(),
        })
    }
}

/// Contains data for the glyph.
#[derive(Debug)]
pub struct GlyphData<'a> {
    bytes: &'a [u8],
}

impl<'a> GlyphData<'a> {
    /// Returns the number of contours in the glyph.
    pub fn number_of_contours(&self) -> isize {
        Cursor::new(self.bytes).read_i16::<BigEndian>().ok().unwrap_or(0) as isize
    }

    /// Returns `true` if nothing is drawn for this glyph.
    pub fn is_empty(&self) -> bool {
        self.number_of_contours() == 0
    }

    /// Returns the bounding box of the glyph.
    pub fn bounding_box(&self) -> Option<BBox> {
        if self.bytes.len() < 5 * 2 {
            return None;
        }

        let mut cursor = Cursor::new(&self.bytes[2..]);
        let x0 = cursor.read_i16::<BigEndian>().unwrap() as i32;
        let y0 = cursor.read_i16::<BigEndian>().unwrap() as i32;
        let x1 = cursor.read_i16::<BigEndian>().unwrap() as i32;
        let y1 = cursor.read_i16::<BigEndian>().unwrap() as i32;
        Some(BBox { x0: x0, y0: y0, x1: x1, y1: y1 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Error::*;
    use tables::{MAXP, HEAD, LOCA};

    #[test]
    fn smoke() {
        let data = ::utils::read_file("tests/Tuffy_Bold.ttf");
        let maxp_offset = ::utils::find_table_offset(&data, 0, b"maxp").unwrap().unwrap();
        let glyphs = MAXP::from_data(&data, maxp_offset).unwrap().num_glyphs();
        let head_offset = ::utils::find_table_offset(&data, 0, b"head").unwrap().unwrap();
        let format = HEAD::from_data(&data, head_offset).unwrap().location_format();
        let loca_offset = ::utils::find_table_offset(&data, 0, b"loca").unwrap().unwrap();
        let loca = LOCA::from_data(&data, loca_offset, glyphs, format).unwrap();

        let glyf_offset = ::utils::find_table_offset(&data, 0, b"glyf").unwrap().unwrap();
        let glyf = GLYF::from_data(&data, glyf_offset, loca.size_of_glyf_table()).unwrap();
    }
}
