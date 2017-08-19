//! # bmp_rs
//!
//! A bitmap reader and writer.
mod bmp;

use std::fmt;
use std::io::{
    Read,
    Result,
};

/// The color type that is able to hold 32-bit color values.
#[derive( Debug, Eq, PartialEq, Copy, Clone )]
pub struct Color {
    pub r : u8,
    pub g : u8,
    pub b : u8,
    pub a : u8,
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
        let data = vec![color; len];

        Bitmap { width, height, data: data }
    }

    fn read( input : &mut Read ) -> Result<Bitmap> {
        use bmp::{
            ReadBmpExt,
        };

        let _ = input.read_file_header()?;
        let _ = input.read_bitmap_header()?;

        Ok( Bitmap::new( 0, 0 ) )
    }
    /*pub fn from_reader( reader : &mut Read ) -> Result<Bitmap> {
        // Read File header


        /







        // Read color palette entries
        // TODO: Verify the actual amount of color palette entries
        let entry_size = ( 1 << bpp ) as usize;
        let mut palette = Vec::with_capacity( entry_size );
        let mut entry : [u8; 3] = [0; 3];

        for _ in 0..entry_size {
            reader.read_exact( &mut entry )?;
            palette.push( Color {
                b : entry[0],
                g : entry[1],
                r : entry[2],
                a : 255 } );
        }

        // Read image data
        let size = ( width * height ) as usize;
        let data = vec![Color::default(); size];

        // TODO: Fix bitmap reading issue
        for y in 0..height {
            for x in 0..width {

            }

        }

        Ok( Bitmap {
            width : width as i32,
            height : height as i32,
            data } )
    }*/
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
    fn color_copy_test() {
        let mut a = Color::default();
        let mut b = a;

        a.r = 255;

        assert_ne!( a.r, b.r );
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
        let bmp = Bitmap::with_color( 100, 200, color );

        assert_eq!( 100, bmp.width );
        assert_eq!( 200, bmp.height );

        let len = ( bmp.width * bmp.height ) as usize;

        assert_eq!( len, bmp.data.len() );

        for i in 0..len {
            assert_eq!( color, bmp.data[i] );
        }
    }
}
