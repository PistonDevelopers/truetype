#![feature(test)]

extern crate test;
extern crate piston_truetype;

use piston_truetype::*;

#[bench]
fn font_initialization(bencher: &mut test::Bencher) {
    let bs = include_bytes!("../tests/Tuffy_Bold.ttf");
    let data = test::black_box(&bs[..]);
    bencher.bytes = data.len() as u64;
    bencher.iter(|| {
        let f = FontInfo::new_with_offset(data, 0).ok().expect("Failed to load font");
        test::black_box(f)
    })
}
