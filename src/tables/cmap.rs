
use Error;
use Result;
use byteorder::{BigEndian, ByteOrder, ReadBytesExt};
use utils::{read_u16_from_raw_data, read_i16_from_raw_data};

#[derive(Debug)]
pub struct CMAP {
    encoding_subtable: EncodingSubtable,
    cmap_offset: usize,
    format: Format,
}

impl CMAP {
    pub fn from_data(data: &[u8], offset: usize) -> Result<Self> {

        if offset >= data.len() || offset + 4 > data.len() {
            return Err(Error::Malformed);
        }

        // +2 skip version field.
        let number_subtables = BigEndian::read_u16(&data[offset + 2..]) as usize;
        let subtables_data = &data[offset + 4..];
        if number_subtables * (2 + 2 + 4) > data.len() {
            return Err(Error::Malformed);
        }

        let mut encoding_subtables: Vec<_> = (0..number_subtables).filter_map(|n| {
            let z = n as usize * 8;
            let platform_id = BigEndian::read_u16(&subtables_data[z + 0..]);
            let platform_specific_id = BigEndian::read_u16(&subtables_data[z + 2..]);
            let offset = BigEndian::read_u32(&subtables_data[z + 4..]);
            Platform::new(platform_id, platform_specific_id).map(|platform| {
                EncodingSubtable { platform: platform, offset: offset}
            })
        }).collect();

        encoding_subtables.sort_by(|a, b| a.order().cmp(&b.order()));

        if encoding_subtables.is_empty() {
            return Err(Error::CMAPEncodingSubtableIsNotSupported);
        }

        let encoding_subtable = encoding_subtables.first().unwrap().clone();
        let format = try!(Format::from_data(data, offset + encoding_subtable.offset as usize));

        Ok(CMAP {
            encoding_subtable: encoding_subtable,
            cmap_offset: offset,
            format: format,
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
enum Format {
    F0(Format0),
    F4(Format4),
    F6(Format6),
    F1213(Format1213),
}

impl Format {
    fn from_data(data: &[u8], offset: usize) -> Result<Self> {
        use self::Format::*;
        if offset + 2 > data.len() {
            return Err(Error::Malformed);
        }

        let format = BigEndian::read_u16(&data[offset..]);
        match format {
            0 => Ok(F0(try!(Format0::from_data(data, offset)))),
            4 => Ok(F4(try!(Format4::from_data(data, offset)))),
            6 => Ok(F6(try!(Format6::from_data(data, offset)))),
            12 | 13 => Ok(F1213(try!(Format1213::from_data(data, offset)))),
            _ => Err(Error::CMAPFormatIsNotSupported),
        }
    }
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

#[derive(Debug, Default)]
struct Format4 {
    format: u16,
    length: u16,
    language: u16,
    seg_count_x2: u16,
    search_range: u16,
    entry_selector: u16,
    range_shift: u16,
    end_code: Vec<u8>,
    reserved_pad: u16,
    start_code: Vec<u8>,
    id_delta: Vec<u8>,
    id_range_offset: Vec<u8>,
    glyph_index_array: Vec<u8>,
}

impl Format4 {
    fn from_data(data: &[u8], offset: usize) -> Result<Self> {
        if offset + 2 * 8 > data.len() {
            return Err(Error::Malformed);
        }

        let mut z = offset;
        let mut f = Format4::default();
        f.format = BigEndian::read_u16(&data[z..]);
        z += 2;
        f.length = BigEndian::read_u16(&data[z..]);
        z += 2;
        f.language = BigEndian::read_u16(&data[z..]);
        z += 2;
        f.seg_count_x2 = BigEndian::read_u16(&data[z..]);
        z += 2;
        f.search_range = BigEndian::read_u16(&data[z..]);
        z += 2;
        f.entry_selector = BigEndian::read_u16(&data[z..]);
        z += 2;
        f.range_shift = BigEndian::read_u16(&data[z..]);
        z += 2;


        // Check that length is correct.
        if (f.length as usize) < 2 * 8 + f.seg_count_x2 as usize * 4 {
            return Err(Error::Malformed);
        }

        f.end_code = data[z..z + f.seg_count_x2 as usize].to_owned();
        z += f.seg_count_x2 as usize;
        f.reserved_pad = BigEndian::read_u16(&data[z..]);
        z += 2;
        f.start_code = data[z..z + f.seg_count_x2 as usize].to_owned();
        z += f.seg_count_x2 as usize;
        f.id_delta = data[z..z + f.seg_count_x2 as usize].to_owned();
        z += f.seg_count_x2 as usize;
        f.id_range_offset = data[z..z + f.seg_count_x2 as usize].to_owned();
        z += f.seg_count_x2 as usize;
        f.glyph_index_array = data[z..z + f.length as usize].to_owned();

        Ok(f)
    }

    fn index_for_code(&self, code: usize) -> Option<usize> {
        if code >= 0xffff {
            return None;
        }

        let mut r = (None, None); // Just to reduce indentation.
        for i in 0..self.end_code.len() / 2 {
            if BigEndian::read_u16(&self.end_code[i * 2..]) as usize >= code {
                r = (self.segment_at_index(i), Some(i));
                break;
            }
        }

        if let (Some(s), Some(i)) = r {
            if s.start_code <= code {
                if s.id_range_offset == 0 {
                   return Some((s.id_delta + code as isize) as usize);
                }
                let index = s.id_range_offset / 2 + (code - s.start_code) + i;
                if let Some(glyph_id) = read_u16_from_raw_data(&self.glyph_index_array, index) {
                    if glyph_id != 0 {
                        return Some((glyph_id as isize + s.id_delta) as usize);
                    }
                }
            }
        }

        None
    }

    fn segment_at_index(&self, i: usize) -> Option<Format4Segment> {
        let s = read_u16_from_raw_data(&self.start_code, i);
        let e = read_u16_from_raw_data(&self.end_code, i);
        let d = read_i16_from_raw_data(&self.id_delta, i);
        let r = read_u16_from_raw_data(&self.id_range_offset, i);
        if let (Some(s), Some(e), Some(d), Some(r)) = (s, e, d, r) {
            Some(Format4Segment {
                start_code: s as usize,
                end_code: e as usize,
                id_delta: d as isize,
                id_range_offset: r as usize,
            })
        } else {
            None
        }
    }
}

struct Format4Segment {
    start_code: usize,
    end_code: usize,
    id_delta: isize,
    id_range_offset: usize,
}

#[derive(Debug)]
struct Format6 {
    format: u16,
    length: u16,
    language: u16,
    first_code: u16,
    entry_count: u16,
    raw_glyph_index_array: Vec<u8>,
}

impl Format6 {
    fn from_data(data: &[u8], offset: usize) -> Result<Self> {
        if offset + 2 * 5 > data.len() {
            return Err(Error::Malformed);
        }

        let format = BigEndian::read_u16(&data[offset..]);
        let length = BigEndian::read_u16(&data[offset + 2..]);
        let language = BigEndian::read_u16(&data[offset + 4..]);
        let first_code = BigEndian::read_u16(&data[offset + 6..]);
        let entry_count = BigEndian::read_u16(&data[offset + 8..]);

        let size = entry_count as usize * 2;
        if offset + 2 * 5 + size > data.len() {
            return Err(Error::Malformed);
        }

        Ok(Format6 {
            format: format,
            length: length,
            language: language,
            first_code: first_code,
            entry_count: entry_count,
            raw_glyph_index_array: data[offset + 2 * 5..size].to_owned(),
        })
    }

    fn index_for_code(&self, code: usize) -> Option<usize> {
        let first_code = self.first_code as usize;
        let entry_count = self.entry_count as usize;
        if code < first_code || code >= first_code + entry_count {
            None
        } else {
            let offset = (code - first_code) * 2;
            if offset >= self.raw_glyph_index_array.len() {
                None
            } else {
                Some(BigEndian::read_u16(&self.raw_glyph_index_array[offset..]) as usize)
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct GroupFormat1213 {
    start_char_code: u32,
    end_char_code: u32,
    start_glyph_code: u32,
}

#[derive(Debug, Default)]
struct Format1213 {
    format: u32,
    length: u32,
    language: u32,
    n_groups: u32,
    groups: Vec<GroupFormat1213>,
}

impl Format1213 {
    fn from_data(data: &[u8], offset: usize) -> Result<Self> {
        if offset + 4 * 4 > data.len() {
            return Err(Error::Malformed);
        }

        let mut f = Format1213::default();
        f.format = BigEndian::read_u32(&data[offset..]);
        f.length = BigEndian::read_u32(&data[offset + 4..]);
        f.language = BigEndian::read_u32(&data[offset + 8..]);
        f.n_groups = BigEndian::read_u32(&data[offset + 12..]);

        if offset + f.n_groups as usize * 12 > data.len() {
            return Err(Error::Malformed);
        }

        let data = &data[offset + 4 * 4..];
        for n in 0..f.n_groups {
            let z = n as usize * 3 * 4;
            let sc = BigEndian::read_u32(&data[z..]);
            let ec = BigEndian::read_u32(&data[z + 4..]);
            let sg = BigEndian::read_u32(&data[z + 8..]);
            f.groups.push(GroupFormat1213 {
                start_char_code: sc,
                end_char_code: ec,
                start_glyph_code: sg
            });
        }

        Ok(f)
    }

    fn index_for_code(&self, code: usize) -> Option<usize> {
        use std::cmp::Ordering::*;

        let group = self.groups.binary_search_by(|group| {
            if code < group.start_char_code as usize {
                Greater
            } else if code > group.end_char_code as usize {
                Less
            } else {
                Equal
            }
        }).ok().map(|i| self.groups[i]);

        group.map(|group| {
            if self.format == 12 << 16 { // format 12.0
                code - group.start_char_code as usize + group.start_glyph_code as usize
            } else {
                group.start_glyph_code as usize
            }
        })
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
