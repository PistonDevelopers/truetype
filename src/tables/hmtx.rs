
use Error;
use Result;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

/// A record of horizontal metrics.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct LongHorizontalMetric {
    pub advance_width: u16,
    pub left_side_bearing: i16,
}

/// A table of horizontal metrics.
///
/// The 'hmtx' table contains metric information for the horizontal layout
/// each of the glyphs in the font.
#[derive(Debug, Default)]
pub struct HMTX {
    metrics: Vec<LongHorizontalMetric>,
    left_side_bearings: Vec<i16>,
}

impl HMTX {
    /// Returns `hmtx` font table.
    ///
    /// Attempts to read `data` starting from `offset` position.
    /// `metrics` is a number of long horizontal metrics taken from `hhea`
    /// font table.
    /// `glyphs` is a number of glyphs in the font.
    ///
    /// # Errors
    /// Returns error if there is not enough data to read or the number of
    /// `metrics` is greater than the number of `glyphs`.
    pub fn from_data(data: &[u8], offset: usize, metrics: usize, glyphs: usize) -> Result<HMTX> {
        if offset >= data.len() {
            return Err(Error::Malformed);
        }
        if metrics > glyphs {
            return Err(Error::Malformed);
        }
        let bearings = glyphs - metrics;

        let mut hmtx = HMTX {
            metrics: Vec::with_capacity(metrics),
            left_side_bearings: Vec::with_capacity(bearings),
        };

        let mut cursor = Cursor::new(&data[offset..]);
        for _ in 0..metrics {
            let w = try!(cursor.read_u16::<BigEndian>());
            let b = try!(cursor.read_i16::<BigEndian>());
            hmtx.metrics.push(LongHorizontalMetric { advance_width: w, left_side_bearing: b });
        }

        for _ in 0..bearings {
            hmtx.left_side_bearings.push(try!(cursor.read_i16::<BigEndian>()));
        }

        Ok(hmtx)
    }

    #[cfg(test)]
    fn bytes(&self) -> Vec<u8> {
        use byteorder::WriteBytesExt;

        let mut data = vec![];
        for metric in &self.metrics {
            data.write_u16::<BigEndian>(metric.advance_width).unwrap();
            data.write_i16::<BigEndian>(metric.left_side_bearing).unwrap();
        }
        for &bearing in &self.left_side_bearings {
            data.write_i16::<BigEndian>(bearing).unwrap();
        }
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Error::*;
    use expectest::prelude::*;

    #[test]
    fn smoke() {

    }
}