extern crate piston_truetype;

use std::ptr::{null_mut};
use piston_truetype::*;

fn expect_glyph(letter: char, expected: String) {
    unsafe {
        let bs = include_bytes!("Tuffy_Bold.ttf");
        let s = 20.0;

        let mut w = 0;
        let mut h = 0;

        let offset = get_font_offset_for_index(bs.as_ptr(),0) as usize;

        let font = FontInfo::new_with_offset(&bs[..], offset).ok().expect("Failed to load font");
        let scale = font.scale_for_pixel_height(s);
        let bitmap = get_codepoint_bitmap(&font, 0.0,scale, letter as isize, &mut w, &mut h, null_mut(),null_mut());

        let mut result = String::new();
        for j in 0..h {
            for i in 0..w {
                result.push([' ', '.', ':', 'i', 'o', 'V', 'M', '@'][((*bitmap.offset(j*w+i))>>5) as usize]);
            }
            result.push('\n');
        }

        if result != expected {
            println!("\n{:?}", expected);
            println!("{:?}", result);
            panic!("The `A` is malformed.\n\n\nExpected:\n{}|\n\nGot:\n{}|", result, expected);
        }
    }
}


#[test]
fn draw_capital_a() {
    expect_glyph('A', String::new() +
        "    VMM     \n" +
        "    @@@i    \n" +
        "   i@@@M    \n" +
        "   M@o@@.   \n" +
        "  .@@.V@o   \n" +
        "  o@M i@@   \n" +
        "  M@i .@@.  \n" +
        " .@@@@@@@o  \n" +
        " o@@@@@@@@  \n" +
        " @@o   .@@: \n" +
        ":@@.    M@V \n" +
        "V@M     i@@ \n" );
}

#[test]
fn draw_capital_g() {
    expect_glyph('G', String::new() +
        "     .     \n" +
        "   o@@@V.  \n" +
        "  M@@M@@@  \n" +
        " i@@:  Vo. \n" +
        " @@o       \n" +
        ".@@.       \n" +
        ".@@  .oooo:\n" +
        ".@@  .@@@@i\n" +
        " @@:  iiM@i\n" +
        " M@o    @@.\n" +
        " i@@:  i@@ \n" +
        "  M@@MM@@: \n" +
        "   o@@@@:  \n" +
        "     .     \n" );
}
