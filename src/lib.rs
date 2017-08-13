//! # bmp_rs
//!
//! A bitmap reader and writer.
use std::io::Read;
use std::io::Result;
use std::fmt;

/// The color type that is able to hold 32-bit color values.
#[derive( Debug, Eq, PartialEq )]
pub struct Color {
    r : u8,
    g : u8,
    b : u8,
    a : u8,
}

impl Default for Color {
    /// Returns a default color with values ( r: 0, g: 0, b: 0, a: 255 )
    fn default() -> Color {
        Color { r: 0, g: 0, b: 0, a: 255 }
    }
}

impl fmt::Display for Color {
    fn fmt( &self, f : &mut fmt::Formatter ) -> fmt::Result {
        write!( f, "({}, {}, {}, {})", self.r, self.g, self.b, self.a )
    }
}

// #[derive( Debug )]
// pub struct Bitmap {
//     width : i16,
//     height : i16,
//     bpp : u16,
//     data : Vec<Color>,
// }

// impl Bitmap {
//     pub fn new( width : i16, height : i16, bpp : u16 ) -> Bitmap {
//         let len = ( width * height ) as usize;
//         Bitmap { width, height, bpp, data: Vec::with_capacity( len ) }
//     }

//     pub fn from( input : &mut Read ) -> Result<Bitmap> {
//         Ok ( Bitmap::new( 0, 0, 1 ) )
//     }
// }

#[cfg( test )]
mod tests {
    use std::fmt;

    use super::Color;

    #[test]
    fn color_default_test() {
        assert_eq!( Color { r: 0, g: 0, b: 0, a: 255 }, Color::default() );
    }

    #[test]
    fn color_equality_test() {
        let a1 = Color { r: 0, g: 0, b: 0, a: 0 };
        let a2 = Color { r: 0, g: 0, b: 0, a: 0 };
        let b1 = Color { r: 255, g: 0, b: 0, a: 0 };
        let b2 = Color { r: 255, g: 0, b: 0, a: 0 };
        let c1 = Color { r: 0, g: 255, b: 0, a: 0 };
        let c2 = Color { r: 0, g: 255, b: 0, a: 0 };
        let d1 = Color { r: 0, g: 0, b: 255, a: 0 };
        let d2 = Color { r: 0, g: 0, b: 255, a: 0 };
        let e1 = Color { r: 0, g: 0, b: 0, a: 255 };
        let e2 = Color { r: 0, g: 0, b: 0, a: 255 };

        assert_eq!( a1, a1 );
        assert_eq!( a1, a2 );
        assert_ne!( a1, b1 );
        assert_ne!( a1, c1 );
        assert_ne!( a1, d1 );
        assert_ne!( a1, e1 );

        assert_eq!( b1, b1 );
        assert_eq!( b1, b2 );
        assert_ne!( b1, a1 );
        assert_ne!( b1, c1 );
        assert_ne!( b1, d1 );
        assert_ne!( b1, e1 );

        assert_eq!( c1, c1 );
        assert_eq!( c1, c2 );
        assert_ne!( c1, a1 );
        assert_ne!( c1, b1 );
        assert_ne!( c1, d1 );
        assert_ne!( c1, e1 );

        assert_eq!( d1, d1 );
        assert_eq!( d1, d2 );
        assert_ne!( d1, a1 );
        assert_ne!( d1, b1 );
        assert_ne!( d1, c1 );
        assert_ne!( d1, e1 );

        assert_eq!( e1, e1 );
        assert_eq!( e1, e2 );
        assert_ne!( e1, a1 );
        assert_ne!( e1, b1 );
        assert_ne!( e1, c1 );
        assert_ne!( e1, d1 );
    }

    #[test]
    fn color_debug_format_test() {
        let s = fmt::format( format_args!( "{:?}", Color { r: 0, g: 55, b: 155, a: 255 } ) );
        assert_eq!( "Color { r: 0, g: 55, b: 155, a: 255 }", s );
    }

    #[test]
    fn color_display_test() {
        let s = fmt::format( format_args!( "{}", Color { r: 0, g: 55, b: 155, a: 255 } ) );
        assert_eq!( "(0, 55, 155, 255)", s );
    }
}
