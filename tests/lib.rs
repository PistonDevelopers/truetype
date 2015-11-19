extern crate piston_truetype;

use std::ptr::{null_mut};
use piston_truetype::*;

#[test]
fn draw_capital_a() {
    unsafe {
        let mut font = stbtt_fontinfo::uninitialized();
        let c = 'A' as u8;
        let s = 20.0;
        let mut ttf_buffer = include_bytes!("Tuffy_Bold.ttf").to_vec();

        let mut w = 0;
        let mut h = 0;

        let offset = stbtt_GetFontOffsetForIndex(ttf_buffer.as_ptr(),0) as isize;
        stbtt_InitFont(&mut font, ttf_buffer[..].as_mut_ptr(), offset);
        let scale = scale_for_pixel_height(&font, s);
        let bitmap = get_codepoint_bitmap(&font, 0.0,scale, c as isize, &mut w, &mut h, null_mut(),null_mut());

        let mut result = String::new();
        for j in 0..h {
            for i in 0..w {
                result.push([' ', '.', ':', 'i', 'o', 'V', 'M', '@'][((*bitmap.offset(j*w+i))>>5) as usize]);
            }
            result.push('\n');
        }

        let expected = String::new() +
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
            "V@M     i@@ \n";

        if result != expected {
            panic!("The `A` is malformed.\n\n\nExpected:\n{}\n\nGot:\n{}", result, expected);
        }
    }
}
