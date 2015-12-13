// stb_truetype.h - v1.08 - public domain
// authored from 2009-2015 by Sean Barrett / RAD Game Tools
//
//   This library processes TrueType files:
//        parse files
//        extract glyph metrics
//        extract glyph shapes
//        render glyphs to one-channel bitmaps with antialiasing (box filter)
//
//   Todo:
//        non-MS cmaps
//        crashproof on bad data
//        hinting? (no longer patented)
//        cleartype-style AA?
//        optimize: use simple memory allocator for intermediates
//        optimize: build edge-list directly from curves
//        optimize: rasterize directly from curves?
//
// ADDITIONAL CONTRIBUTORS
//
//   Mikko Mononen: compound shape support, more cmap formats
//   Tor Andersson: kerning, subpixel rendering
//
//   Bug/warning reports/fixes:
//       "Zer" on mollyrocket (with fix)
//       Cass Everitt
//       stoiko (Haemimont Games)
//       Brian Hook
//       Walter van Niftrik
//       David Gow
//       David Given
//       Ivan-Assen Ivanov
//       Anthony Pesch
//       Johan Duparc
//       Hou Qiming
//       Fabian "ryg" Giesen
//       Martins Mozeiko
//       Cap Petschulat
//       Omar Cornut
//       github:aloucks
//       Peter LaValle
//       Sergey Popov
//       Giumo X. Clanjor
//       Higor Euripedes
//
//   Misc other:
//       Ryan Gordon
//
// VERSION HISTORY
//
//   1.08 (2015-09-13) document stbtt_Rasterize(); fixes for vertical & horizontal edges
//   1.07 (2015-08-01) allow PackFontRanges to accept arrays of sparse codepoints;
//                     variant PackFontRanges to pack and render in separate phases;
//                     fix stbtt_GetFontOFfsetForIndex (never worked for non-0 input?);
//                     fixed an assert() bug in the new rasterizer
//                     replace assert() with STBTT_assert() in new rasterizer
//   1.06 (2015-07-14) performance improvements (~35% faster on x86 and x64 on test machine)
//                     also more precise AA rasterizer, except if shapes overlap
//                     remove need for STBTT_sort
//   1.05 (2015-04-15) fix misplaced definitions for STBTT_STATIC
//   1.04 (2015-04-15) typo in example
//   1.03 (2015-04-12) STBTT_STATIC, fix memory leak in new packing, various fixes
//
//   Full history can be found at the end of this file.
//
// LICENSE
//
//   This software is in the public domain. Where that dedication is not
//   recognized, you are granted a perpetual, irrevocable license to copy,
//   distribute, and modify this file as you see fit.
//
// USAGE
//
//   Include this file in whatever places neeed to refer to it. In ONE C/C++
//   file, write:
//      #define STB_TRUETYPE_IMPLEMENTATION
//   before the #include of this file. This expands out the actual
//   implementation into that C/C++ file.
//
//   To make the implementation private to the file that generates the implementation,
//      #define STBTT_STATIC
//
//   Simple 3D API (don't ship this, but it's fine for tools and quick start)
//           stbtt_BakeFontBitmap()               -- bake a font to a bitmap for use as texture
//           stbtt_GetBakedQuad()                 -- compute quad to draw for a given char
//
//   Improved 3D API (more shippable):
//           #include "stb_rect_pack.h"           -- optional, but you really want it
//           stbtt_PackBegin()
//           stbtt_PackSetOversample()            -- for improved quality on small fonts
//           stbtt_PackFontRanges()               -- pack and renders
//           stbtt_PackEnd()
//           stbtt_GetPackedQuad()
//
//   "Load" a font file from a memory buffer (you have to keep the buffer loaded)
//           stbtt_InitFont()
//           stbtt_GetFontOffsetForIndex()        -- use for TTC font collections
//
//   Render a unicode codepoint to a bitmap
//           stbtt_GetCodepointBitmap()           -- allocates and returns a bitmap
//           stbtt_MakeCodepointBitmap()          -- renders into bitmap you provide
//           stbtt_GetCodepointBitmapBox()        -- how big the bitmap must be
//
//   Character advance/positioning
//           stbtt_GetCodepointHMetrics()
//           stbtt_GetFontVMetrics()
//           stbtt_GetCodepointKernAdvance()
//
//   Starting with version 1.06, the rasterizer was replaced with a new,
//   faster and generally-more-precise rasterizer. The new rasterizer more
//   accurately measures pixel coverage for anti-aliasing, except in the case
//   where multiple shapes overlap, in which case it overestimates the AA pixel
//   coverage. Thus, anti-aliasing of intersecting shapes may look wrong. If
//   this turns out to be a problem, you can re-enable the old rasterizer with
//        #define STBTT_RASTERIZER_VERSION 1
//   which will incur about a 15% speed hit.
//
// ADDITIONAL DOCUMENTATION
//
//   Immediately after this block comment are a series of sample programs.
//
//   After the sample programs is the "header file" section. This section
//   includes documentation for each API function.
//
//   Some important concepts to understand to use this library:
//
//      Codepoint
//         Characters are defined by unicode codepoints, e.g. 65 is
//         uppercase A, 231 is lowercase c with a cedilla, 0x7e30 is
//         the hiragana for "ma".
//
//      Glyph
//         A visual character shape (every codepoint is rendered as
//         some glyph)
//
//      Glyph index
//         A font-specific integer ID representing a glyph
//
//      Baseline
//         Glyph shapes are defined relative to a baseline, which is the
//         bottom of uppercase characters. Characters extend both above
//         and below the baseline.
//
//      Current Point
//         As you draw text to the screen, you keep track of a "current point"
//         which is the origin of each character. The current point's vertical
//         position is the baseline. Even "baked fonts" use this model.
//
//      Vertical Font Metrics
//         The vertical qualities of the font, used to vertically position
//         and space the characters. See docs for stbtt_GetFontVMetrics.
//
//      Font Size in Pixels or Points
//         The preferred interface for specifying font sizes in stb_truetype
//         is to specify how tall the font's vertical extent should be in pixels.
//         If that sounds good enough, skip the next paragraph.
//
//         Most font APIs instead use "points", which are a common typographic
//         measurement for describing font size, defined as 72 points per inch.
//         stb_truetype provides a point API for compatibility. However, true
//         "per inch" conventions don't make much sense on computer displays
//         since they different monitors have different number of pixels per
//         inch. For example, Windows traditionally uses a convention that
//         there are 96 pixels per inch, thus making 'inch' measurements have
//         nothing to do with inches, and thus effectively defining a point to
//         be 1.333 pixels. Additionally, the TrueType font data provides
//         an explicit scale factor to scale a given font's glyphs to points,
//         but the author has observed that this scale factor is often wrong
//         for non-commercial fonts, thus making fonts scaled in points
//         according to the TrueType spec incoherently sized in practice.
//
// ADVANCED USAGE
//
//   Quality:
//
//    - Use the functions with Subpixel at the end to allow your characters
//      to have subpixel positioning. Since the font is anti-aliased, not
//      hinted, this is very import for quality. (This is not possible with
//      baked fonts.)
//
//    - Kerning is now supported, and if you're supporting subpixel rendering
//      then kerning is worth using to give your text a polished look.
//
//   Performance:
//
//    - Convert Unicode codepoints to glyph indexes and operate on the glyphs;
//      if you don't do this, stb_truetype is forced to do the conversion on
//      every call.
//
//    - There are a lot of memory allocations. We should modify it to take
//      a temp buffer and allocate from the temp buffer (without freeing),
//      should help performance a lot.
//
// NOTES
//
//   The system uses the raw data found in the .ttf file without changing it
//   and without building auxiliary data structures. This is a bit inefficient
//   on little-endian systems (the data is big-endian), but assuming you're
//   caching the bitmaps or glyph shapes this shouldn't be a big deal.
//
//   It appears to be very hard to programmatically determine what font a
//   given file is in a general way. I provide an API for this, but I don't
//   recommend it.
//
//
// SOURCE STATISTICS (based on v0.6c, 2050 LOC)
//
//   Documentation & header file        520 LOC  \___ 660 LOC documentation
//   Sample code                        140 LOC  /
//   Truetype parsing                   620 LOC  ---- 620 LOC TrueType
//   Software rasterization             240 LOC  \                           .
//   Curve tesselation                  120 LOC   \__ 550 LOC Bitmap creation
//   Bitmap management                  100 LOC   /
//   Baked bitmap interface              70 LOC  /
//   Font name matching & access        150 LOC  ---- 150
//   C runtime library abstraction       60 LOC  ----  60
//
//
// PERFORMANCE MEASUREMENTS FOR 1.06:
//
//                      32-bit     64-bit
//   Previous release:  8.83 s     7.68 s
//   Pool allocations:  7.72 s     6.34 s
//   Inline sort     :  6.54 s     5.65 s
//   New rasterizer  :  5.63 s     5.00 s

//////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////
////
////  SAMPLE PROGRAMS
////
//
//  Incomplete text-in-3d-api example, which draws quads properly aligned to be lossless
//

//////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////
////
////   INTEGRATION WITH YOUR CODEBASE
////
////   The following sections allow you to supply alternate definitions
////   of C library functions used by stb_truetype.

extern crate byteorder;
extern crate libc;

use std::ptr::{ null, null_mut };
use std::mem::size_of;
use std::ffi::CString;
use std::slice;
use byteorder::{BigEndian, ByteOrder};
use libc::{ c_void, free, malloc, size_t, c_char };


//   #define STBTT_ifloor(x)   ((int) floor(x))
fn ifloor(x: f32) -> isize {
    x.floor() as isize
}

macro_rules! STBTT_sqrt {
    ($x:expr) => {
        $x.sqrt()
    }
}

//   #define STBTT_sqrt(x)      sqrt(x)

macro_rules! STBTT_malloc {
    ($x:expr) => {
        malloc($x)
    }
}

   // #define your own functions "STBTT_malloc" / "STBTT_free" to avoid malloc.h
//   #define STBTT_malloc(x,u)  ((void)(u),malloc(x))

macro_rules! STBTT_free {
    ($x:expr) => {
        free($x)
    }
}
//   #define STBTT_free(x,u)    ((void)(u),free(x))

macro_rules! STBTT_assert {
    ($x:expr) => {
        assert!($x)
    }
}

//   #define STBTT_assert(x)    assert(x)

use libc::strlen as STBTT_strlen;

//   #define STBTT_strlen(x)    strlen(x)

use std::ptr::copy as STBTT_memcpy;

//   #define STBTT_memcpy       memcpy

//   #define STBTT_memset       memset

fn memset(buf: *mut c_void, b: u8, count: usize) {
    let buf = buf as *mut u8;
    for idx in 0..count {
        unsafe {
            *buf.offset(idx as isize) = b;
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
////
////   INTERFACE
////
////

//////////////////////////////////////////////////////////////////////////////
//
// TEXTURE BAKING API
//
// If you use this API, you only have to call two functions ever.
//

pub struct BakedChar {
    // coordinates of bbox in bitmap
    x0: u16,
    y0: u16,
    x1: u16,
    y1: u16,
    xoff: f32,
    yoff: f32,
    xadvance: f32,
}

pub struct AlignedQuad {
    // top-left
    x0: f32,
    y0: f32,
    s0: f32,
    t0: f32,
    // bottom-right
    x1: f32,
    y1: f32,
    s1: f32,
    t1: f32,
}

//////////////////////////////////////////////////////////////////////////////
//
// NEW TEXTURE BAKING API
//
// This provides options for packing multiple fonts into one atlas, not
// perfectly but better than nothing.

pub struct PackedChar {
    // coordinates of bbox in bitmap
    x0: u16,
    y0: u16,
    x1: u16,
    y1: u16,
    xoff: f32,
    yoff: f32,
    xadvance: f32,
    xoff2: f32,
    yoff2: f32,
}

// TODO: Macro
// #define STBTT_POINT_SIZE(x)   (-(x))

pub struct PackRange {
   font_size: f32,
   // if non-zero, then the chars are continuous, and this is the first codepoint
   first_unicode_codepoint_in_range: isize,
   // if non-zero, then this is an array of unicode codepoints
   array_of_unicode_codepoints: *const isize,
   num_chars: isize,
   // output
   chardata_for_range: *mut PackedChar,
   // don't set these, they're used internally
   h_oversample: u8,
   v_oversample: u8,
}

// this is an opaque structure that you shouldn't mess with which holds
// all the context needed from PackBegin to PackEnd.
pub struct PackContext {
   user_allocator_context: *const (),
   pack_info: *mut c_void,
   width: isize,
   height: isize,
   stride_in_bytes: isize,
   padding: isize,
   h_oversample: usize,
   v_oversample: usize,
   pixels: *mut u8,
   nodes: *mut c_void,
}

//////////////////////////////////////////////////////////////////////////////
//
// FONT LOADING
//
//

// The following structure is defined publically so you can declare one on
// the stack or as a global or etc, but you should treat it as opaque.
pub struct FontInfo<'a> {
   // pointer to .ttf file
   data: &'a [u8],
   // offset of start of font
   fontstart: usize,
   // number of glyphs, needed for range checking
   num_glyphs: usize,

   // table locations as offset from start of .ttf
   loca: usize,
   head: usize,
   glyf: usize,
   hhea: usize,
   hmtx: usize,
   kern: usize,
   // a cmap mapping for our chosen character encoding
   index_map: usize,
   // format needed to map from glyph index to glyph
   index_to_loc_format: usize,
}

pub enum Error {
    Malformed,
    MissingTable,
}

impl<'a> FontInfo<'a> {
    // Given an offset into the file that defines a font, this function builds
    // the necessary cached info for the rest of the system.
    pub fn new_with_offset(data: &[u8], fontstart: usize) -> Result<FontInfo, Error> {
        let mut info = FontInfo{
            data: data,
            fontstart: 0,
            num_glyphs: 0,
            loca: 0,
            head: 0,
            glyf: 0,
            hhea: 0,
            hmtx: 0,
            kern: 0,
            index_map: 0,
            index_to_loc_format: 0,
        };

        info.fontstart = fontstart;

        let cmap = try!(info.find_required_table(b"cmap"));
        info.loca = try!(info.find_required_table(b"loca"));
        info.head = try!(info.find_required_table(b"head"));
        info.glyf = try!(info.find_required_table(b"glyf"));
        info.hhea = try!(info.find_required_table(b"hhea"));
        info.hmtx = try!(info.find_required_table(b"hmtx"));
        info.kern = try!(info.find_table(b"kern")).unwrap_or(0);

        info.num_glyphs = match try!(info.find_table(b"maxp")) {
            Some(maxp) => try!(info.read_u16(maxp + 4)) as usize,
            None => 0xffff
        };

        // find a cmap encoding table we understand *now* to avoid searching
        // later. (todo: could make this installable)
        // the same regardless of glyph.
        let num_tables = try!(info.read_u16(cmap + 2));
        info.index_map = 0;
        for encoding_record in info.data[cmap + 4..].chunks(8).take(num_tables as usize) {
            if encoding_record.len() != 8 {
                return Err(Error::Malformed);
            }
            let val: PlatformId = BigEndian::read_u16(&encoding_record[0..2]).into();
            match val {
                PlatformId::Microsoft => {
                    let val: MsEid = BigEndian::read_u16(&encoding_record[2..4]).into();
                    match val {
                        MsEid::UnicodeBmp
                        | MsEid::UnicodeFull => {
                            // MS/Unicode
                            info.index_map = cmap + BigEndian::read_u32(&encoding_record[4..8]) as usize;
                        }
                        _ => {
                            // TODO: Check extra cases.
                        }
                    }
                }
                PlatformId::Unicode => {
                    // Mac/iOS has these
                    // all the encodingIDs are unicode, so we don't bother to check it
                    info.index_map = cmap + BigEndian::read_u32(&encoding_record[4..8]) as usize;
                }
                _ => {
                    // TODO: Mac not supported?
                }
            }
        }
        if info.index_map == 0 {
            return Err(Error::MissingTable);
        }

        info.index_to_loc_format = try!(info.read_u16(info.head+50)) as usize;

        Ok(info)
    }

    fn read_u16(&self, offset: usize) -> Result<u16, Error> {
        if self.data.len()<2 || offset >= self.data.len() {
            return Err(Error::Malformed);
        }

        Ok(BigEndian::read_u16(&self.data[offset..offset+2]))
    }

    fn find_required_table(&self, tag: &[u8; 4]) -> Result<usize, Error> {
        match try!(self.find_table(tag)) {
            Some(offset) => Ok(offset),
            None => Err(Error::MissingTable)
        }
    }

    fn find_table(&self, tag: &[u8; 4]) -> Result<Option<usize>, Error> {
        let num_tables = try!(self.read_u16(self.fontstart + 4)) as usize;
        let tabledir: usize = self.fontstart + 12;

        if tabledir > self.data.len() {
            return Err(Error::Malformed);
        }
        for table_chunk in self.data[tabledir..].chunks(16).take(num_tables) {
            if table_chunk.len()==16 && prefix_is_tag(table_chunk, tag) {
                return Ok(Some(BigEndian::read_u32(&table_chunk[8..12]) as usize));
            }
        }
        return Ok(None);
    }
}

fn prefix_is_tag(bs: &[u8], tag: &[u8; 4]) -> bool {
    bs.len()>=4 && bs[0]==tag[0] && bs[1]==tag[1] && bs[2]==tag[2] && bs[3]==tag[3]
}

//////////////////////////////////////////////////////////////////////////////
//
// CHARACTER TO GLYPH-INDEX CONVERSIOn

//////////////////////////////////////////////////////////////////////////////
//
// CHARACTER PROPERTIES
//

//////////////////////////////////////////////////////////////////////////////
//
// GLYPH SHAPES (you probably don't need these, but they have to go before
// the bitmaps for C declaration-order reasons)
//

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum Cmd {
  Move=1,
  Line=2,
  Curve=3
}

type VertexType = i16;
#[derive(Copy, Clone)]
pub struct Vertex {
   x: i16,
   y: i16,
   cx: i16,
   cy: i16,
   type_: Cmd,
   flags: u8,
}

// @TODO: don't expose this structure
pub struct Bitmap
{
    w: isize,
    h: isize,
    stride: isize,
    pixels: *mut u8,
}

//////////////////////////////////////////////////////////////////////////////
//
// Finding the right font...
//
// You should really just solve this offline, keep your own tables
// of what font is what, and don't try to get it out of the .ttf file.
// That's because getting it out of the .ttf file is really hard, because
// the names in the file can appear in many possible encodings, in many
// possible languages, and e.g. if you need a case-insensitive comparison,
// the details of that depend on the encoding & language in a complex way
// (actually underspecified in truetype, but also gigantic).
//
// But you can use the provided functions in two possible ways:
//     stbtt_FindMatchingFont() will use *case-sensitive* comparisons on
//             unicode-encoded names to try to find the font you want;
//             you can run this before calling stbtt_InitFont()
//
//     stbtt_GetFontNameString() lets you get any of the various strings
//             from the file yourself and do your own comparisons on them.
//             You have to have called stbtt_InitFont() first.

// const STBTT_MACSTYLE_DONTCARE: u8 = 0;
// const STBTT_MACSTYLE_BOLD: u8 = 1;
// const STBTT_MACSTYLE_ITALIC: u8 = 2;
// const STBTT_MACSTYLE_UNDERSCORE: u8 = 4;
// const STBTT_MACSTYLE_NONE: u8 = 8;   // <= not same as 0, this makes us check the bitfield is 0

enum PlatformId { // platform_id
   Unicode   =0,
   Mac       =1,
   Iso       =2,
   Microsoft =3
}

impl From<u16> for PlatformId {
    fn from(val: u16) -> PlatformId {
        match val {
            0 => PlatformId::Unicode,
            1 => PlatformId::Mac,
            2 => PlatformId::Iso,
            3 => PlatformId::Microsoft,
            _ => panic!("Unknown STBTT_PLATFORM_ID")
        }
    }
}

/*
enum STBTT_UNICODE_EID { // encoding_id for STBTT_PLATFORM_ID_UNICODE
   UNICODE_1_0    =0,
   UNICODE_1_1    =1,
   ISO_10646      =2,
   UNICODE_2_0_BMP=3,
   UNICODE_2_0_FULL=4
}
*/

enum MsEid { // encoding_id for STBTT_PLATFORM_ID_MICROSOFT
   Symbol        =0,
   UnicodeBmp    =1,
   ShiftJIS      =2,
   UnicodeFull   =10
}

impl From<u16> for MsEid {
    fn from(val: u16) -> MsEid {
        match val {
            0 => MsEid::Symbol,
            1 => MsEid::UnicodeBmp,
            2 => MsEid::ShiftJIS,
            10 => MsEid::UnicodeFull,
            _ => panic!("Unknown STBTT_MS_EID")
        }
    }
}

/*
enum STBTT_MAC_EID { // encoding_id for STBTT_PLATFORM_ID_MAC; same as Script Manager codes
   ROMAN        =0,   ARABIC       =4,
   JAPANESE     =1,   HEBREW       =5,
   CHINESE_TRAD =2,   GREEK        =6,
   KOREAN       =3,   RUSSIAN      =7
}
*/

/*
enum STBTT_MS_LANG { // language_id for STBTT_PLATFORM_ID_MICROSOFT; same as LCID...
       // problematic because there are e.g. 16 english LCIDs and 16 arabic LCIDs
   ENGLISH     =0x0409,   ITALIAN     =0x0410,
   CHINESE     =0x0804,   JAPANESE    =0x0411,
   DUTCH       =0x0413,   KOREAN      =0x0412,
   FRENCH      =0x040c,   RUSSIAN     =0x0419,
   GERMAN      =0x0407,   // TODO: Duplicate, SPANISH     =0x0409,
   HEBREW      =0x040d,   SWEDISH     =0x041D
}
*/

/*
enum STBTT_MAC_LANG { // language_id for STBTT_PLATFORM_ID_MAC
   ENGLISH      =0 ,   JAPANESE     =11,
   ARABIC       =12,   KOREAN       =23,
   DUTCH        =4 ,   RUSSIAN      =32,
   FRENCH       =1 ,   SPANISH      =6 ,
   GERMAN       =2 ,   SWEDISH      =5 ,
   HEBREW       =10,   CHINESE_SIMPLIFIED =33,
   ITALIAN      =3 ,   LANG_CHINESE_TRAD =19
}
*/

///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
////
////   IMPLEMENTATION
////
////

// Can not be > 255.
const STBTT_MAX_OVERSAMPLE: usize = 8;

// const STBTT_RASTERIZER_VERSION: u8 = 2;

//////////////////////////////////////////////////////////////////////////
//
// accessors to parse data from file
//

// on platforms that don't allow misaligned reads, if we want to allow
// truetype fonts that aren't padded to alignment, define ALLOW_UNALIGNED_TRUETYPE

macro_rules! ttBYTE {
    ($p:expr) => {
        *($p as *const u8)
    }
}

// #define ttBYTE(p)     (* (stbtt_uint8 *) (p))

macro_rules! ttCHAR {
    ($p:expr) => {
        *($p as *const i8)
    }
}

// #define ttCHAR(p)     (* (stbtt_int8 *) (p))
// TODO: Macro.
// #define ttFixed(p)    ttLONG(p)

// TODO: Find out what is right to do with big or small endian.

macro_rules! ttUSHORT {
    ($p:expr) => {
        BigEndian::read_u16(slice::from_raw_parts($p, 2))
    }
}

macro_rules! ttSHORT {
    ($p:expr) => {
        BigEndian::read_i16(slice::from_raw_parts($p, 2))
    }
}

macro_rules! ttULONG {
    ($p:expr) => {
        BigEndian::read_u32(slice::from_raw_parts($p, 4))
    }
}

macro_rules! ttLONG {
    ($p:expr) => {
        BigEndian::read_i32(slice::from_raw_parts($p, 4))
    }
}

macro_rules! stbtt_tag4 {
    ($p:expr, $c0:expr, $c1:expr, $c2:expr, $c3:expr) => {
        *$p.offset(0) == ($c0) && *$p.offset(1) == ($c1) && *$p.offset(2) == ($c2) && *$p.offset(3) == ($c3)
    }
}

// #define stbtt_tag4(p,c0,c1,c2,c3) ((p)[0] == (c0) && (p)[1] == (c1) && (p)[2] == (c2) && (p)[3] == (c3))

macro_rules! stbtt_tag {
    ($p:expr, $s:expr) => {
        stbtt_tag4!($p,*$s.offset(0),*$s.offset(1),*$s.offset(2),*$s.offset(3))
    }
}

// #define stbtt_tag(p,str)           stbtt_tag4(p,str[0],str[1],str[2],str[3])

pub unsafe fn isfont(font: *const u8) -> isize {
   // check the version number
   if stbtt_tag4!(font, '1' as u8,0,0,0) { return 1; } // TrueType 1
   if stbtt_tag!(font, "typ1".as_ptr())  { return 1; } // TrueType with type 1 font -- we don't support this!
   if stbtt_tag!(font, "OTTO".as_ptr())  { return 1; } // OpenType with CFF
   if stbtt_tag4!(font, 0,1,0,0) { return 1; } // OpenType 1.0
   return 0;
}

// @OPTIMIZE: binary search
pub unsafe fn find_table(
    data: *const u8,
    fontstart: u32,
    tag: *const c_char
) -> u32 {
   let num_tables: i32 = ttUSHORT!(data.offset(fontstart as isize +4)) as i32;
   let tabledir: u32 = fontstart + 12;
   for i in 0..num_tables {
      let loc: u32 = tabledir + 16*i as u32;
      if stbtt_tag!(data.offset(loc as isize +0), tag as *const u8) {
         return ttULONG!(data.offset(loc as isize +8));
      }
   }
   return 0;
}

// Each .ttf/.ttc file may have more than one font. Each font has a sequential
// index number starting from 0. Call this function to get the font offset for
// a given index; it returns -1 if the index is out of range. A regular .ttf
// file will only define one font and it always be at offset 0, so it will
// return '0' for index 0, and -1 for all other indices. You can just skip
// this step if you know it's that kind of font.
pub unsafe fn get_font_offset_for_index(
    font_collection: *const u8,
    index: isize
) -> i32 {
   // if it's just a font, there's only one valid index
   if isfont(font_collection) != 0 {
      return if index == 0 { 0 } else { -1 };
   }

   // check if it's a TTC
   if stbtt_tag!(font_collection, "ttcf".as_ptr()) {
      // version 1?
      if ttULONG!(font_collection.offset(4)) == 0x00010000
       || ttULONG!(font_collection.offset(4)) == 0x00020000 {
         let n: i32 = ttLONG!(font_collection.offset(8));
         if index >= n as isize {
            return -1;
         }
         return ttULONG!(font_collection.offset(12+index*4)) as i32;
      }
   }
   return -1;
}

// If you're going to perform multiple operations on the same character
// and you want a speed-up, call this function with the character you're
// going to process, then use glyph-based functions instead of the
// codepoint-based functions.
pub unsafe fn find_glyph_index(
    info: *const FontInfo,
    unicode_codepoint: isize
) -> isize {
   let data: *const u8 = (*info).data.as_ptr();
   let index_map: u32 = (*info).index_map as u32;

   let format: u16 = ttUSHORT!(data.offset(index_map as isize + 0));
   if format == 0 { // apple byte encoding
      let bytes: i32 = ttUSHORT!(data.offset(index_map as isize + 2)) as i32;
      if unicode_codepoint < bytes as isize -6 {
         return ttBYTE!(data.offset(index_map as isize + 6 + unicode_codepoint as isize)) as isize;
      }
      return 0;
   } else if format == 6 {
      let first: u32 = ttUSHORT!(data.offset(index_map as isize + 6)) as u32;
      let count: u32 = ttUSHORT!(data.offset(index_map as isize + 8)) as u32;
      if (unicode_codepoint as u32) >= first
      && (unicode_codepoint as u32) < first+count {
         return ttUSHORT!(data.offset(
             index_map as isize + 10 + (unicode_codepoint - first as isize)*2)) as isize;
      }
      return 0;
   } else if format == 2 {
      STBTT_assert!(false); // @TODO: high-byte mapping for japanese/chinese/korean
      return 0;
   } else if format == 4 { // standard mapping for windows fonts: binary search collection of ranges
      let segcount: u16 = ttUSHORT!(data.offset(index_map as isize +6)) >> 1;
      let mut search_range: u16 = ttUSHORT!(data.offset(index_map as isize +8)) >> 1;
      let mut entry_selector: u16 = ttUSHORT!(data.offset(index_map as isize +10));
      let range_shift: u16 = ttUSHORT!(data.offset(index_map as isize +12)) >> 1;

      // do a binary search of the segments
      let end_count: u32 = index_map + 14;
      let mut search: u32 = end_count;

      if unicode_codepoint > 0xffff {
         return 0;
      }

      // they lie from endCount .. endCount + segCount
      // but searchRange is the nearest power of two, so...
      if unicode_codepoint >= ttUSHORT!(data.offset(
          search as isize + range_shift as isize *2)) as isize {
         search += range_shift as u32 *2;
      }

      // now decrement to bias correctly to find smallest
      search -= 2;
      while entry_selector != 0 {
         let end: u16;
         search_range >>= 1;
         end = ttUSHORT!(data.offset(search as isize + search_range as isize *2));
         if unicode_codepoint > end as isize {
            search += search_range as u32 *2;
         }
         entry_selector -= 1;
      }
      search += 2;

      {
         let offset: u16;
         let start: u16;
         let item: u16 = ((search - end_count) >> 1) as u16;

         STBTT_assert!(unicode_codepoint <= ttUSHORT!(data.offset(
             end_count as isize + 2*item as isize)) as isize);
         start = ttUSHORT!(data.offset(index_map as isize + 14 +
             segcount as isize *2 + 2 + 2*item as isize));
         if unicode_codepoint < start as isize {
            return 0;
         }

         offset = ttUSHORT!(data.offset(index_map as isize + 14 +
             segcount as isize *6 + 2 + 2*item as isize));
         if offset == 0 {
            return (unicode_codepoint + ttSHORT!(data.offset(
                index_map as isize + 14 + segcount as isize *4 + 2 + 2*item as isize)) as isize)
                as isize;
         }

         return ttUSHORT!(data.offset(offset as isize +
             (unicode_codepoint-start as isize)*2 +
             index_map as isize + 14 + segcount as isize *6 + 2 + 2*item as isize)) as isize;
      }
   } else if format == 12 || format == 13 {
      let ngroups: u32 = ttULONG!(data.offset(index_map as isize +12));
      let mut low: i32;
      let mut high: i32;
      low = 0; high = ngroups as i32;
      // Binary search the right group.
      while low < high {
         let mid: i32 = low + ((high-low) >> 1); // rounds down, so low <= mid < high
         let start_char: u32 = ttULONG!(data.offset(index_map as isize +16+mid as isize *12));
         let end_char: u32 = ttULONG!(data.offset(index_map as isize +16+mid as isize*12+4));
         if (unicode_codepoint as u32) < start_char {
            high = mid;
         }
         else if (unicode_codepoint as u32) > end_char {
            low = mid+1;
         }
         else {
            let start_glyph: u32 = ttULONG!(data.offset(index_map as isize +16+mid as isize *12+8));
            if format == 12 {
               return start_glyph as isize + unicode_codepoint-start_char as isize;
            }
            else { // format == 13
               return start_glyph as isize;
            }
         }
      }
      return 0; // not found
   }
   // @TODO
   STBTT_assert!(false);
   return 0;
}

pub unsafe fn get_codepoint_shape(
    info: *const FontInfo,
    unicode_codepoint: isize,
     vertices: *mut *mut Vertex
) -> isize {
   return get_glyph_shape(info, find_glyph_index(info, unicode_codepoint), vertices);
}

pub unsafe fn stbtt_setvertex(
    v: *mut Vertex,
    type_: Cmd,
    x: i32,
    y: i32,
    cx: i32,
    cy: i32
) {
   (*v).type_ = type_;
   (*v).x = x as i16;
   (*v).y = y as i16;
   (*v).cx = cx as i16;
   (*v).cy = cy as i16;
}

pub unsafe fn get_glyph_offset(
    info: *const FontInfo,
    glyph_index: isize
) -> isize {
   let g1: isize;
   let g2: isize;

   if glyph_index >= (*info).num_glyphs as isize { return -1; } // glyph index out of range
   if (*info).index_to_loc_format >= 2   { return -1; } // unknown index->glyph map format

   if (*info).index_to_loc_format == 0 {
      g1 = (*info).glyf as isize + ttUSHORT!((*info).data.as_ptr().offset((*info).loca as isize + glyph_index * 2)) as isize * 2;
      g2 = (*info).glyf as isize + ttUSHORT!((*info).data.as_ptr().offset((*info).loca as isize + glyph_index * 2 + 2)) as isize * 2;
   } else {
      g1 = (*info).glyf as isize + ttULONG!((*info).data.as_ptr().offset((*info).loca as isize + glyph_index * 4)) as isize;
      g2 = (*info).glyf as isize + ttULONG!((*info).data.as_ptr().offset((*info).loca as isize + glyph_index * 4 + 4)) as isize;
   }

   return if g1==g2 { -1 } else { g1 }; // if length is 0, return -1
}

// as above, but takes one or more glyph indices for greater efficiency
pub unsafe fn get_glyph_box(
    info: *const FontInfo,
    glyph_index: isize,
    x0: *mut isize,
    y0: *mut isize,
    x1: *mut isize,
    y1: *mut isize
) -> isize {
   let g: isize = get_glyph_offset(info, glyph_index);
   if g < 0 { return 0; }

   if x0 != null_mut() { *x0 = ttSHORT!((*info).data.as_ptr().offset(g + 2)) as isize; }
   if y0 != null_mut() { *y0 = ttSHORT!((*info).data.as_ptr().offset(g + 4)) as isize; }
   if x1 != null_mut() { *x1 = ttSHORT!((*info).data.as_ptr().offset(g + 6)) as isize; }
   if y1 != null_mut() { *y1 = ttSHORT!((*info).data.as_ptr().offset(g + 8)) as isize; }
   return 1;
}

// Gets the bounding box of the visible part of the glyph, in unscaled coordinates
pub unsafe fn get_codepoint_box(
    info: *const FontInfo,
    codepoint: isize,
    x0: *mut isize,
    y0: *mut isize,
    x1: *mut isize,
    y1: *mut isize
) -> isize {
   return get_glyph_box(info, find_glyph_index(info,codepoint), x0,y0,x1,y1);
}

// returns non-zero if nothing is drawn for this glyph
pub unsafe fn is_glyph_empty(
    info: *const FontInfo,
    glyph_index: isize
) -> isize {
   let number_of_contours: i16;
   let g: isize = get_glyph_offset(info, glyph_index);
   if g < 0 { return 1; }
   number_of_contours = ttSHORT!((*info).data.as_ptr().offset(g));
   return if number_of_contours == 0 { 0 } else { 1 };
}

pub unsafe fn close_shape(
    vertices: *mut Vertex,
    mut num_vertices: isize,
    was_off: isize,
    start_off: isize,
    sx: i32,
    sy: i32,
    scx: i32,
    scy: i32,
    cx: i32,
    cy: i32
) -> isize {
   if start_off != 0 {
      if was_off != 0 {
         stbtt_setvertex(vertices.offset(num_vertices),
             Cmd::Curve, (cx+scx)>>1, (cy+scy)>>1, cx,cy);
         num_vertices += 1;
      }
      stbtt_setvertex(vertices.offset(num_vertices), Cmd::Curve, sx,sy,scx,scy);
      num_vertices += 1;
   } else {
      if was_off != 0 {
         stbtt_setvertex(vertices.offset(num_vertices), Cmd::Curve,sx,sy,cx,cy);
         num_vertices += 1;
      } else {
         stbtt_setvertex(vertices.offset(num_vertices), Cmd::Line,sx,sy,0,0);
         num_vertices += 1;
      }
   }
   return num_vertices;
}

// returns # of vertices and fills *vertices with the pointer to them
//   these are expressed in "unscaled" coordinates
//
// The shape is a series of countours. Each one starts with
// a STBTT_moveto, then consists of a series of mixed
// STBTT_lineto and STBTT_curveto segments. A lineto
// draws a line from previous endpoint to its x,y; a curveto
// draws a quadratic bezier from previous endpoint to
// its x,y, using cx,cy as the bezier control point.
pub unsafe fn get_glyph_shape(
    info: *const FontInfo,
    glyph_index: isize,
    pvertices: *mut *mut Vertex
) -> isize {
   let number_of_contours: i16;
   let end_pts_of_contours: *const u8;
   let data: *const u8 = (*info).data.as_ptr();
   let mut vertices: *mut Vertex=null_mut();
   let mut num_vertices: isize =0;
   let g: isize = get_glyph_offset(info, glyph_index);

   *pvertices = null_mut();

   if g < 0 { return 0; }

   number_of_contours = ttSHORT!(data.offset(g));

   if number_of_contours > 0 {
      let mut flags: u8 =0;
      let mut flagcount: u8;
      let ins: i32;
      let mut j: i32 =0;
      let m: i32;
      let n: i32;
      let mut next_move: i32;
      let mut was_off: i32 =0;
      let off: i32;
      let mut start_off: i32 =0;
      let mut x: i32;
      let mut y: i32;
      let mut cx: i32;
      let mut cy: i32;
      let mut sx: i32;
      let mut sy: i32;
      let mut scx: i32;
      let mut scy: i32;
      let mut points: *const u8;
      end_pts_of_contours = data.offset(g + 10);
      ins = ttUSHORT!(data.offset(g + 10 + number_of_contours as isize * 2)) as i32;
      points = data.offset(g + 10 + number_of_contours as isize * 2 + 2 + ins as isize);

      n = 1+ttUSHORT!(end_pts_of_contours.offset(number_of_contours as isize *2-2)) as i32;

      m = n + 2*number_of_contours as i32;  // a loose bound on how many vertices we might need
      vertices = STBTT_malloc!(m as usize * size_of::<Vertex>()) as *mut Vertex;
      if vertices == null_mut() {
         return 0;
      }

      next_move = 0;
      flagcount=0;

      // in first pass, we load uninterpreted data into the allocated array
      // above, shifted to the end of the array so we won't overwrite it when
      // we create our final data starting from the front

      off = m - n; // starting offset for uninterpreted data, regardless of how m ends up being calculated

      // first load flags

      for i in 0..n {
         if flagcount == 0 {
            flags = *points;
            points = points.offset(1);
            if (flags & 8) != 0 {
               flagcount = *points;
               points = points.offset(1);
            }
         } else {
            flagcount -= 1;
         }
         (*vertices.offset(off as isize +i as isize)).flags = flags;
      }
      // now load x coordinates
      x=0;
      for i in 0..n {
         flags = (*vertices.offset(off as isize + i as isize)).flags;
         if (flags & 2) != 0 {
            let dx: i16 = *points as i16;
            points = points.offset(1);
            x += if (flags & 16) != 0 { dx as i32 } else { -dx as i32 }; // ???
         } else {
            if (flags & 16) == 0 {
               x = x + BigEndian::read_i16(slice::from_raw_parts(points, 2)) as i32;
               points = points.offset(2);
            }
         }
         (*vertices.offset(off as isize +i as isize)).x = x as i16;
      }

      // now load y coordinates
      y=0;
      for i in 0..n {
         flags = (*vertices.offset(off as isize + i as isize)).flags;
         if (flags & 4) != 0 {
            let dy: i16 = *points as i16;
            points = points.offset(1);
            y += if (flags & 32) != 0 { dy as i32 } else { -dy as i32 }; // ???
         } else {
            if (flags & 32) == 0 {
               y = y + BigEndian::read_i16(slice::from_raw_parts(points, 2)) as i32;
               points = points.offset(2);
            }
         }
         (*vertices.offset(off as isize +i as isize)).y = y as i16;
      }

      // now convert them to our format
      num_vertices=0;
      sx = 0; sy = 0;
      cx = 0; cy = 0;
      scx = 0; scy = 0;
      let mut i_iter = (0..n).into_iter();
      let mut i = 0;
      while { if let Some(v) = i_iter.next() { i = v; true } else { false } } {
         flags = (*vertices.offset(off as isize +i as isize)).flags;
         x     = (*vertices.offset(off as isize +i as isize)).x as i32;
         y     = (*vertices.offset(off as isize +i as isize)).y as i32;
         if next_move == i {
            if i != 0 {
               num_vertices = close_shape(vertices,
                   num_vertices, was_off as isize, start_off as isize, sx,sy,scx,scy,cx,cy);
            }

            // now start the new one
            start_off = (1 - (flags & 1)) as i32;
            if start_off != 0 {
               // if we start off with an off-curve point, then when we need to find a point on the curve
               // where we can start, and we need to save some state for when we wraparound.
               scx = x;
               scy = y;
               if (*vertices.offset(off as isize +i as isize +1)).type_ == Cmd::Line {
                  // next point is also a curve point, so interpolate an on-point curve
                  sx = (x + (*vertices.offset(off as isize +i as isize +1)).x as i32) >> 1;
                  sy = (y + (*vertices.offset(off as isize +i as isize +1)).y as i32) >> 1;
               } else {
                  // otherwise just use the next point as our start point
                  sx = (*vertices.offset(off as isize +i as isize +1)).x as i32;
                  sy = (*vertices.offset(off as isize +i as isize +1)).y as i32;
                  i_iter.next(); // we're using point i+1 as the starting point, so skip it
               }
            } else {
               sx = x;
               sy = y;
            }
            stbtt_setvertex(vertices.offset(num_vertices), Cmd::Move,sx,sy,0,0);
            num_vertices += 1;
            was_off = 0;
            next_move = 1 + ttUSHORT!(end_pts_of_contours.offset(j as isize *2)) as i32;
            j += 1;
         } else {
            if (flags & 1) == 0 { // if it's a curve
               if was_off != 0 { // two off-curve control points in a row means interpolate an on-curve midpoint
                  stbtt_setvertex(vertices.offset(num_vertices),
                      Cmd::Curve, (cx+x)>>1, (cy+y)>>1, cx, cy);
                  num_vertices += 1;
               }
               cx = x;
               cy = y;
               was_off = 1;
            } else {
               if was_off != 0 {
                  stbtt_setvertex(vertices.offset(num_vertices), Cmd::Curve, x,y, cx, cy);
                  num_vertices += 1;
               } else {
                  stbtt_setvertex(vertices.offset(num_vertices), Cmd::Line, x,y,0,0);
                  num_vertices += 1;
               }
               was_off = 0;
            }
         }
      }
      num_vertices = close_shape(vertices, num_vertices, was_off as isize, start_off as isize, sx,sy,scx,scy,cx,cy);
   } else if number_of_contours == -1 {
      // Compound shapes.
      let mut more: isize = 1;
      let mut comp: *const u8 = data.offset(g + 10);
      num_vertices = 0;
      vertices = null_mut();
      while more != 0 {
         let flags: u16;
         let gidx: u16;
         let comp_num_verts: isize;
         let mut comp_verts: *mut Vertex = null_mut();
         let tmp: *mut Vertex;
         let mut mtx: [f32; 6] = [1.0,0.0,0.0,1.0,0.0,0.0];
         let m: f32;
         let n: f32;

         flags = ttSHORT!(comp) as u16; comp=comp.offset(2);
         gidx = ttSHORT!(comp) as u16; comp=comp.offset(2);

         if (flags & 2) != 0 { // XY values
            if (flags & 1) != 0 { // shorts
               mtx[4] = ttSHORT!(comp) as f32; comp=comp.offset(2);
               mtx[5] = ttSHORT!(comp) as f32; comp=comp.offset(2);
            } else {
               mtx[4] = ttCHAR!(comp) as f32; comp=comp.offset(1);
               mtx[5] = ttCHAR!(comp) as f32; comp=comp.offset(1);
            }
         }
         else {
            // @TODO handle matching point
            STBTT_assert!(false);
         }
         if (flags & (1<<3)) != 0 { // WE_HAVE_A_SCALE
             let v = ttSHORT!(comp) as f32 /16384.0; comp=comp.offset(2);
            mtx[0] = v;
            mtx[3] = v;
            mtx[1] = 0.0;
            mtx[2] = 0.0;
         } else if (flags & (1<<6)) != 0 { // WE_HAVE_AN_X_AND_YSCALE
            mtx[0] = ttSHORT!(comp) as f32 /16384.0; comp=comp.offset(2);
            mtx[1] = 0.0;
            mtx[2] = 0.0;
            mtx[3] = ttSHORT!(comp) as f32 /16384.0; comp=comp.offset(2);
         } else if (flags & (1<<7)) != 0 { // WE_HAVE_A_TWO_BY_TWO
            mtx[0] = ttSHORT!(comp) as f32 /16384.0; comp=comp.offset(2);
            mtx[1] = ttSHORT!(comp) as f32 /16384.0; comp=comp.offset(2);
            mtx[2] = ttSHORT!(comp) as f32 /16384.0; comp=comp.offset(2);
            mtx[3] = ttSHORT!(comp) as f32 /16384.0; comp=comp.offset(2);
         }

         // Find transformation scales.
         m = STBTT_sqrt!(mtx[0]*mtx[0] + mtx[1]*mtx[1]) as f32;
         n = STBTT_sqrt!(mtx[2]*mtx[2] + mtx[3]*mtx[3]) as f32;

         // Get indexed glyph.
         comp_num_verts = get_glyph_shape(info, gidx as isize, &mut comp_verts);
         if comp_num_verts > 0 {
            // Transform vertices.
            for i in 0..comp_num_verts {
               let v: *mut Vertex = comp_verts.offset(i);
               let mut x: VertexType;
               let mut y: VertexType;
               x=(*v).x; y=(*v).y;
               (*v).x = (m as f32 * (mtx[0]*x as f32 + mtx[2]*y as f32 + mtx[4])) as VertexType;
               (*v).y = (n as f32 * (mtx[1]*x as f32 + mtx[3]*y as f32 + mtx[5])) as VertexType;
               x=(*v).cx; y=(*v).cy;
               (*v).cx = (m as f32 * (mtx[0]*x as f32 + mtx[2]*y as f32 + mtx[4])) as VertexType;
               (*v).cy = (n as f32 * (mtx[1]*x as f32 + mtx[3]*y as f32 + mtx[5])) as VertexType;
            }
            // Append vertices.
            tmp = STBTT_malloc!((num_vertices+comp_num_verts) as usize *size_of::<Vertex>())
                as *mut Vertex;
            if tmp == null_mut() {
               if vertices != null_mut() { STBTT_free!(vertices as *mut c_void); }
               if comp_verts != null_mut() { STBTT_free!(comp_verts as *mut c_void); }
               return 0;
            }
            if num_vertices > 0 {
                STBTT_memcpy(tmp, vertices,
                    num_vertices as usize *size_of::<Vertex>());
            }
            STBTT_memcpy(tmp.offset(num_vertices), comp_verts,
                comp_num_verts as usize *size_of::<Vertex>());
            if vertices != null_mut() { STBTT_free!(vertices as *mut c_void); }
            vertices = tmp;
            STBTT_free!(comp_verts as *mut c_void);
            num_vertices += comp_num_verts;
         }
         // More components ?
         more = (flags & (1<<5)) as isize;
      }
   } else if number_of_contours < 0 {
      // @TODO other compound variations?
      STBTT_assert!(false);
   } else {
      // numberOfCounters == 0, do nothing
   }

   *pvertices = vertices;
   return num_vertices;
}

pub unsafe fn get_glyph_hmetrics(
    info: *const FontInfo,
    glyph_index: isize,
    advance_width: *mut isize,
    left_side_bearing: *mut isize
) {
   let num_of_long_hor_metrics: u16 = ttUSHORT!((*info).data.as_ptr().offset((*info).hhea as isize + 34));
   if glyph_index < num_of_long_hor_metrics as isize {
      if advance_width != null_mut() {
          *advance_width    = ttSHORT!((*info).data.as_ptr().offset((*info).hmtx as isize + 4*glyph_index)) as isize;
      }
      if left_side_bearing != null_mut() {
          *left_side_bearing = ttSHORT!((*info).data.as_ptr().offset((*info).hmtx as isize + 4*glyph_index + 2)) as isize;
      }
   } else {
      if advance_width != null_mut() {
          *advance_width    = ttSHORT!((*info).data.as_ptr().offset((*info).hmtx as isize + 4*(num_of_long_hor_metrics as isize -1))) as isize;
      }
      if left_side_bearing != null_mut() {
          *left_side_bearing = ttSHORT!((*info).data.as_ptr().offset(
              (*info).hmtx as isize + 4*num_of_long_hor_metrics as isize + 2*(glyph_index - num_of_long_hor_metrics as isize))) as isize;
      }
   }
}

pub unsafe fn get_glyph_kern_advance(
    info: *mut FontInfo,
    glyph1: isize,
    glyph2: isize
) -> isize {
   let data: *const u8 = (*info).data.as_ptr().offset((*info).kern as isize);
   let needle: u32;
   let mut straw: u32;
   let mut l: isize;
   let mut r: isize;
   let mut m: isize;

   // we only look at the first table. it must be 'horizontal' and format 0.
   if (*info).kern == 0 {
      return 0;
   }
   if ttUSHORT!(data.offset(2)) < 1 { // number of tables, need at least 1
      return 0;
   }
   if ttUSHORT!(data.offset(8)) != 1 { // horizontal flag must be set in format
      return 0;
   }

   l = 0;
   r = ttUSHORT!(data.offset(10)) as isize - 1;
   needle = (glyph1 << 16 | glyph2) as u32;
   while l <= r {
      m = (l + r) >> 1;
      straw = ttULONG!(data.offset(18+(m*6))); // note: unaligned read
      if needle < straw {
         r = m - 1;
      }
      else if needle > straw {
         l = m + 1;
      } else {
         return ttSHORT!(data.offset(22+(m*6))) as isize;
      }
   }
   return 0;
}

// an additional amount to add to the 'advance' value between ch1 and ch2
pub unsafe fn get_codepoint_kern_advance(
    info: *mut FontInfo,
    ch1: isize,
    ch2: isize
) -> isize {
   if (*info).kern == 0 { // if no kerning table, don't waste time looking up both codepoint->glyphs
      return 0;
   }
   return get_glyph_kern_advance(info, find_glyph_index(info,ch1), find_glyph_index(info,ch2));
}

// leftSideBearing is the offset from the current horizontal position to the left edge of the character
// advanceWidth is the offset from the current horizontal position to the next horizontal position
//   these are expressed in unscaled coordinates
pub unsafe fn get_codepoint_hmetrics(
    info: *const FontInfo,
    codepoint: isize,
    advance_width: *mut isize,
    left_side_bearing: *mut isize
) {
   get_glyph_hmetrics(info, find_glyph_index(info,codepoint), advance_width, left_side_bearing);
}

// ascent is the coordinate above the baseline the font extends; descent
// is the coordinate below the baseline the font extends (i.e. it is typically negative)
// lineGap is the spacing between one row's descent and the next row's ascent...
// so you should advance the vertical position by "*ascent - *descent + *lineGap"
//   these are expressed in unscaled coordinates, so you must multiply by
//   the scale factor for a given size
pub unsafe fn get_font_vmetrics(
    info: *const FontInfo,
    ascent: *mut isize,
    descent: *mut isize,
    line_gap: *mut isize
) {
   if ascent != null_mut() {
       *ascent  = ttSHORT!((*info).data.as_ptr().offset((*info).hhea as isize + 4)) as isize;
   }
   if descent != null_mut() {
       *descent = ttSHORT!((*info).data.as_ptr().offset((*info).hhea as isize + 6)) as isize;
   }
   if line_gap != null_mut() {
       *line_gap = ttSHORT!((*info).data.as_ptr().offset((*info).hhea as isize + 8)) as isize;
   }
}

// the bounding box around all possible characters
pub unsafe fn get_font_bounding_box(
    info: *const FontInfo,
    x0: *mut isize,
    y0: *mut isize,
    x1: *mut isize,
    y1: *mut isize
) {
   *x0 = ttSHORT!((*info).data.as_ptr().offset((*info).head as isize + 36)) as isize;
   *y0 = ttSHORT!((*info).data.as_ptr().offset((*info).head as isize + 38)) as isize;
   *x1 = ttSHORT!((*info).data.as_ptr().offset((*info).head as isize + 40)) as isize;
   *y1 = ttSHORT!((*info).data.as_ptr().offset((*info).head as isize + 42)) as isize;
}

// computes a scale factor to produce a font whose "height" is 'pixels' tall.
// Height is measured as the distance from the highest ascender to the lowest
// descender; in other words, it's equivalent to calling stbtt_GetFontVMetrics
// and computing:
//       scale = pixels / (ascent - descent)
// so if you prefer to measure height by the ascent only, use a similar calculation.
pub unsafe fn scale_for_pixel_height(
    info: *const FontInfo,
    height: f32
) -> f32 {
   let fheight = ttSHORT!((*info).data.as_ptr().offset((*info).hhea as isize + 4))
        - ttSHORT!((*info).data.as_ptr().offset((*info).hhea as isize + 6));
   return height / fheight as f32;
}

// computes a scale factor to produce a font whose EM size is mapped to
// 'pixels' tall. This is probably what traditional APIs compute, but
// I'm not positive.
pub unsafe fn scale_for_mapping_em_to_pixels(
    info: *const FontInfo,
    pixels: f32
) -> f32 {
   let units_per_em = ttUSHORT!((*info).data.as_ptr().offset((*info).head as isize + 18));
   return pixels / units_per_em as f32;
}

// frees the data allocated above

//////////////////////////////////////////////////////////////////////////////
//
// BITMAP RENDERING
//
pub unsafe fn free_shape(_info: *const FontInfo, v: *mut Vertex)
{
   STBTT_free!(v as *mut c_void);
}

//////////////////////////////////////////////////////////////////////////////
//
// antialiasing software rasterizer
//

pub unsafe fn get_glyph_bitmap_box_subpixel(
    font: *const FontInfo,
    glyph: isize,
    scale_x: f32,
    scale_y: f32,
    shift_x: f32,
    shift_y: f32,
    ix0: *mut isize,
    iy0: *mut isize,
    ix1: *mut isize,
    iy1: *mut isize
) {
   let mut x0: isize = 0;
   let mut y0: isize = 0;
   let mut x1: isize = 0;
   let mut y1: isize = 0;
   if get_glyph_box(font, glyph, &mut x0,&mut y0,&mut x1,&mut y1) == 0 {
      // e.g. space character
      if ix0 != null_mut() { *ix0 = 0; }
      if iy0 != null_mut() { *iy0 = 0; }
      if ix1 != null_mut() { *ix1 = 0; }
      if iy1 != null_mut() { *iy1 = 0; }
   } else {
      // move to integral bboxes (treating pixels as little squares, what pixels get touched)?
      if ix0 != null_mut() { *ix0 = ifloor( x0 as f32 * scale_x + shift_x); }
      if iy0 != null_mut() { *iy0 = ifloor(-y1 as f32 * scale_y + shift_y); }
      if ix1 != null_mut() { *ix1 = ( x1 as f32 * scale_x + shift_x).ceil() as isize; }
      if iy1 != null_mut() { *iy1 = (-y0 as f32 * scale_y + shift_y).ceil() as isize; }
   }
}

pub unsafe fn get_glyph_bitmap_box(
    font: *const FontInfo,
    glyph: isize,
    scale_x: f32,
    scale_y: f32,
    ix0: *mut isize,
    iy0: *mut isize,
    ix1: *mut isize,
    iy1: *mut isize
) {
   get_glyph_bitmap_box_subpixel(font, glyph, scale_x, scale_y,0.0,0.0, ix0, iy0, ix1, iy1);
}

// same as stbtt_GetCodepointBitmapBox, but you can specify a subpixel
// shift for the character
pub unsafe fn get_codepoint_bitmap_box_subpixel(
    font: *const FontInfo,
    codepoint: isize,
    scale_x: f32,
    scale_y: f32,
    shift_x: f32,
    shift_y: f32,
    ix0: *mut isize,
    iy0: *mut isize,
    ix1: *mut isize,
    iy1: *mut isize
) {
   get_glyph_bitmap_box_subpixel(font, find_glyph_index(font,codepoint), scale_x, scale_y,shift_x,shift_y, ix0,iy0,ix1,iy1);
}

// get the bbox of the bitmap centered around the glyph origin; so the
// bitmap width is ix1-ix0, height is iy1-iy0, and location to place
// the bitmap top left is (leftSideBearing*scale,iy0).
// (Note that the bitmap uses y-increases-down, but the shape uses
// y-increases-up, so CodepointBitmapBox and CodepointBox are inverted.)
pub unsafe fn get_codepoint_bitmap_box(
    font: *const FontInfo,
    codepoint: isize,
    scale_x: f32,
    scale_y: f32,
    ix0: *mut isize,
    iy0: *mut isize,
    ix1: *mut isize,
    iy1: *mut isize
) {
   get_codepoint_bitmap_box_subpixel(font, codepoint, scale_x, scale_y,0.0,0.0, ix0,iy0,ix1,iy1);
}

//////////////////////////////////////////////////////////////////////////////
//
//  Rasterizer

struct HheapChunk {
   next: *mut HheapChunk
}

pub struct Hheap
{
   head: *mut HheapChunk,
   first_free: *mut (),
   num_remaining_in_head_chunk: isize,
}

pub unsafe fn hheap_alloc(
    hh: *mut Hheap,
    size: size_t
) -> *const () {
   if (*hh).first_free != null_mut() {
      let p: *mut () = (*hh).first_free;
      (*hh).first_free = *(p as *mut *mut ());
      return p;
   } else {
      if (*hh).num_remaining_in_head_chunk == 0 {
         let count: isize = if size < 32 {
                2000
            } else {
                if size < 128 { 800 } else { 100 }
            };
         let c: *mut HheapChunk = STBTT_malloc!(
             size_of::<HheapChunk>() + size * count as usize)
             as *mut HheapChunk;
         if c == null_mut() {
            return null();
         }
         (*c).next = (*hh).head;
         (*hh).head = c;
         (*hh).num_remaining_in_head_chunk = count;
      }
      (*hh).num_remaining_in_head_chunk -= 1;
      return ((*hh).head as *const u8).offset(size as isize * (*hh).num_remaining_in_head_chunk)
            as *const ();
   }
}

pub unsafe fn hheap_free(hh: *mut Hheap, p: *mut ()) {
   *(p as *mut *mut ()) = (*hh).first_free;
   (*hh).first_free = p;
}

pub unsafe fn hheap_cleanup(hh: *mut Hheap) {
   let mut c: *mut HheapChunk = (*hh).head;
   while c != null_mut() {
      let n: *mut HheapChunk = (*c).next;
      STBTT_free!(c as *mut c_void);
      c = n;
   }
}

#[derive(Copy, Clone)]
pub struct Edge {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
   invert: isize,
}

pub struct ActiveEdge {
   next: *mut ActiveEdge,
   // TODO: Conditional compilation.
   // #if STBTT_RASTERIZER_VERSION==1
   // int x,dx;
   // float ey;
   // int direction;
   // #elif STBTT_RASTERIZER_VERSION==2
   fx: f32,
   fdx: f32,
   fdy: f32,
   direction: f32,
   sy: f32,
   ey: f32,
   // #else
   // #error "Unrecognized value of STBTT_RASTERIZER_VERSION"
   // #endif
}

// TODO: Conditional compilation.
// #if STBTT_RASTERIZER_VERSION == 1
// #define STBTT_FIXSHIFT   10
// #define STBTT_FIX        (1 << STBTT_FIXSHIFT)
// #define STBTT_FIXMASK    (STBTT_FIX-1)

/*
static stbtt__active_edge *stbtt__new_active(stbtt__hheap *hh, stbtt__edge *e, int off_x, float start_point)
{
   stbtt__active_edge *z = (stbtt__active_edge *) stbtt__hheap_alloc(hh, sizeof(*z));
   float dxdy = (e->x1 - e->x0) / (e->y1 - e->y0);
   if (!z) return z;

   // round dx down to avoid overshooting
   if (dxdy < 0)
      z->dx = -STBTT_ifloor(STBTT_FIX * -dxdy);
   else
      z->dx = STBTT_ifloor(STBTT_FIX * dxdy);

   z->x = STBTT_ifloor(STBTT_FIX * e->x0 + z->dx * (start_point - e->y0)); // use z->dx so when we offset later it's by the same amount
   z->x -= off_x * STBTT_FIX;

   z->ey = e->y1;
   z->next = 0;
   z->direction = e->invert ? 1 : -1;
   return z;
}
*/
// #elif STBTT_RASTERIZER_VERSION == 2
pub unsafe fn new_active(
    hh: *mut Hheap,
    e: *mut Edge,
    off_x: isize,
    start_point: f32
) -> *mut ActiveEdge {
   let z: *mut ActiveEdge = hheap_alloc(
       hh, size_of::<ActiveEdge>())
        as *mut ActiveEdge;
   let dxdy: f32 = ((*e).x1 - (*e).x0) / ((*e).y1 - (*e).y0);
   //STBTT_assert(e->y0 <= start_point);
   if z == null_mut() { return z; }
   (*z).fdx = dxdy;
   (*z).fdy = if dxdy != 0.0 { 1.0/dxdy } else { 0.0 };
   (*z).fx = (*e).x0 + dxdy * (start_point - (*e).y0);
   (*z).fx -= off_x as f32;
   (*z).direction = if (*e).invert != 0 { 1.0 } else { -1.0 };
   (*z).sy = (*e).y0;
   (*z).ey = (*e).y1;
   (*z).next = null_mut();
   return z;
}
// #else
// #error "Unrecognized value of STBTT_RASTERIZER_VERSION"
// #endif

// TODO: Conditional compilation.
/*
#if STBTT_RASTERIZER_VERSION == 1
// note: this routine clips fills that extend off the edges... ideally this
// wouldn't happen, but it could happen if the truetype glyph bounding boxes
// are wrong, or if the user supplies a too-small bitmap
static void stbtt__fill_active_edges(unsigned char *scanline, int len, stbtt__active_edge *e, int max_weight)
{
   // non-zero winding fill
   int x0=0, w=0;

   while (e) {
      if (w == 0) {
         // if we're currently at zero, we need to record the edge start point
         x0 = e->x; w += e->direction;
      } else {
         int x1 = e->x; w += e->direction;
         // if we went to zero, we need to draw
         if (w == 0) {
            int i = x0 >> STBTT_FIXSHIFT;
            int j = x1 >> STBTT_FIXSHIFT;

            if (i < len && j >= 0) {
               if (i == j) {
                  // x0,x1 are the same pixel, so compute combined coverage
                  scanline[i] = scanline[i] + (stbtt_uint8) ((x1 - x0) * max_weight >> STBTT_FIXSHIFT);
               } else {
                  if (i >= 0) // add antialiasing for x0
                     scanline[i] = scanline[i] + (stbtt_uint8) (((STBTT_FIX - (x0 & STBTT_FIXMASK)) * max_weight) >> STBTT_FIXSHIFT);
                  else
                     i = -1; // clip

                  if (j < len) // add antialiasing for x1
                     scanline[j] = scanline[j] + (stbtt_uint8) (((x1 & STBTT_FIXMASK) * max_weight) >> STBTT_FIXSHIFT);
                  else
                     j = len; // clip

                  for (++i; i < j; ++i) // fill pixels between x0 and x1
                     scanline[i] = scanline[i] + (stbtt_uint8) max_weight;
               }
            }
         }
      }

      e = e->next;
   }
}

static void stbtt__rasterize_sorted_edges(stbtt__bitmap *result, stbtt__edge *e, int n, int vsubsample, int off_x, int off_y)
{
   stbtt__hheap hh = { 0, 0, 0 };
   stbtt__active_edge *active = NULL;
   int y,j=0;
   int max_weight = (255 / vsubsample);  // weight per vertical scanline
   int s; // vertical subsample index
   unsigned char scanline_data[512], *scanline;

   if (result->w > 512)
      scanline = (unsigned char *) STBTT_malloc(result->w);
   else
      scanline = scanline_data;

   y = off_y * vsubsample;
   e[n].y0 = (off_y + result->h) * (float) vsubsample + 1;

   while (j < result->h) {
      STBTT_memset(scanline, 0, result->w);
      for (s=0; s < vsubsample; ++s) {
         // find center of pixel for this scanline
         float scan_y = y + 0.5f;
         stbtt__active_edge **step = &active;

         // update all active edges;
         // remove all active edges that terminate before the center of this scanline
         while (*step) {
            stbtt__active_edge * z = *step;
            if (z->ey <= scan_y) {
               *step = z->next; // delete from list
               STBTT_assert(z->direction);
               z->direction = 0;
               stbtt__hheap_free(&hh, z);
            } else {
               z->x += z->dx; // advance to position for current scanline
               step = &((*step)->next); // advance through list
            }
         }

         // resort the list if needed
         for(;;) {
            int changed=0;
            step = &active;
            while (*step && (*step)->next) {
               if ((*step)->x > (*step)->next->x) {
                  stbtt__active_edge *t = *step;
                  stbtt__active_edge *q = t->next;

                  t->next = q->next;
                  q->next = t;
                  *step = q;
                  changed = 1;
               }
               step = &(*step)->next;
            }
            if (!changed) break;
         }

         // insert all edges that start before the center of this scanline -- omit ones that also end on this scanline
         while (e->y0 <= scan_y) {
            if (e->y1 > scan_y) {
               stbtt__active_edge *z = stbtt__new_active(&hh, e, off_x, scan_y);
               // find insertion point
               if (active == NULL)
                  active = z;
               else if (z->x < active->x) {
                  // insert at front
                  z->next = active;
                  active = z;
               } else {
                  // find thing to insert AFTER
                  stbtt__active_edge *p = active;
                  while (p->next && p->next->x < z->x)
                     p = p->next;
                  // at this point, p->next->x is NOT < z->x
                  z->next = p->next;
                  p->next = z;
               }
            }
            ++e;
         }

         // now process all active edges in XOR fashion
         if (active)
            stbtt__fill_active_edges(scanline, result->w, active, max_weight);

         ++y;
      }
      STBTT_memcpy(result->pixels + j * result->stride, scanline, result->w);
      ++j;
   }

   stbtt__hheap_cleanup(&hh);

   if (scanline != scanline_data)
      STBTT_free(scanline);
}
*/
// #elif STBTT_RASTERIZER_VERSION == 2

// the edge passed in here does not cross the vertical line at x or the vertical line at x+1
// (i.e. it has already been clipped to those)
pub unsafe fn handle_clipped_edge(
    scanline: *mut f32,
    x: isize,
    e: *mut ActiveEdge,
    mut x0: f32,
    mut y0: f32,
    mut x1: f32,
    mut y1: f32
) {
   if y0 == y1 { return; }
   STBTT_assert!(y0 < y1);
   STBTT_assert!((*e).sy <= (*e).ey);
   if y0 > (*e).ey { return; }
   if y1 < (*e).sy { return; }
   if y0 < (*e).sy {
      x0 += (x1-x0) * ((*e).sy - y0) / (y1-y0);
      y0 = (*e).sy;
   }
   if y1 > (*e).ey {
      x1 += (x1-x0) * ((*e).ey - y1) / (y1-y0);
      y1 = (*e).ey;
   }

   if x0 == x as f32 {
      STBTT_assert!(x1 <= x as f32 +1.0);
   }
   else if x0 == x as f32 +1.0 {
      STBTT_assert!(x1 >= x as f32);
   }
   else if x0 <= x as f32 {
      STBTT_assert!(x1 <= x as f32);
   }
   else if x0 >= x as f32 +1.0 {
      STBTT_assert!(x1 >= x as f32 +1.0);
   }
   else {
      STBTT_assert!(x1 >= x as f32 && x1 <= x as f32 +1.0);
   }

   if x0 <= x as f32 && x1 <= x as f32 {
      *scanline.offset(x) += (*e).direction * (y1-y0);
   }
   else if x0 >= x as f32 +1.0 && x1 >= x as f32 +1.0 {}
   else {
      STBTT_assert!(x0 >= x as f32 && x0 <= x as f32 +1.0 && x1 >= x as f32 && x1 <= x as f32 +1.0);
      *scanline.offset(x) += (*e).direction * (y1-y0) * (1.0-((x0-x as f32)+(x1-x as f32))/2.0); // coverage = 1 - average x position
   }
}

pub unsafe fn fill_active_edges_new(
    scanline: *mut f32,
    scanline_fill: *mut f32,
    len: isize,
    mut e: *mut ActiveEdge,
    y_top: f32
) {
   let y_bottom: f32 = y_top+1.0;

   while e != null_mut() {
      // brute force every pixel

      // compute intersection points with top & bottom
      STBTT_assert!((*e).ey >= y_top);

      if (*e).fdx == 0.0 {
         let x0: f32 = (*e).fx;
         if x0 < len as f32 {
            if x0 >= 0.0 {
               handle_clipped_edge(scanline,x0 as isize,e, x0,y_top, x0,y_bottom);
               handle_clipped_edge(scanline_fill.offset(-1),x0 as isize +1,e, x0,y_top, x0,y_bottom);
            } else {
               handle_clipped_edge(scanline_fill.offset(-1),0,e, x0,y_top, x0,y_bottom);
            }
         }
      } else {
         let mut x0: f32 = (*e).fx;
         let dx: f32 = (*e).fdx;
         let xb: f32 = x0 + dx;
         let mut x_top: f32;
         let mut x_bottom: f32;
         let mut sy0: f32;
         let mut sy1: f32;
         let mut dy: f32 = (*e).fdy;
         STBTT_assert!((*e).sy <= y_bottom && (*e).ey >= y_top);

         // compute endpoints of line segment clipped to this scanline (if the
         // line segment starts on this scanline. x0 is the intersection of the
         // line with y_top, but that may be off the line segment.
         if (*e).sy > y_top {
            x_top = x0 + dx * ((*e).sy - y_top);
            sy0 = (*e).sy;
         } else {
            x_top = x0;
            sy0 = y_top;
         }
         if (*e).ey < y_bottom {
            x_bottom = x0 + dx * ((*e).ey - y_top);
            sy1 = (*e).ey;
         } else {
            x_bottom = xb;
            sy1 = y_bottom;
         }

         if x_top >= 0.0
          && x_bottom >= 0.0
          && x_top < len as f32
          && x_bottom < len as f32 {
            // from here on, we don't have to range check x values

            if x_top as isize == x_bottom as isize {
               let height: f32;
               // simple case, only spans one pixel
               let x = x_top as isize;
               height = sy1 - sy0;
               STBTT_assert!(x >= 0 && x < len);
               *scanline.offset(x) += (*e).direction * (1.0-((x_top - x as f32) + (x_bottom-x as f32))/2.0)  * height;
               *scanline_fill.offset(x) += (*e).direction * height; // everything right of this pixel is filled
            } else {
               let x1: isize;
               let x2: isize;
               let mut y_crossing: f32;
               let step: f32;
               let sign: f32;
               let mut area: f32;
               // covers 2+ pixels
               if x_top > x_bottom {
                  // flip scanline vertically; signed area is the same
                  let mut t: f32;
                  sy0 = y_bottom - (sy0 - y_top);
                  sy1 = y_bottom - (sy1 - y_top);
                  t = sy0;
                  sy0 = sy1;
                  sy1 = t;
                  t = x_bottom;
                  x_bottom = x_top;
                  x_top = t;
                  dy = -dy;
                  x0 = xb;
               }

               x1 = x_top as isize;
               x2 = x_bottom as isize;
               // compute intersection with y axis at x1+1
               y_crossing = (x1 as f32 +1.0 - x0) * dy + y_top;

               sign = (*e).direction;
               // area of the rectangle covered from y0..y_crossing
               area = sign * (y_crossing-sy0);
               // area of the triangle (x_top,y0), (x+1,y0), (x+1,y_crossing)
               (*scanline.offset(x1)) += area * (1.0-((x_top - x1 as f32)+(x1+1-x1) as f32)/2.0);

               step = sign * dy;
               for x in x1 + 1..x2 {
                  (*scanline.offset(x)) += area + step/2.0;
                  area += step;
               }
               y_crossing += dy * (x2 - (x1+1)) as f32;

               STBTT_assert!(area.abs() <= 1.01);

               (*scanline.offset(x2)) += area + sign * (1.0-((x2-x2) as f32
                    +(x_bottom-x2 as f32))/2.0) * (sy1-y_crossing);

               (*scanline_fill.offset(x2)) += sign * (sy1-sy0);
            }
         } else {
            // if edge goes outside of box we're drawing, we require
            // clipping logic. since this does not match the intended use
            // of this library, we use a different, very slow brute
            // force implementation
            for x in 0..len {
               // cases:
               //
               // there can be up to two intersections with the pixel. any intersection
               // with left or right edges can be handled by splitting into two (or three)
               // regions. intersections with top & bottom do not necessitate case-wise logic.
               //
               // the old way of doing this found the intersections with the left & right edges,
               // then used some simple logic to produce up to three segments in sorted order
               // from top-to-bottom. however, this had a problem: if an x edge was epsilon
               // across the x border, then the corresponding y position might not be distinct
               // from the other y segment, and it might ignored as an empty segment. to avoid
               // that, we need to explicitly produce segments based on x positions.

               // rename variables to clear pairs
               let y0: f32 = y_top;
               let x1: f32 = x as f32;
               let x2: f32 = x as f32 +1.0 as f32;
               let x3: f32 = xb;
               let y3: f32 = y_bottom;
               let y1: f32;
               let y2: f32;

               // x = e->x + e->dx * (y-y_top)
               // (y-y_top) = (x - e->x) / e->dx
               // y = (x - e->x) / e->dx + y_top
               y1 = (x as f32 - x0) / dx + y_top;
               y2 = (x as f32+1.0 - x0) / dx + y_top;

               if x0 < x1 && x3 > x2 {         // three segments descending down-right
                  handle_clipped_edge(scanline,x,e, x0,y0, x1,y1);
                  handle_clipped_edge(scanline,x,e, x1,y1, x2,y2);
                  handle_clipped_edge(scanline,x,e, x2,y2, x3,y3);
               } else if x3 < x1 && x0 > x2 {  // three segments descending down-left
                  handle_clipped_edge(scanline,x,e, x0,y0, x2,y2);
                  handle_clipped_edge(scanline,x,e, x2,y2, x1,y1);
                  handle_clipped_edge(scanline,x,e, x1,y1, x3,y3);
               } else if x0 < x1 && x3 > x1 {  // two segments across x, down-right
                  handle_clipped_edge(scanline,x,e, x0,y0, x1,y1);
                  handle_clipped_edge(scanline,x,e, x1,y1, x3,y3);
               } else if x3 < x1 && x0 > x1 {  // two segments across x, down-left
                  handle_clipped_edge(scanline,x,e, x0,y0, x1,y1);
                  handle_clipped_edge(scanline,x,e, x1,y1, x3,y3);
               } else if x0 < x2 && x3 > x2 {  // two segments across x+1, down-right
                  handle_clipped_edge(scanline,x,e, x0,y0, x2,y2);
                  handle_clipped_edge(scanline,x,e, x2,y2, x3,y3);
               } else if x3 < x2 && x0 > x2 {  // two segments across x+1, down-left
                  handle_clipped_edge(scanline,x,e, x0,y0, x2,y2);
                  handle_clipped_edge(scanline,x,e, x2,y2, x3,y3);
               } else {  // one segment
                  handle_clipped_edge(scanline,x,e, x0,y0, x3,y3);
               }
            }
         }
      }
      e = (*e).next;
   }
}

// directly AA rasterize edges w/o supersampling
pub unsafe fn rasterize_sorted_edges(
    result: *mut Bitmap,
    mut e: *mut Edge,
    n: isize,
    _vsubsample: isize,
    off_x: isize,
    off_y: isize
) {
   let mut hh: Hheap = Hheap {
      head: null_mut(),
      first_free: null_mut(),
      num_remaining_in_head_chunk: 0,
   };
   let mut active: *mut ActiveEdge = null_mut();
   let mut y: isize;
   let mut j: isize =0;
   let mut scanline_data: [f32; 129] = [0.0; 129];
   let scanline: *mut f32;
   let scanline2: *mut f32;

   if (*result).w > 64 {
      scanline = STBTT_malloc!(((*result).w*2+1) as usize * size_of::<f32>()) as *mut f32;
   } else {
      scanline = scanline_data.as_mut_ptr();
   }

   scanline2 = scanline.offset((*result).w);

   y = off_y;
   (*e.offset(n)).y0 = (off_y + (*result).h) as f32 + 1.0;

   while j < (*result).h {
      // find center of pixel for this scanline
      let scan_y_top: f32 = y as f32 + 0.0;
      let scan_y_bottom: f32 = y as f32 + 1.0;
      let mut step: *mut *mut ActiveEdge = &mut active;

      memset(scanline as *mut c_void, 0, (*result).w as usize * size_of::<f32>());
      memset(scanline2 as *mut c_void, 0,
          ((*result).w+1) as usize * size_of::<f32>());

      // update all active edges;
      // remove all active edges that terminate before the top of this scanline
      while (*step) != null_mut() {
          // Location B.
          let z: *mut ActiveEdge = *step;
         if (*z).ey <= scan_y_top {
            *step = (*z).next; // delete from list
            STBTT_assert!((*z).direction != 0.0);
            (*z).direction = 0.0;
            hheap_free(&mut hh, z as *mut ());
         } else {
            step = &mut ((**step).next); // advance through list
         }
      }

      // insert all edges that start before the bottom of this scanline
      while (*e).y0 <= scan_y_bottom {
         if (*e).y0 != (*e).y1 {
            let z: *mut ActiveEdge = new_active(
                &mut hh, e, off_x, scan_y_top);
            STBTT_assert!((*z).ey >= scan_y_top);
            // insert at front
            (*z).next = active;
            active = z;
         }
         e = e.offset(1);
      }

      // now process all active edges
      if active != null_mut() {
         fill_active_edges_new(scanline, scanline2.offset(1), (*result).w,
            active, scan_y_top);
      }

      {
         let mut sum: f32 = 0.0;
         for i in 0..(*result).w {
            let mut k: f32;
            let mut m: isize;
            sum += *scanline2.offset(i);
            k = *scanline.offset(i) + sum;
            k = k.abs() as f32 * 255.0 as f32 + 0.5;
            m = k as isize;
            if m > 255 { m = 255; }
            *(*result).pixels.offset(j*(*result).stride + i) = m as u8;
         }
      }
      // advance all the edges
      step = &mut active;
      while *step != null_mut() {
         let z: *mut ActiveEdge = *step;
         (*z).fx += (*z).fdx; // advance to position for current scanline
         step = &mut ((**step).next); // advance through list
      }

      y += 1;
      j += 1;
   }

   hheap_cleanup(&mut hh);

   if scanline != scanline_data.as_mut_ptr() {
      STBTT_free!(scanline as *mut c_void);
   }
}
// #else
// #error "Unrecognized value of STBTT_RASTERIZER_VERSION"
// #endif

macro_rules! STBTT__COMPARE {
    ($a:expr, $b:expr) => {
        ($a).y0 < ($b).y0
    }
}

// #define STBTT__COMPARE(a,b)  ((a)->y0 < (b)->y0)

pub unsafe fn sort_edges_ins_sort(
    p: *mut Edge,
    n: isize
) {
   let mut j: isize;
   for i in 1..n {
      let t: Edge = *p.offset(i);
      let a: *const Edge = &t;
      j = i;
      while j > 0 {
         let b: *const Edge = p.offset(j-1);
         let c = STBTT__COMPARE!((*a),(*b));
         if !c { break; }
         *p.offset(j) = *p.offset(j-1);
         j -= 1;
      }
      if i != j {
         (*p.offset(j)) = t;
      }
   }
}

pub unsafe fn sort_edges_quicksort(mut p: *mut Edge, mut n: isize)
{
   /* threshhold for transitioning to insertion sort */
   while n > 12 {
      let mut t: Edge;
      let c01: bool;
      let c12: bool;
      let c: bool;
      let m: isize;
      let mut i: isize;
      let mut j: isize;

      /* compute median of three */
      m = n >> 1;
      c01 = STBTT__COMPARE!((*p.offset(0)),(*p.offset(m)));
      c12 = STBTT__COMPARE!((*p.offset(m)),(*p.offset(n-1)));
      /* if 0 >= mid >= end, or 0 < mid < end, then use mid */
      if c01 != c12 {
         /* otherwise, we'll need to swap something else to middle */
         let z: isize;
         c = STBTT__COMPARE!((*p.offset(0)),(*p.offset(n-1)));
         /* 0>mid && mid<n:  0>n => n; 0<n => 0 */
         /* 0<mid && mid>n:  0>n => 0; 0<n => n */
         z = if c == c12 { 0 } else { n-1 };
         t = *p.offset(z);
         *p.offset(z) = *p.offset(m);
         *p.offset(m) = t;
      }
      /* now p[m] is the median-of-three */
      /* swap it to the beginning so it won't move around */
      t = *p.offset(0);
      *p.offset(0) = *p.offset(m);
      *p.offset(m) = t;

      /* partition loop */
      i=1;
      j=n-1;
      loop {
         /* handling of equality is crucial here */
         /* for sentinels & efficiency with duplicates */
         loop {
            if !STBTT__COMPARE!((*p.offset(i)), (*p.offset(0))) { break; }
            i += 1;
         }
         loop {
            if !STBTT__COMPARE!((*p.offset(0)), (*p.offset(j))) { break; }
            j -= 1;
         }
         /* make sure we haven't crossed */
         if i >= j { break; }
         t = *p.offset(i);
         *p.offset(i) = *p.offset(j);
         *p.offset(j) = t;

         i += 1;
         j -= 1;
      }
      /* recurse on smaller side, iterate on larger */
      if j < (n-i) {
         sort_edges_quicksort(p,j);
         p = p.offset(i);
         n = n-i;
      } else {
         sort_edges_quicksort(p.offset(i), n-i);
         n = j;
      }
   }
}

pub unsafe fn sort_edges(p: *mut Edge, n: isize) {
   sort_edges_quicksort(p, n);
   sort_edges_ins_sort(p, n);
}

pub struct Point
{
   x: f32,
   y: f32,
}

unsafe fn rasterize_(
    result: *mut Bitmap,
    pts: *mut Point,
    wcount: *mut isize,
    windings: isize,
    scale_x: f32,
    scale_y: f32,
    shift_x: f32,
    shift_y: f32,
    off_x: isize,
    off_y: isize,
    invert: isize
) {
   let y_scale_inv: f32 = if invert != 0 { -scale_y } else { scale_y };
   let e: *mut Edge;
   let mut n: isize;
   let mut j: isize;
   let mut m: isize;
// TODO: Conditional compilation.
// #if STBTT_RASTERIZER_VERSION == 1
//    int vsubsample = result->h < 8 ? 15 : 5;
// #elif STBTT_RASTERIZER_VERSION == 2
   let vsubsample: isize = 1;
// #else
//   #error "Unrecognized value of STBTT_RASTERIZER_VERSION"
// #endif
   // vsubsample should divide 255 evenly; otherwise we won't reach full opacity

   // now we have to blow out the windings into explicit edge lists
   n = 0;
   for i in 0..windings {
      n = n + *wcount.offset(i);
   }

   e = STBTT_malloc!(size_of::<Edge>() * (n+1) as usize)
        as *mut Edge; // add an extra one as a sentinel
   if e == null_mut() { return };
   n = 0;

   m=0;
   for i in 0..windings {
      let p: *const Point = pts.offset(m);
      m += *wcount.offset(i);
      j = *wcount.offset(i)-1;
      for k in 0..(*wcount.offset(i)) {
         let mut a: isize=k;
         let mut b: isize =j;
         // skip the edge if horizontal
         if (*p.offset(j)).y != (*p.offset(k)).y {
            // add edge from j to k to the list
            (*e.offset(n)).invert = 0;
            if if invert != 0 { (*p.offset(j)).y > (*p.offset(k)).y }
               else { (*p.offset(j)).y < (*p.offset(k)).y } {
               (*e.offset(n)).invert = 1;
               a=j;
               b=k;
            }
            (*e.offset(n)).x0 = (*p.offset(a)).x * scale_x + shift_x;
            (*e.offset(n)).y0 = ((*p.offset(a)).y * y_scale_inv + shift_y) * vsubsample as f32;
            (*e.offset(n)).x1 = (*p.offset(b)).x * scale_x + shift_x;
            (*e.offset(n)).y1 = ((*p.offset(b)).y * y_scale_inv + shift_y) * vsubsample as f32;

            n += 1;
         }
         j = k;
      }
   }

   // now sort the edges by their highest point (should snap to integer, and then by x)
   //STBTT_sort(e, n, sizeof(e[0]), stbtt__edge_compare);
   sort_edges(e, n);

   // now, traverse the scanlines and find the intersections on each scanline, use xor winding rule
   rasterize_sorted_edges(result, e, n, vsubsample, off_x, off_y);

   STBTT_free!(e as *mut c_void);
}

pub unsafe fn add_point(
    points: *mut Point,
    n: isize,
    x: f32,
    y: f32
) {
   if points == null_mut() { return; } // during first pass, it's unallocated
   (*points.offset(n)).x = x;
   (*points.offset(n)).y = y;
}

// tesselate until threshhold p is happy... @TODO warped to compensate for non-linear stretching
pub unsafe fn tesselate_curve(
    points: *mut Point,
    num_points: *mut isize,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    objspace_flatness_squared: f32,
    n: isize
) -> isize {
   // midpoint
   let mx: f32 = (x0 + 2.0*x1 + x2)/4.0;
   let my: f32 = (y0 + 2.0*y1 + y2)/4.0;
   // versus directly drawn line
   let dx: f32 = (x0+x2)/2.0 - mx;
   let dy: f32 = (y0+y2)/2.0 - my;
   if n > 16 { // 65536 segments on one curve better be enough!
      return 1;
   }
   if dx*dx+dy*dy > objspace_flatness_squared { // half-pixel error allowed... need to be smaller if AA
      tesselate_curve(points, num_points, x0,y0, (x0+x1)/2.0,(y0+y1)/2.0, mx,my, objspace_flatness_squared,n+1);
      tesselate_curve(points, num_points, mx,my, (x1+x2)/2.0,(y1+y2)/2.0, x2,y2, objspace_flatness_squared,n+1);
   } else {
      add_point(points, *num_points,x2,y2);
      *num_points = *num_points+1;
   }
   return 1;
}

// returns number of contours
pub unsafe fn flatten_curves(
    vertices: *mut Vertex,
    num_verts: isize,
    objspace_flatness: f32,
    contour_lengths: *mut *mut isize,
    num_contours: *mut isize,
) -> *mut Point {
    let mut points: *mut Point = null_mut();
    let mut num_points: isize =0;

   let objspace_flatness_squared: f32 = objspace_flatness * objspace_flatness;
   let mut n: isize =0;
   let mut start: isize =0;

   // count how many "moves" there are to get the contour count
   for i in 0..num_verts {
      if (*vertices.offset(i)).type_ == Cmd::Move {
         n += 1;
      }
   }

   *num_contours = n;
   if n == 0 { return null_mut(); }

   *contour_lengths = STBTT_malloc!(size_of::<isize>() * n as usize) as *mut isize;

   if *contour_lengths == null_mut() {
      *num_contours = 0;
      return null_mut();
   }

   'error: loop {
   // make two passes through the points so we don't need to realloc
   for pass in 0..2 {
      let mut x: f32=0.0;
      let mut y: f32=0.0;
      if pass == 1 {
         points = STBTT_malloc!(num_points as usize * size_of::<Point>())
            as *mut Point;
         if points == null_mut() {
             break 'error;
         };
      }
      num_points = 0;
      n= -1;
      for i in 0..num_verts {
         match (*vertices.offset(i)).type_ {
            Cmd::Move => {
               // start the next contour
               if n >= 0 {
                  *(*contour_lengths).offset(n) = num_points - start;
               }
               n += 1;
               start = num_points;

               x = (*vertices.offset(i)).x as f32;
               y = (*vertices.offset(i)).y as f32;
               add_point(points, num_points, x,y);
               num_points += 1;
            }
            Cmd::Line => {
               x = (*vertices.offset(i)).x as f32;
               y = (*vertices.offset(i)).y as f32;
               add_point(points, num_points, x, y);
               num_points += 1;
            }
            Cmd::Curve => {
               tesselate_curve(points, &mut num_points, x,y,
                                        (*vertices.offset(i)).cx as f32, (*vertices.offset(i)).cy as f32,
                                        (*vertices.offset(i)).x as f32,  (*vertices.offset(i)).y as f32,
                                        objspace_flatness_squared, 0);
               x = (*vertices.offset(i)).x as f32;
               y = (*vertices.offset(i)).y as f32;
           }
         }
      }
      *(*contour_lengths).offset(n) = num_points - start;
   }
   return points;
   } // 'error

   STBTT_free!(points as *mut c_void);
   STBTT_free!(*contour_lengths as *mut c_void);
   *contour_lengths = null_mut();
   *num_contours = 0;
   return null_mut();
}

// rasterize a shape with quadratic beziers into a bitmap
pub unsafe fn rasterize(
    // 1-channel bitmap to draw into
    result: *mut Bitmap,
    // allowable error of curve in pixels
    flatness_in_pixels: f32,
    // array of vertices defining shape
    vertices: *mut Vertex,
    // number of vertices in above array
    num_verts: isize,
    // scale applied to input vertices
    scale_x: f32,
    scale_y: f32,
    // translation applied to input vertices
    shift_x: f32,
    shift_y: f32,
    // another translation applied to input
    x_off: isize,
    y_off: isize,
    // if non-zero, vertically flip shape
    invert: isize
) {
   let scale: f32 = if scale_x > scale_y { scale_y } else { scale_x };
   let mut winding_count: isize = 0;
   let mut winding_lengths: *mut isize = null_mut();
   let windings: *mut Point = flatten_curves(vertices, num_verts,
       flatness_in_pixels / scale, &mut winding_lengths, &mut winding_count);
   if windings != null_mut() {
      rasterize_(result, windings, winding_lengths, winding_count,
          scale_x, scale_y, shift_x, shift_y, x_off, y_off, invert);
      STBTT_free!(winding_lengths as *mut c_void);
      STBTT_free!(windings as *mut c_void);
   }
}

// frees the bitmap allocated below
pub unsafe fn free_bitmap(bitmap: *mut u8)
{
   STBTT_free!(bitmap as *mut c_void);
}

pub unsafe fn get_glyph_bitmap_subpixel(
    info: *const FontInfo,
    mut scale_x: f32,
    mut scale_y: f32,
    shift_x: f32,
    shift_y: f32,
    glyph: isize,
    width: *mut isize,
    height: *mut isize,
    xoff: *mut isize,
    yoff: *mut isize
) -> *mut u8 {
   let mut ix0: isize = 0;
   let mut iy0: isize = 0;
   let mut ix1: isize = 0;
   let mut iy1: isize = 0;
   let mut vertices: *mut Vertex = null_mut();
   let num_verts: isize = get_glyph_shape(info, glyph, &mut vertices);

   if scale_x == 0.0 { scale_x = scale_y; }
   if scale_y == 0.0 {
      if scale_x == 0.0 { return null_mut(); }
      scale_y = scale_x;
   }

   get_glyph_bitmap_box_subpixel(info, glyph, scale_x, scale_y,
       shift_x, shift_y, &mut ix0,&mut iy0,&mut ix1,&mut iy1);

   // now we get the size
   let mut gbm = Bitmap
   {
       w: (ix1 - ix0),
       h: (iy1 - iy0),
       stride: 0,
       pixels: null_mut(),
   };

   if width != null_mut() { *width  = gbm.w; }
   if height != null_mut() { *height = gbm.h; }
   if xoff != null_mut() { *xoff   = ix0; }
   if yoff != null_mut() { *yoff   = iy0; }

   if gbm.w != 0 && gbm.h != 0 {
      gbm.pixels = STBTT_malloc!((gbm.w * gbm.h) as usize) as *mut u8;
      if gbm.pixels != null_mut() {
         gbm.stride = gbm.w;

         rasterize(&mut gbm, 0.35,
             vertices, num_verts, scale_x, scale_y, shift_x, shift_y, ix0, iy0,
              1);
      }
   }
   STBTT_free!(vertices as *mut c_void);
   return gbm.pixels;
}

// the following functions are equivalent to the above functions, but operate
// on glyph indices instead of Unicode codepoints (for efficiency)

pub unsafe fn get_glyph_bitmap(
    info: *const FontInfo,
    scale_x: f32,
    scale_y: f32,
    glyph: isize,
    width: *mut isize,
    height: *mut isize,
    xoff: *mut isize,
    yoff: *mut isize
) -> *const u8 {
   return get_glyph_bitmap_subpixel(info, scale_x, scale_y,
       0.0, 0.0, glyph, width, height, xoff, yoff);
}

pub unsafe fn make_glyph_bitmap_subpixel(
    info: *const FontInfo,
    output: *mut u8,
    out_w: isize,
    out_h: isize,
    out_stride: isize,
    scale_x: f32,
    scale_y: f32,
    shift_x: f32,
    shift_y: f32,
    glyph: isize
) {
   let mut ix0: isize = 0;
   let mut iy0: isize = 0;
   let mut vertices: *mut Vertex = null_mut();
   let num_verts: isize = get_glyph_shape(info, glyph, &mut vertices);

   get_glyph_bitmap_box_subpixel(info, glyph, scale_x, scale_y,
       shift_x, shift_y, &mut ix0,&mut iy0,null_mut(),null_mut());
   let mut gbm: Bitmap = Bitmap
   {
       w: out_w,
       h: out_h,
       stride: out_stride,
       pixels: output,
   };

   if gbm.w != 0 && gbm.h != 0 {
      rasterize(&mut gbm, 0.35, vertices, num_verts,
          scale_x, scale_y, shift_x, shift_y, ix0,iy0, 1);
   }

   STBTT_free!(vertices as *mut c_void);
}

pub unsafe fn make_glyph_bitmap(
    info: *const FontInfo,
    output: *mut u8,
    out_w: isize,
    out_h: isize,
    out_stride: isize,
    scale_x: f32,
    scale_y: f32,
    glyph: isize
) {
   make_glyph_bitmap_subpixel(info, output, out_w, out_h,
       out_stride, scale_x, scale_y, 0.0,0.0, glyph);
}

// the same as stbtt_GetCodepoitnBitmap, but you can specify a subpixel
// shift for the character
pub unsafe fn get_codepoint_bitmap_subpixel(
    info: *const FontInfo,
    scale_x: f32,
    scale_y: f32,
    shift_x: f32,
    shift_y: f32,
    codepoint: isize,
    width: *mut isize,
    height: *mut isize,
    xoff: *mut isize,
    yoff: *mut isize
) -> *mut u8 {
   return get_glyph_bitmap_subpixel(info, scale_x,
       scale_y,shift_x,shift_y, find_glyph_index(info,codepoint), width,height,xoff,yoff);
}

// same as stbtt_MakeCodepointBitmap, but you can specify a subpixel
// shift for the character
pub unsafe fn make_codepoint_bitmap_subpixel(
    info: *const FontInfo,
    output: *mut u8,
    out_w: isize,
    out_h: isize,
    out_stride: isize,
    scale_x: f32,
    scale_y: f32,
    shift_x: f32,
    shift_y: f32,
    codepoint: isize
) {
   make_glyph_bitmap_subpixel(info, output, out_w, out_h,
       out_stride, scale_x, scale_y, shift_x, shift_y,
       find_glyph_index(info,codepoint));
}

// allocates a large-enough single-channel 8bpp bitmap and renders the
// specified character/glyph at the specified scale into it, with
// antialiasing. 0 is no coverage (transparent), 255 is fully covered (opaque).
// *width & *height are filled out with the width & height of the bitmap,
// which is stored left-to-right, top-to-bottom.
//
// xoff/yoff are the offset it pixel space from the glyph origin to the top-left of the bitmap
pub unsafe fn get_codepoint_bitmap(
    info: *const FontInfo,
    scale_x: f32,
    scale_y: f32,
    codepoint: isize,
    width: *mut isize,
    height: *mut isize,
    xoff: *mut isize,
    yoff: *mut isize
) -> *mut u8 {
   return get_codepoint_bitmap_subpixel(info, scale_x, scale_y,
       0.0,0.0, codepoint, width,height,xoff,yoff);
}

// the same as stbtt_GetCodepointBitmap, but you pass in storage for the bitmap
// in the form of 'output', with row spacing of 'out_stride' bytes. the bitmap
// is clipped to out_w/out_h bytes. Call stbtt_GetCodepointBitmapBox to get the
// width and height and positioning info for it first.
pub unsafe fn make_codepoint_bitmap(
    info: *const FontInfo,
    output: *mut u8,
    out_w: isize,
    out_h: isize,
    out_stride: isize,
    scale_x: f32,
    scale_y: f32,
    codepoint: isize
) {
   make_codepoint_bitmap_subpixel(info, output, out_w, out_h,
       out_stride, scale_x, scale_y, 0.0,0.0, codepoint);
}

//////////////////////////////////////////////////////////////////////////////
//
// bitmap baking
//

// if return is positive, the first unused row of the bitmap
// if return is negative, returns the negative of the number of characters that fit
// if return is 0, no characters fit and no rows were used
// This uses a very crappy packing.
pub unsafe fn bake_font_bitmap(
    data: &[u8], offset: usize,  // font location (use offset=0 for plain .ttf)
    pixel_height: f32,                     // height of font in pixels
    pixels: *mut u8, pw: isize, ph: isize,  // bitmap to be filled in
    first_char: isize, num_chars: isize,          // characters to bake
    chardata: *mut BakedChar
) -> Result<isize, Error> {
    let scale: f32;
    let mut x: isize;
    let mut y: isize;
    let mut bottom_y: isize;
    let f: FontInfo = try!(FontInfo::new_with_offset(data, offset));
   memset(pixels as *mut _ as *mut c_void, 0, (pw*ph) as usize); // background of 0 around pixels
   x=1;
   y=1;
   bottom_y = 1;

   scale = scale_for_pixel_height(&f, pixel_height);

   for i in 0..num_chars {
      let mut advance: isize = 0;
      let mut lsb: isize = 0;
      let mut x0: isize = 0;
      let mut y0: isize = 0;
      let mut x1: isize = 0;
      let mut y1: isize = 0;
      let gw: isize;
      let gh: isize;
      let g: isize = find_glyph_index(&f, first_char + i);
      get_glyph_hmetrics(&f, g, &mut advance, &mut lsb);
      get_glyph_bitmap_box(&f, g, scale,scale, &mut x0,&mut y0,&mut x1,&mut y1);
      gw = x1-x0;
      gh = y1-y0;
      if x + gw + 1 >= pw {
         y = bottom_y;
         x = 1; // advance to next row
      }
      if y + gh + 1 >= ph { // check if it fits vertically AFTER potentially moving to next row
         return Ok(-i);
      }
      STBTT_assert!(x+gw < pw);
      STBTT_assert!(y+gh < ph);
      make_glyph_bitmap(&f, pixels.offset(x+y*pw), gw,gh,pw, scale,scale, g);
      (*chardata.offset(i)).x0 = x as u16;
      (*chardata.offset(i)).y0 = y as u16;
      (*chardata.offset(i)).x1 = (x + gw) as u16;
      (*chardata.offset(i)).y1 = (y + gh) as u16;
      (*chardata.offset(i)).xadvance = scale * advance as f32;
      (*chardata.offset(i)).xoff     = x0 as f32;
      (*chardata.offset(i)).yoff     = y0 as f32;
      x = x + gw + 1;
      if y+gh+1 > bottom_y {
         bottom_y = y+gh+1;
      }
   }
   return Ok(bottom_y);
}

// Call GetBakedQuad with char_index = 'character - first_char', and it
// creates the quad you need to draw and advances the current position.
//
// The coordinate system used assumes y increases downwards.
//
// Characters will extend both above and below the current position;
// see discussion of "BASELINE" above.
//
// It's inefficient; you might want to c&p it and optimize it.
pub unsafe fn get_baked_quad(
    chardata: *mut BakedChar,
    pw: isize,
    ph: isize,
    // character to display
    char_index: isize,
    // pointers to current position in screen pixel space
    xpos: *mut f32,
    ypos: *const f32,
    q: *mut AlignedQuad, // output: quad to draw
    opengl_fillrule: isize
)
{
   let d3d_bias: f32 = if opengl_fillrule != 0 { 0.0 } else { -0.5 };
   let ipw: f32 = 1.0 / pw as f32;
   let iph = 1.0 / ph as f32;
   let b: *mut BakedChar = chardata.offset(char_index);
   let round_x = ifloor((*xpos + (*b).xoff) + 0.5);
   let round_y = ifloor((*ypos + (*b).yoff) + 0.5);

   (*q).x0 = round_x as f32 + d3d_bias;
   (*q).y0 = round_y as f32 + d3d_bias;
   (*q).x1 = round_x as f32 + (*b).x1 as f32 - (*b).x0 as f32 + d3d_bias;
   (*q).y1 = round_y as f32 + (*b).y1 as f32 - (*b).y0 as f32 + d3d_bias;

   (*q).s0 = (*b).x0 as f32 * ipw;
   (*q).t0 = (*b).y0 as f32 * iph;
   (*q).s1 = (*b).x1 as f32 * ipw;
   (*q).t1 = (*b).y1 as f32 * iph;

   *xpos += (*b).xadvance;
}

//////////////////////////////////////////////////////////////////////////////
//
// rectangle packing replacement routines if you don't have stb_rect_pack.h
//

// TODO: Not sure which is the right one, see comments below.
macro_rules! STBTT__NOTUSED {
    ($v:expr) => {}
}

// #ifndef STB_RECT_PACK_VERSION
// #ifdef _MSC_VER
// #define STBTT__NOTUSED(v)  (void)(v)
// #else
// #define STBTT__NOTUSED(v)  (void)sizeof(v)
// #endif

type Coord = isize;

////////////////////////////////////////////////////////////////////////////////////
//                                                                                //
//                                                                                //
// COMPILER WARNING ?!?!?                                                         //
//                                                                                //
//                                                                                //
// if you get a compile warning due to these symbols being defined more than      //
// once, move #include "stb_rect_pack.h" before #include "stb_truetype.h"         //
//                                                                                //
////////////////////////////////////////////////////////////////////////////////////

pub struct Context
{
   width: isize,
   height: isize,
   x: isize,
   y: isize,
   bottom_y: isize,
}

#[allow(dead_code)]
pub struct Node
{
   x: u8,
}

#[allow(dead_code)]
pub struct Rect
{
    x: Coord,
    y: Coord,
    id: isize,
    w: isize,
    h: isize,
    was_packed: isize
}

pub unsafe fn stbrp_init_target(
    con: *mut Context,
    pw: isize,
    ph: isize,
    _nodes: *mut Node,
    _num_nodes: isize
) {
   (*con).width  = pw;
   (*con).height = ph;
   (*con).x = 0;
   (*con).y = 0;
   (*con).bottom_y = 0;
   STBTT__NOTUSED!(nodes);
   STBTT__NOTUSED!(num_nodes);
}

pub unsafe fn stbrp_pack_rects(
    con: *mut Context,
    rects: *mut Rect,
    num_rects: isize
) {
   for i in 0..num_rects {
      if (*con).x + (*rects.offset(i)).w > (*con).width {
         (*con).x = 0;
         (*con).y = (*con).bottom_y;
      }
      if (*con).y + (*rects.offset(i)).h > (*con).height {
         break;
      }
      (*rects.offset(i)).x = (*con).x;
      (*rects.offset(i)).y = (*con).y;
      (*rects.offset(i)).was_packed = 1;
      (*con).x += (*rects.offset(i)).w;
      if (*con).y + (*rects.offset(i)).h > (*con).bottom_y {
         (*con).bottom_y = (*con).y + (*rects.offset(i)).h;
      }
   }
   // TODO: Weird boundary conditions.
   // for (   ; i < num_rects; ++i)
    //  rects[i].was_packed = 0;
}
// #endif

//////////////////////////////////////////////////////////////////////////////
//
// bitmap baking
//
// This is SUPER-AWESOME (tm Ryan Gordon) packing using stb_rect_pack.h. If
// stb_rect_pack.h isn't available, it uses the BakeFontBitmap strategy.

// Initializes a packing context stored in the passed-in PackContext.
// Future calls using this context will pack characters into the bitmap passed
// in here: a 1-channel bitmap that is weight x height. stride_in_bytes is
// the distance from one row to the next (or 0 to mean they are packed tightly
// together). "padding" is the amount of padding to leave between each
// character (normally you want '1' for bitmaps you'll use as textures with
// bilinear filtering).
//
// Returns 0 on failure, 1 on success.
pub unsafe fn pack_begin(
    spc: *mut PackContext,
    pixels: *mut u8,
    pw: isize,
    ph: isize,
    stride_in_bytes: isize,
    padding: isize,
    alloc_context: *const ()
) -> isize
{
   let context: *mut Context = STBTT_malloc!(
       size_of::<Context>()) as *mut Context;
   let num_nodes: isize = pw - padding;
   let nodes: *mut Node = STBTT_malloc!(
       size_of::<Node>() * num_nodes as usize) as *mut Node;

   if context == null_mut() || nodes == null_mut() {
      if context != null_mut() { STBTT_free!(context as *mut c_void); }
      if nodes   != null_mut() { STBTT_free!(nodes as *mut c_void); }
      return 0;
   }

   (*spc).user_allocator_context = alloc_context;
   (*spc).width = pw;
   (*spc).height = ph;
   (*spc).pixels = pixels;
   (*spc).pack_info = context as *mut c_void;
   (*spc).nodes = nodes as *mut c_void;
   (*spc).padding = padding;
   (*spc).stride_in_bytes = if stride_in_bytes != 0 { stride_in_bytes } else { pw };
   (*spc).h_oversample = 1;
   (*spc).v_oversample = 1;

   stbrp_init_target(context, pw-padding, ph-padding, nodes, num_nodes);

   if pixels != null_mut() {
      memset(pixels as *mut c_void, 0, (pw*ph) as usize); // background of 0 around pixels
   }

   return 1;
}

// Cleans up the packing context and frees all memory.
pub unsafe fn pack_end(spc: *mut PackContext)
{
   STBTT_free!((*spc).nodes);
   STBTT_free!((*spc).pack_info);
}

// Oversampling a font increases the quality by allowing higher-quality subpixel
// positioning, and is especially valuable at smaller text sizes.
//
// This function sets the amount of oversampling for all following calls to
// stbtt_PackFontRange(s) or stbtt_PackFontRangesGatherRects for a given
// pack context. The default (no oversampling) is achieved by h_oversample=1
// and v_oversample=1. The total number of pixels required is
// h_oversample*v_oversample larger than the default; for example, 2x2
// oversampling requires 4x the storage of 1x1. For best results, render
// oversampled textures with bilinear filtering. Look at the readme in
// stb/tests/oversample for information about oversampled fonts
//
// To use with PackFontRangesGather etc., you must set it before calls
// call to PackFontRangesGatherRects.
pub unsafe fn pack_set_oversampling(
    spc: *mut PackContext,
    h_oversample: usize,
    v_oversample: usize)
{
   STBTT_assert!(h_oversample <= STBTT_MAX_OVERSAMPLE);
   STBTT_assert!(v_oversample <= STBTT_MAX_OVERSAMPLE);
   if h_oversample <= STBTT_MAX_OVERSAMPLE {
      (*spc).h_oversample = h_oversample;
   }
   if v_oversample <= STBTT_MAX_OVERSAMPLE {
      (*spc).v_oversample = v_oversample;
   }
}

const STBTT__OVER_MASK: usize = (STBTT_MAX_OVERSAMPLE-1);

pub unsafe fn h_prefilter(
    mut pixels: *mut u8,
    w: isize,
    h: isize,
    stride_in_bytes: isize,
    kernel_width: usize
) {
   let mut buffer: [u8; STBTT_MAX_OVERSAMPLE] = [0; STBTT_MAX_OVERSAMPLE];
   let safe_w: isize = w - kernel_width as isize;
   for _ in 0..h {
      let mut total: usize;
      memset(&mut buffer[0] as *mut _ as *mut c_void, 0, kernel_width);

      total = 0;

      // make kernel_width a constant in common cases so compiler can optimize out the divide
      match kernel_width {
        2 => {
            for i in 0..safe_w {
               total = total + *pixels.offset(i) as usize - buffer[i as usize & STBTT__OVER_MASK] as usize;
               buffer[(i as usize +kernel_width) & STBTT__OVER_MASK] = *pixels.offset(i);
               *pixels.offset(i) = (total / 2) as u8;
            }
        }
        3 => {
            for i in 0..safe_w {
               total += *pixels.offset(i) as usize - buffer[i as usize & STBTT__OVER_MASK] as usize;
               buffer[(i as usize +kernel_width) & STBTT__OVER_MASK] = *pixels.offset(i);
               *pixels.offset(i) = (total / 3) as u8;
            }
        }
        4 => {
            for i in 0..safe_w {
               total += *pixels.offset(i) as usize - buffer[i as usize & STBTT__OVER_MASK] as usize;
               buffer[(i as usize +kernel_width) & STBTT__OVER_MASK] = *pixels.offset(i);
               *pixels.offset(i) = (total / 4) as u8;
            }
        }
        5 => {
            for i in 0..safe_w {
               total += *pixels.offset(i) as usize - buffer[i as usize & STBTT__OVER_MASK] as usize;
               buffer[(i as usize +kernel_width) & STBTT__OVER_MASK] = *pixels.offset(i);
               *pixels.offset(i) = (total / 5) as u8;
            }
        }
        _ => {
            for i in 0..safe_w {
               total += *pixels.offset(i) as usize - buffer[i as usize & STBTT__OVER_MASK] as usize;
               buffer[(i as usize +kernel_width) & STBTT__OVER_MASK] = *pixels.offset(i);
               *pixels.offset(i) = (total / kernel_width) as u8;
            }
        }
      }

      for i in safe_w..w {
         STBTT_assert!(*pixels.offset(i) == 0);
         total -= buffer[i as usize & STBTT__OVER_MASK] as usize;
         *pixels.offset(i) = (total / kernel_width) as u8;
      }

      pixels = pixels.offset(stride_in_bytes);
   }
}

pub unsafe fn v_prefilter(
    mut pixels: *mut u8,
    w: isize,
    h: isize,
    stride_in_bytes: isize,
    kernel_width: usize
) {
   let mut buffer: [u8; STBTT_MAX_OVERSAMPLE] = [0; STBTT_MAX_OVERSAMPLE];
   let safe_h: isize = h - kernel_width as isize;
   for _ in 0..w {
      let mut total: usize;
      memset(&mut buffer[0] as *mut _ as *mut c_void, 0, kernel_width);

      total = 0;

      // make kernel_width a constant in common cases so compiler can optimize out the divide
      match kernel_width {
         2 => {
            for i in 0..safe_h {
               total += *pixels.offset(i*stride_in_bytes) as usize - buffer[i as usize & STBTT__OVER_MASK] as usize;
               buffer[(i as usize +kernel_width) & STBTT__OVER_MASK] = *pixels.offset(i*stride_in_bytes);
               *pixels.offset(i*stride_in_bytes) = (total / 2) as u8;
            }
        }
        3 => {
            for i in 0..safe_h {
               total += *pixels.offset(i*stride_in_bytes) as usize - buffer[i as usize & STBTT__OVER_MASK] as usize;
               buffer[(i as usize +kernel_width) & STBTT__OVER_MASK] = *pixels.offset(i*stride_in_bytes);
               *pixels.offset(i*stride_in_bytes) = (total / 3) as u8;
            }
        }
        4 => {
            for i in 0..safe_h {
               total += *pixels.offset(i*stride_in_bytes) as usize - buffer[i as usize & STBTT__OVER_MASK] as usize;
               buffer[(i as usize +kernel_width) & STBTT__OVER_MASK] = *pixels.offset(i*stride_in_bytes);
               *pixels.offset(i*stride_in_bytes) = (total / 4) as u8;
            }
        }
        5 => {
            for i in 0..safe_h {
               total += *pixels.offset(i*stride_in_bytes) as usize - buffer[i as usize & STBTT__OVER_MASK] as usize;
               buffer[(i as usize +kernel_width) & STBTT__OVER_MASK] = *pixels.offset(i*stride_in_bytes);
               *pixels.offset(i*stride_in_bytes) = (total / 5) as u8;
            }
        }
        _ => {
            for i in 0..safe_h {
               total += *pixels.offset(i*stride_in_bytes) as usize - buffer[i as usize & STBTT__OVER_MASK] as usize;
               buffer[(i+kernel_width as isize) as usize & STBTT__OVER_MASK] = *pixels.offset(i*stride_in_bytes);
               *pixels.offset(i*stride_in_bytes) = (total / kernel_width) as u8;
            }
        }
      }

      for i in safe_h..h {
         STBTT_assert!(*pixels.offset(i*stride_in_bytes) == 0);
         total -= buffer[i as usize & STBTT__OVER_MASK] as usize;
         *pixels.offset(i*stride_in_bytes) = (total / kernel_width) as u8;
      }

      pixels = pixels.offset(1);
   }
}

pub fn oversample_shift(oversample: isize) -> f32
{
   if oversample == 0 {
      return 0.0;
   }

   // The prefilter is a box filter of width "oversample",
   // which shifts phase by (oversample - 1)/2 pixels in
   // oversampled space. We want to shift in the opposite
   // direction to counter this.
   return -(oversample - 1) as f32 / (2.0 * oversample as f32);
}

// - stbtt_PackFontRangesGatherRects
// - stbtt_PackFontRangesPackRects
// - stbtt_PackFontRangesRenderIntoRects
//
// Calling these functions in sequence is roughly equivalent to calling
// stbtt_PackFontRanges(). If you more control over the packing of multiple
// fonts, or if you want to pack custom data into a font texture, take a look
// at the source to of stbtt_PackFontRanges() and create a custom version
// using these functions, e.g. call GatherRects multiple times,
// building up a single array of rects, then call PackRects once,
// then call RenderIntoRects repeatedly. This may result in a
// better packing than calling PackFontRanges multiple times
// (or it may not).

// rects array must be big enough to accommodate all characters in the given ranges
pub unsafe fn pack_font_ranges_gather_rects(
    spc: *mut PackContext,
    info: *mut FontInfo,
    ranges: *mut PackRange,
    num_ranges: isize,
    rects: *mut Rect
) -> isize {
   let mut k: isize;

   k=0;
   for i in 0..num_ranges {
      let fh: f32 = (*ranges.offset(i)).font_size;
      let scale: f32 = if fh > 0.0 { scale_for_pixel_height(info, fh) }
        else { scale_for_mapping_em_to_pixels(info, -fh) };
      (*ranges.offset(i)).h_oversample = (*spc).h_oversample as u8;
      (*ranges.offset(i)).v_oversample = (*spc).v_oversample as u8;
      for j in 0..(*ranges.offset(i)).num_chars {
         let mut x0: isize = 0;
         let mut y0: isize = 0;
         let mut x1: isize = 0;
         let mut y1: isize = 0;
         let codepoint: isize = if (*ranges.offset(i)).array_of_unicode_codepoints == null() {
                (*ranges.offset(i)).first_unicode_codepoint_in_range + j
             } else {
                *(*ranges.offset(i)).array_of_unicode_codepoints.offset(j)
             };
         let glyph: isize = find_glyph_index(info, codepoint);
         get_glyph_bitmap_box_subpixel(info,glyph,
                                         scale * (*spc).h_oversample as f32,
                                         scale * (*spc).v_oversample as f32,
                                         0.0,0.0,
                                         &mut x0,&mut y0,&mut x1,&mut y1);
         (*rects.offset(k)).w = (x1-x0 + (*spc).padding as isize + (*spc).h_oversample as isize -1) as Coord;
         (*rects.offset(k)).h = (y1-y0 + (*spc).padding as isize + (*spc).v_oversample as isize -1) as Coord;
         k += 1;
      }
   }

   return k;
}

// rects array must be big enough to accommodate all characters in the given ranges
pub unsafe fn pack_font_ranges_render_into_rects(
    spc: *mut PackContext,
    info: *mut FontInfo,
    ranges: *mut PackRange,
    num_ranges: isize,
    rects: *mut Rect
) -> isize {
   let mut k: isize;
   let mut return_value: isize = 1;

   // save current values
   let old_h_over: isize = (*spc).h_oversample as isize;
   let old_v_over: isize = (*spc).v_oversample as isize;

   k = 0;
   for i in 0..num_ranges {
      let fh: f32 = (*ranges.offset(i)).font_size;
      let scale: f32 = if fh > 0.0 {
            scale_for_pixel_height(info, fh)
          } else { scale_for_mapping_em_to_pixels(info, -fh) };
      let recip_h: f32;
      let recip_v: f32;
      let sub_x: f32;
      let sub_y: f32;
      (*spc).h_oversample = (*ranges.offset(i)).h_oversample as usize;
      (*spc).v_oversample = (*ranges.offset(i)).v_oversample as usize;
      recip_h = 1.0 / (*spc).h_oversample as f32;
      recip_v = 1.0 / (*spc).v_oversample as f32;
      sub_x = oversample_shift((*spc).h_oversample as isize);
      sub_y = oversample_shift((*spc).v_oversample as isize);
      for j in 0..(*ranges.offset(i)).num_chars {
         let r: *mut Rect = rects.offset(k);
         if (*r).was_packed != 0 {
            let bc: *mut PackedChar = (*ranges.offset(i)).chardata_for_range.offset(j);
            let mut advance: isize = 0;
            let mut lsb: isize = 0;
            let mut x0: isize = 0;
            let mut y0: isize = 0;
            let mut x1: isize = 0;
            let mut y1: isize = 0;
            let codepoint: isize =
                if (*ranges.offset(i)).array_of_unicode_codepoints == null() {
                    (*ranges.offset(i)).first_unicode_codepoint_in_range + j
                } else {
                    (*(*ranges.offset(i)).array_of_unicode_codepoints.offset(j))
                };
            let glyph: isize = find_glyph_index(info, codepoint);
            let pad: Coord = (*spc).padding as Coord;

            // pad on left and top
            (*r).x += pad;
            (*r).y += pad;
            (*r).w -= pad;
            (*r).h -= pad;
            get_glyph_hmetrics(info, glyph, &mut advance, &mut lsb);
            get_glyph_bitmap_box(info, glyph,
                                    scale * (*spc).h_oversample as f32,
                                    scale * (*spc).v_oversample as f32,
                                    &mut x0,&mut y0,&mut x1,&mut y1);
            make_glyph_bitmap_subpixel(info,
                                          (*spc).pixels.offset((*r).x + (*r).y*(*spc).stride_in_bytes),
                                          (*r).w - (*spc).h_oversample as isize +1,
                                          (*r).h - (*spc).v_oversample as isize +1,
                                          (*spc).stride_in_bytes,
                                          scale * (*spc).h_oversample as f32,
                                          scale * (*spc).v_oversample as f32,
                                          0.0,0.0,
                                          glyph);

            if (*spc).h_oversample > 1 {
               h_prefilter((*spc).pixels.offset((*r).x + (*r).y*(*spc).stride_in_bytes),
                                  (*r).w, (*r).h, (*spc).stride_in_bytes,
                                  (*spc).h_oversample);
            }

            if (*spc).v_oversample > 1 {
               v_prefilter((*spc).pixels.offset((*r).x + (*r).y*(*spc).stride_in_bytes),
                                  (*r).w, (*r).h, (*spc).stride_in_bytes,
                                  (*spc).v_oversample);
            }

            (*bc).x0 = (*r).x as u16;
            (*bc).y0 = (*r).y as u16;
            (*bc).x1 = ((*r).x + (*r).w) as u16;
            (*bc).y1 = ((*r).y + (*r).h) as u16;
            (*bc).xadvance = scale * advance as f32;
            (*bc).xoff = x0 as f32 * recip_h + sub_x;
            (*bc).yoff = y0 as f32 * recip_v + sub_y;
            (*bc).xoff2 = (x0 + (*r).w) as f32 * recip_h + sub_x;
            (*bc).yoff2 = (y0 + (*r).h) as f32 * recip_v + sub_y;
         } else {
            return_value = 0; // if any fail, report failure
         }

         k += 1;
      }
   }

   // restore original values
   (*spc).h_oversample = old_h_over as usize;
   (*spc).v_oversample = old_v_over as usize;

   return return_value;
}

pub unsafe fn pack_font_ranges_pack_rects(
    spc: *mut PackContext,
    rects: *mut Rect,
    num_rects: isize)
{
   stbrp_pack_rects((*spc).pack_info as *mut Context, rects, num_rects);
}

// Creates character bitmaps from multiple ranges of characters stored in
// ranges. This will usually create a better-packed bitmap than multiple
// calls to stbtt_PackFontRange. Note that you can call this multiple
// times within a single PackBegin/PackEnd.
pub unsafe fn pack_font_ranges(
    spc: *mut PackContext,
    fontdata: &[u8],
    font_index: isize,
    ranges: *mut PackRange,
    num_ranges: isize
) -> Result<isize, Error>
{
   let mut n: isize;
   //stbrp_context *context = (stbrp_context *) spc->pack_info;
   let rects: *mut Rect;

   // flag all characters as NOT packed
   for i in 0..num_ranges {
      for j in 0..(*ranges.offset(i)).num_chars {
         (*(*ranges.offset(i)).chardata_for_range.offset(j)).x0 = 0;
         (*(*ranges.offset(i)).chardata_for_range.offset(j)).y0 = 0;
         (*(*ranges.offset(i)).chardata_for_range.offset(j)).x1 = 0;
         (*(*ranges.offset(i)).chardata_for_range.offset(j)).y1 = 0;
      }
   }

   n = 0;
   for i in 0..num_ranges {
      n += (*ranges.offset(i)).num_chars;
   }

   rects = STBTT_malloc!(size_of::<Rect>() * n as usize)
        as *mut Rect;
   if rects == null_mut() {
      return Ok(0);
   }

   let mut info = try!(FontInfo::new_with_offset(fontdata, get_font_offset_for_index(fontdata.as_ptr(),font_index) as usize));

   n = pack_font_ranges_gather_rects(spc, &mut info, ranges, num_ranges, rects);

   pack_font_ranges_pack_rects(spc, rects, n);

   let return_value = pack_font_ranges_render_into_rects(spc, &mut info, ranges, num_ranges, rects);

   STBTT_free!(rects as *mut c_void);
   return Ok(return_value);
}

// Creates character bitmaps from the font_index'th font found in fontdata (use
// font_index=0 if you don't know what that is). It creates num_chars_in_range
// bitmaps for characters with unicode values starting at first_unicode_char_in_range
// and increasing. Data for how to render them is stored in chardata_for_range;
// pass these to stbtt_GetPackedQuad to get back renderable quads.
//
// font_size is the full height of the character from ascender to descender,
// as computed by stbtt_ScaleForPixelHeight. To use a point size as computed
// by stbtt_ScaleForMappingEmToPixels, wrap the point size in STBTT_POINT_SIZE()
// and pass that result as 'font_size':
//       ...,                  20 , ... // font max minus min y is 20 pixels tall
//       ..., STBTT_POINT_SIZE(20), ... // 'M' is 20 pixels tall
pub unsafe fn pack_font_range(
    spc: *mut PackContext,
    fontdata: &[u8],
    font_index: isize,
    font_size: f32,
    first_unicode_codepoint_in_range: isize,
    num_chars_in_range: isize,
    chardata_for_range: *mut PackedChar
) -> Result<isize, Error> {
   let mut range: PackRange = PackRange {
       first_unicode_codepoint_in_range: first_unicode_codepoint_in_range,
       array_of_unicode_codepoints: null(),
       num_chars: num_chars_in_range,
       chardata_for_range: chardata_for_range,
       font_size: font_size,
       v_oversample: 0,
       h_oversample: 0,
   };
   pack_font_ranges(spc, fontdata, font_index, &mut range, 1)
}

pub unsafe fn get_packed_quad(
    chardata: *mut PackedChar,
    pw: isize,
    ph: isize,
    // character to display
    char_index: isize,
    // pointers to current position in screen pixel space
    xpos: *mut f32,
    ypos: *mut f32,
    // output: quad to draw
    q: *mut AlignedQuad,
    align_to_integer: isize
) {
   let ipw: f32 = 1.0 / pw as f32;
   let iph: f32 = 1.0 / ph as f32;
   let b: *const PackedChar = chardata.offset(char_index);

   if align_to_integer != 0 {
      let x = ((*xpos + (*b).xoff) + 0.5).floor();
      let y = ((*ypos + (*b).yoff) + 0.5).floor();
      (*q).x0 = x;
      (*q).y0 = y;
      (*q).x1 = x + (*b).xoff2 - (*b).xoff;
      (*q).y1 = y + (*b).yoff2 - (*b).yoff;
   } else {
      (*q).x0 = *xpos + (*b).xoff;
      (*q).y0 = *ypos + (*b).yoff;
      (*q).x1 = *xpos + (*b).xoff2;
      (*q).y1 = *ypos + (*b).yoff2;
   }

   (*q).s0 = (*b).x0 as f32 * ipw;
   (*q).t0 = (*b).y0 as f32 * iph;
   (*q).s1 = (*b).x1 as f32 * ipw;
   (*q).t1 = (*b).y1 as f32 * iph;

   *xpos += (*b).xadvance;
}


//////////////////////////////////////////////////////////////////////////////
//
// font name matching -- recommended not to use this
//

// check if a utf8 string contains a prefix which is the utf16 string; if so return length of matching utf8 string
pub unsafe fn compare_utf8_to_utf16_bigendian_prefix(
    s1: *const u8,
    len1: i32,
    mut s2: *const u8,
    mut len2: i32
) -> i32 {
   let mut i: i32 =0;

   // convert utf16 to utf8 and compare the results while converting
   while len2 != 0 {
      let ch: u16 = *s2.offset(0) as u16 *256 + *s2.offset(1) as u16;
      if ch < 0x80 {
         if i >= len1 { return -1; }
         if *s1.offset(i as isize) != ch as u8 { return -1; }
         i += 1;
      } else if ch < 0x800 {
         if i+1 >= len1 { return -1; }
         if *s1.offset(i as isize) != (0xc0 + (ch >> 6)) as u8 { return -1; }
         i += 1;
         if *s1.offset(i as isize) != (0x80 + (ch & 0x3f)) as u8 { return -1; }
         i += 1;
      } else if ch >= 0xd800 && ch < 0xdc00 {
         let c: u32;
         let ch2: u16 = *s2.offset(2) as u16 *256 + *s2.offset(3) as u16;
         if i+3 >= len1 { return -1; }
         c = ((ch - 0xd800) << 10) as u32 + (ch2 - 0xdc00) as u32 + 0x10000;
         if *s1.offset(i as isize) != (0xf0 + (c >> 18)) as u8 { return -1; }
         i += 1;
         if *s1.offset(i as isize) != (0x80 + ((c >> 12) & 0x3f)) as u8 { return -1; }
         i += 1;
         if *s1.offset(i as isize) != (0x80 + ((c >>  6) & 0x3f)) as u8 { return -1; }
         i += 1;
         if *s1.offset(i as isize) != (0x80 + ((c      ) & 0x3f)) as u8 { return -1; }
         i += 1;
         s2 = s2.offset(2); // plus another 2 below
         len2 -= 2;
      } else if ch >= 0xdc00 && ch < 0xe000 {
         return -1;
      } else {
         if i+2 >= len1 { return -1; }
         if *s1.offset(i as isize) != (0xe0 + (ch >> 12)) as u8 { return -1; }
         i += 1;
         if *s1.offset(i as isize) != (0x80 + ((ch >> 6) & 0x3f)) as u8 { return -1; }
         i += 1;
         if *s1.offset(i as isize) != (0x80 + ((ch     ) & 0x3f)) as u8 { return -1; }
         i += 1;
      }
      s2 = s2.offset(2);
      len2 -= 2;
   }
   return i;
}

// returns 1/0 whether the first string interpreted as utf8 is identical to
// the second string interpreted as big-endian utf16... useful for strings from next func
pub unsafe fn compare_utf8_to_utf16_bigendian(
    s1: *const u8,
    len1: isize,
    s2: *const u8,
    len2: isize
) -> isize {
   return (len1 == compare_utf8_to_utf16_bigendian_prefix(
       s1 as *const u8, len1 as i32, s2 as *const u8, len2 as i32) as isize) as isize;
}

// returns the string (which may be big-endian double byte, e.g. for unicode)
// and puts the length in bytes in *length.
//
// some of the values for the IDs are below; for more see the truetype spec:
//     http://developer.apple.com/textfonts/TTRefMan/RM06/Chap6name.html
//     http://www.microsoft.com/typography/otspec/name.htm
//
// returns results in whatever encoding you request... but note that 2-byte encodings
// will be BIG-ENDIAN... use stbtt_CompareUTF8toUTF16_bigendian() to compare
pub unsafe fn get_font_name_string(
    font: *const FontInfo,
    length: *mut isize,
    platform_id: isize,
    encoding_id: isize,
    language_id: isize,
    name_id: isize
) -> *const u8 {
   let count: i32;
   let string_offset: i32;
   let fc: *const u8 = (*font).data.as_ptr();
   let offset: u32 = (*font).fontstart as u32;
   let nm: u32 = find_table(fc, offset, CString::new("name").unwrap().as_ptr());
   if nm == 0 { return null(); }

   count = ttUSHORT!(fc.offset(nm as isize +2)) as i32;
   string_offset = nm as i32 + ttUSHORT!(fc.offset(nm as isize +4)) as i32;
   for i in 0..count as u32 {
      let loc: u32 = nm + 6 + 12 * i;
      if platform_id == ttUSHORT!(fc.offset(loc as isize +0)) as isize && encoding_id == ttUSHORT!(fc.offset(loc as isize +2)) as isize
          && language_id == ttUSHORT!(fc.offset(loc as isize +4)) as isize && name_id == ttUSHORT!(fc.offset(loc as isize +6)) as isize {
         *length = ttUSHORT!(fc.offset(loc as isize +8)) as isize;
         return (fc.offset(string_offset as isize +ttUSHORT!(fc.offset(loc as isize +10)) as isize)) as *const u8;
      }
   }
   return null();
}

pub unsafe fn matchpair(
    fc: *mut u8,
    nm: u32,
    name: *mut u8,
    nlen: i32,
    target_id: i32,
    next_id: i32
) -> isize {
    let count: u32 = ttUSHORT!(fc.offset(nm as isize +2)) as u32;
    let string_offset: i32 = nm as i32 + ttUSHORT!(fc.offset(nm as isize +4)) as i32;

   for i in 0..count as u32 {
      let loc: u32 = nm + 6 + 12 * i;
      let id: i32 = ttUSHORT!(fc.offset(loc as isize +6)) as i32;
      if id == target_id {
         // find the encoding
         let platform: i32 = ttUSHORT!(fc.offset(loc as isize +0)) as i32;
         let encoding: i32 = ttUSHORT!(fc.offset(loc as isize +2)) as i32;
         let language: i32 = ttUSHORT!(fc.offset(loc as isize +4)) as i32;

         // is this a Unicode encoding?
         if platform == 0 || (platform == 3 && encoding == 1) || (platform == 3 && encoding == 10) {
            let mut slen: i32 = ttUSHORT!(fc.offset(loc as isize +8)) as i32;
            let mut off: i32 = ttUSHORT!(fc.offset(loc as isize +10)) as i32;

            // check if there's a prefix match
            let mut matchlen: i32 = compare_utf8_to_utf16_bigendian_prefix(
                name, nlen, fc.offset(string_offset as isize + off as isize),slen);
            if matchlen >= 0 {
               // check for target_id+1 immediately following, with same encoding & language
               if i+1 < count && ttUSHORT!(fc.offset(loc as isize +12+6)) == next_id as u16
               && ttUSHORT!(fc.offset(loc as isize +12)) == platform as u16
               && ttUSHORT!(fc.offset(loc as isize +12+2)) == encoding as u16
               && ttUSHORT!(fc.offset(loc as isize +12+4)) == language as u16 {
                  slen = ttUSHORT!(fc.offset(loc as isize +12+8)) as i32;
                  off = ttUSHORT!(fc.offset(loc as isize +12+10)) as i32;
                  if slen == 0 {
                     if matchlen == nlen {
                        return 1;
                     }
                  } else if matchlen < nlen && *name.offset(matchlen as isize) == ' ' as u8 {
                     matchlen += 1;
                     if compare_utf8_to_utf16_bigendian(
                         (name.offset(matchlen as isize)) as *mut u8, (nlen - matchlen) as isize,
                         (fc.offset(string_offset as isize + off as isize)) as *mut u8,slen as isize) != 0 {
                        return 1;
                    }
                  }
               } else {
                  // if nothing immediately following
                  if matchlen == nlen {
                     return 1;
                  }
               }
            }
         }

         // @TODO handle other encodings
      }
   }
   return 0;
}

pub unsafe fn matches(
    fc: *mut u8,
    offset: u32,
    name: *mut u8,
    flags: i32
) -> isize {
    let nlen: i32 = STBTT_strlen(name as *mut c_char) as i32;
    let nm: u32;
    let hd: u32;
   if isfont(fc.offset(offset as isize)) == 0 { return 0; }

   // check italics/bold/underline flags in macStyle...
   if flags != 0 {
      hd = find_table(fc, offset, CString::new("head").unwrap().as_ptr());
      if (ttUSHORT!(fc.offset(hd as isize + 44)) & 7) != (flags as u16 & 7) { return 0; }
   }

   nm = find_table(fc, offset, CString::new("name").unwrap().as_ptr());
   if nm == 0 { return 0; }

   if flags != 0 {
      // if we checked the macStyle flags, then just check the family and ignore the subfamily
      if matchpair(fc, nm, name, nlen, 16, -1) != 0 { return 1; }
      if matchpair(fc, nm, name, nlen,  1, -1) != 0 { return 1; }
      if matchpair(fc, nm, name, nlen,  3, -1) != 0 { return 1; }
   } else {
      if matchpair(fc, nm, name, nlen, 16, 17) != 0 { return 1; }
      if matchpair(fc, nm, name, nlen,  1,  2) != 0 { return 1; }
      if matchpair(fc, nm, name, nlen,  3, -1) != 0 { return 1; }
   }

   return 0;
}

// returns the offset (not index) of the font that matches, or -1 if none
//   if you use STBTT_MACSTYLE_DONTCARE, use a font name like "Arial Bold".
//   if you use any other flag, use a font name like "Arial"; this checks
//     the 'macStyle' header field; i don't know if fonts set this consistently
pub unsafe fn find_matching_font(
    font_collection: *const u8,
    name_utf8: *const u8,
    flags: i32
) -> i32 {
   for i in 0.. {
      let off: i32 = get_font_offset_for_index(font_collection, i);
      if off < 0 { return off; }
      if matches(font_collection as *mut u8,
            off as u32, name_utf8 as *mut u8, flags) != 0 {
         return off;
      }
   }
   return 0;
}

// #endif // STB_TRUETYPE_IMPLEMENTATION


// FULL VERSION HISTORY
//
//   1.08 (2015-09-13) document stbtt_Rasterize(); fixes for vertical & horizontal edges
//   1.07 (2015-08-01) allow PackFontRanges to accept arrays of sparse codepoints;
//                     allow PackFontRanges to pack and render in separate phases;
//                     fix stbtt_GetFontOFfsetForIndex (never worked for non-0 input?);
//                     fixed an assert() bug in the new rasterizer
//                     replace assert() with STBTT_assert() in new rasterizer
//   1.06 (2015-07-14) performance improvements (~35% faster on x86 and x64 on test machine)
//                     also more precise AA rasterizer, except if shapes overlap
//                     remove need for STBTT_sort
//   1.05 (2015-04-15) fix misplaced definitions for STBTT_STATIC
//   1.04 (2015-04-15) typo in example
//   1.03 (2015-04-12) STBTT_STATIC, fix memory leak in new packing, various fixes
//   1.02 (2014-12-10) fix various warnings & compile issues w/ stb_rect_pack, C++
//   1.01 (2014-12-08) fix subpixel position when oversampling to exactly match
//                        non-oversampled; STBTT_POINT_SIZE for packed case only
//   1.00 (2014-12-06) add new PackBegin etc. API, w/ support for oversampling
//   0.99 (2014-09-18) fix multiple bugs with subpixel rendering (ryg)
//   0.9  (2014-08-07) support certain mac/iOS fonts without an MS platform_id
//   0.8b (2014-07-07) fix a warning
//   0.8  (2014-05-25) fix a few more warnings
//   0.7  (2013-09-25) bugfix: subpixel glyph bug fixed in 0.5 had come back
//   0.6c (2012-07-24) improve documentation
//   0.6b (2012-07-20) fix a few more warnings
//   0.6  (2012-07-17) fix warnings; added stbtt_ScaleForMappingEmToPixels,
//                        stbtt_GetFontBoundingBox, stbtt_IsGlyphEmpty
//   0.5  (2011-12-09) bugfixes:
//                        subpixel glyph renderer computed wrong bounding box
//                        first vertex of shape can be off-curve (FreeSans)
//   0.4b (2011-12-03) fixed an error in the font baking example
//   0.4  (2011-12-01) kerning, subpixel rendering (tor)
//                    bugfixes for:
//                        codepoint-to-glyph conversion using table fmt=12
//                        codepoint-to-glyph conversion using table fmt=4
//                        stbtt_GetBakedQuad with non-square texture (Zer)
//                    updated Hello World! sample to use kerning and subpixel
//                    fixed some warnings
//   0.3  (2009-06-24) cmap fmt=12, compound shapes (MM)
//                    userdata, malloc-from-userdata, non-zero fill (stb)
//   0.2  (2009-03-11) Fix unsigned/signed char warnings
//   0.1  (2009-03-09) First public release
//
