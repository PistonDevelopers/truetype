
use Error;
use Result;
use byteorder::{BigEndian, ByteOrder};

#[derive(Debug)]
pub struct CMAP {
    encoding_subtable: EncodingSubtable,
    cmap_offset: usize,
}

impl CMAP {
    pub fn from_data(data: &[u8], offset: usize) -> Result<Self> {

        if offset >= data.len() || offset + 4 > data.len() {
            return Err(Error::Malformed);
        }

        // +2 skip version field.
        let number_subtables = BigEndian::read_u16(&data[offset + 2..]) as usize;
        let data = &data[offset + 4..];
        if number_subtables * (2 + 2 + 4) > data.len() {
            return Err(Error::Malformed);
        }

        let mut encoding_subtables: Vec<_> = (0..number_subtables).filter_map(|n| {
            let z = n as usize * 8;
            let platform_id = BigEndian::read_u16(&data[z + 0..]);
            let platform_specific_id = BigEndian::read_u16(&data[z + 2..]);
            let offset = BigEndian::read_u32(&data[z + 4..]);
            Platform::new(platform_id, platform_specific_id).map(|platform| {
                EncodingSubtable { platform: platform, offset: offset}
            })
        }).collect();

        encoding_subtables.sort_by(|a, b| a.order().cmp(&b.order()));

        if encoding_subtables.is_empty() {
            return Err(Error::CMAPEncodingSubtableIsNotSupported);
        }

        Ok(CMAP {
            encoding_subtable: encoding_subtables.first().unwrap().clone(),
            cmap_offset: offset,
        })
    }

    pub fn index_map(&self) -> usize {
        self.encoding_subtable.offset as usize + self.cmap_offset
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct EncodingSubtable {
    platform: Platform,
    offset: u32,
}

impl EncodingSubtable {
    /// Defines an order in which the encoding subtables should be selected.
    fn order(&self) -> u32 {
        use self::Platform::*;
        use self::UnicodeEncodingId::*;
        use self::MicrosoftEncodingId::*;

        match self.platform {
            Unicode(Unicode20) => 0,
            Unicode(Unicode20BMPOnly) => 1,
            Unicode(Version11Semantics) => 1,
            Unicode(DefaultSemantics) => 1,
            Microsoft(UnicodeUCS4) => 2,
            Microsoft(UnicodeUCS2) => 3,
            Microsoft(Symbol) => 4,
            _ => 10,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Platform {
    Unicode(UnicodeEncodingId),
    Microsoft(MicrosoftEncodingId),
}

impl Platform {
    fn new(platform_id: u16, platform_specific_id: u16) -> Option<Self> {
        use self::Platform::*;
        use self::UnicodeEncodingId::*;
        use self::MicrosoftEncodingId::*;

        match platform_id {
            0 => match platform_specific_id {
                0 => Some(Unicode(DefaultSemantics)),
                1 => Some(Unicode(Version11Semantics)),
                3 => Some(Unicode(Unicode20BMPOnly)),
                4 => Some(Unicode(Unicode20)),
                5 => Some(Unicode(UnicodeVariationSequences)),
                6 => Some(Unicode(FullUnicodeCoverage)),
                _ => None,
            },
            3 => match platform_specific_id {
                0 => Some(Microsoft(Symbol)),
                1 => Some(Microsoft(UnicodeUCS2)),
                2 => Some(Microsoft(ShiftJIS)),
                3 => Some(Microsoft(PRC)),
                4 => Some(Microsoft(BigFive)),
                5 => Some(Microsoft(Johab)),
                10 => Some(Microsoft(UnicodeUCS4)),
                _ => None,
            },
            _ => None,
        }
    }
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum UnicodeEncodingId {
    DefaultSemantics = 0,
    Version11Semantics = 1,
    Unicode20BMPOnly = 3,
    Unicode20 = 4,
    UnicodeVariationSequences = 5,
    FullUnicodeCoverage = 6,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum MicrosoftEncodingId {
    Symbol = 0,
    UnicodeUCS2 = 1,
    ShiftJIS = 2,
    PRC = 3,
    BigFive = 4,
    Johab = 5,
    UnicodeUCS4 = 10
}

#[derive(Debug)]
struct Format0 {
    format: u16,
    length: u16,
    language: u16,
    glyph_index_array: Vec<u8>,
}

impl Format0 {
    fn from_data(data: &[u8], offset: usize) -> Result<Self> {
        const SIZE: usize = 262;
        if offset + SIZE > data.len() {
            return Err(Error::Malformed);
        }

        let format = BigEndian::read_u16(&data[offset..]);
        let length = BigEndian::read_u16(&data[offset + 2..]);
        if length as usize != SIZE {
            return Err(Error::Malformed);
        }
        let language = BigEndian::read_u16(&data[offset + 4..]);

        Ok(Format0 {
            format: format,
            length: length,
            language: language,
            glyph_index_array: data[offset + 6..SIZE].to_owned(),
        })
    }

    fn index_for_code(&self, code: usize) -> Option<usize> {
        self.glyph_index_array.get(code).map(|&i| i as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Error::*;
    use expectest::prelude::*;

    #[test]
    fn smoke() {
        let data = ::utils::read_file("tests/Tuffy_Bold.ttf");
        let offset = ::utils::find_table_offset(&data, 0, b"cmap").unwrap().unwrap();

        let _ = CMAP::from_data(&data, offset).unwrap();
    }
}
