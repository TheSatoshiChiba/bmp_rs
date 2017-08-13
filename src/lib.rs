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
    /// Returns a default color with values ( r: 0, g: 0, b: 0, a: 255 ).
    fn default() -> Color {
        Color { r: 0, g: 0, b: 0, a: 255 }
    }
}

impl fmt::Display for Color {
    fn fmt( &self, f : &mut fmt::Formatter ) -> fmt::Result {
        write!( f, "({}, {}, {}, {})", self.r, self.g, self.b, self.a )
    }
}

/// The bitmap type that is able to hold a vector of 32-bit color values.
#[derive( Debug )]
pub struct Bitmap {
    width : i32,
    height : i32,
    data : Vec<Color>,
}

impl Bitmap {
    /// Creates a new bitmap filled with black.
    pub fn new( width : i32, height : i32 ) -> Bitmap {
        Bitmap::with_color( width, height, Color::default() )
    }

    /// Creates a new bitmap filled with the given color.
    pub fn with_color( width : i32, height : i32, color : Color ) -> Bitmap {
        let len = ( width * height ) as usize;
        let mut data = Vec::<Color>::with_capacity( len );

        for _ in 0..data.capacity() {
            data.push( Color { r: color.r, g: color.g, b: color.b, a: color.a } );
        }

        Bitmap { width, height, data: data }
    }
}

#[cfg( test )]
mod tests {
    use std::fmt;

    use super::Color;
    use super::Bitmap;

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

    #[test]
    fn bitmap_debug_format_test() {
        let s = fmt::format(
            format_args!(
                "{:?}",
                Bitmap {
                    width: 100,
                    height: 200,
                    data: Vec::new() } ) );

        assert_eq!( "Bitmap { width: 100, height: 200, data: [] }", s );
    }

    #[test]
    fn bitmap_new_test() {
        let bmp = Bitmap::new( 100, 200 );

        assert_eq!( 100, bmp.width );
        assert_eq!( 200, bmp.height );

        let len = ( bmp.width * bmp.height ) as usize;

        assert_eq!( len, bmp.data.len() );

        for i in 0..len {
            assert_eq!( Color::default(), bmp.data[i] );
        }
    }

    #[test]
    fn bitmap_with_color_test() {
        let color = Color { r: 0, g: 55, b: 155, a: 255 };
        let bmp = Bitmap::with_color( 100, 200, Color { r: 0, g: 55, b: 155, a: 255 } );

        assert_eq!( 100, bmp.width );
        assert_eq!( 200, bmp.height );

        let len = ( bmp.width * bmp.height ) as usize;

        assert_eq!( len, bmp.data.len() );

        for i in 0..len {
            assert_eq!( color, bmp.data[i] );
        }
    }
}
