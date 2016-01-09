
use Error;
use Result;
use byteorder::{BigEndian, ByteOrder};

/// Attempts to find the table offset in `data` for a font table `tag`
/// starting from a `fontstart` offset.
pub fn find_table_offset(data: &[u8], fontstart: usize, tag: &[u8; 4]) -> Result<Option<usize>> {
    let tabledir = fontstart + 12;
    if tabledir >= data.len() {
        return Err(Error::Malformed);
    }

    let num_tables = BigEndian::read_u16(&data[fontstart + 4..]) as usize;
    for table_chunk in data[tabledir..].chunks(16).take(num_tables) {
        if table_chunk.len()==16 && prefix_is_tag(table_chunk, tag) {
            return Ok(Some(BigEndian::read_u32(&table_chunk[8..12]) as usize));
        }
    }
    return Ok(None);
}

/// Compatibility with unsafe code. TODO: Remove as soon as possible.
pub unsafe fn find_table(data: *const u8, fontstart: u32, tag: &[u8; 4]) -> u32 {
    let slice = ::std::slice::from_raw_parts(data, 1024); // DANGER: Don't care about size.
    find_table_offset(slice, fontstart as usize, tag).unwrap_or(None).unwrap_or(0) as u32
}

/// Checks that perfix of `bs` is equal to `tag`.
pub fn prefix_is_tag(bs: &[u8], tag: &[u8; 4]) -> bool {
    bs.len()>=4 && bs[0]==tag[0] && bs[1]==tag[1] && bs[2]==tag[2] && bs[3]==tag[3]
}

#[cfg(test)]
pub fn read_file(path: &str) -> Vec<u8> {
    use std::fs::{self, File};
    use std::io::{Read};
    use std::path::PathBuf;

    let path = PathBuf::from(path);
    assert!(fs::metadata(&path).is_ok());
    let mut file = File::open(&path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_is_tag() {
        assert!(prefix_is_tag(b"abcde", b"abcd"));
        assert!(!prefix_is_tag(b"abc", b"abcd"));
        assert!(!prefix_is_tag(b"abcc", b"abcd"));
    }
}
