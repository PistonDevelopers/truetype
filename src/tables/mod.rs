
mod hhea;
mod head;

pub use self::hhea::HHEA;
pub use self::head::HEAD;

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Fixed(pub i32);


#[cfg(test)]
fn read_file(path: &str) -> Vec<u8> {
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

