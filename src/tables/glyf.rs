
use Error;
use Result;

#[derive(Debug)]
pub struct GLYF {
    bytes: Vec<u8>,
}


impl GLYF {
    pub fn from_data(data: &[u8], offset: usize, size: usize) -> Result<Self> {
        if offset + size >= data.len() {
            return Err(Error::Malformed);
        }

        Ok(GLYF {
            bytes: data[offset..offset + size].to_owned(),
        })
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
