//! # bmp_rs
//!
//! A bmp (bitmap) file decoder.
//!
//! ## Example
//!
//! ```
//! use std::fs::File;
//! use bmp_rs::{
//!     Result,
//!     BMPDecorder,
//! };
//!
//! struct ImageDecoder {
//!     // your builder type that is able to construct an image
//! }
//!
//!
//! impl BMPDecoder for ImageDecoder {
//!     type TResult = MyImageType; // Your image type
//!
//!     fn set_size( &mut self, width: u32, height: u32 ) {
//!         // Set image size
//!     }
//!
//!     fn set_pixel( &mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8 ) {
//!         // Set a specific pixel within that image to the given color
//!     }
//!
//!     fn build( &mut self ) -> Result<Self::TResult> {
//!         // Build and return your final image
//!     }
//! }
//!
//! fn main() {
//!     let mut file = File::open( "image.bmp" ).unwrap();
//!     let image = bmp_rs::decode( &mut file, YourImageDecoderInstance );
//!     // Do something with your image
//! }
//! ```
//!
extern crate byteorder;

use std::io;
use std::error;
use std::fmt;

use byteorder::{
    ReadBytesExt,
    LittleEndian,
};


#[derive( Debug )]
pub enum DecodingError {
    IOError( std::io::Error ),
}

pub type Result<T> = std::result::Result<T, DecodingError>;

impl DecodingError {
    fn new_io( message : &str ) -> DecodingError {
        DecodingError::IOError(
            io::Error::new( io::ErrorKind::InvalidData, message ) )
    }
}

impl fmt::Display for DecodingError {
    fn fmt( &self, formatter: &mut fmt::Formatter ) -> fmt::Result {
        match *self {
            DecodingError::IOError( ref error )
                => write!( formatter, "IO error: {}", *error ),
        }
    }
}

impl error::Error for DecodingError {
    fn description( &self ) -> &str {
        match *self {
            DecodingError::IOError( ref error )
                => error.description(),
        }
    }

    fn cause( &self ) -> Option<&error::Error> {
        match *self {
            DecodingError::IOError( ref error ) => Some( error ),
        }
    }
}

impl From<std::io::Error> for DecodingError {
    fn from( error: std::io::Error ) -> Self {
        DecodingError::IOError( error )
    }
}

#[derive( Clone, Copy )]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

pub trait BMPDecoder {
    type TResult;

    fn set_size( &mut self, width: u32, height: u32 );
    fn set_pixel( &mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8 );
    fn build( &mut self ) -> Result<Self::TResult>;
}

pub fn decode<TDecoder: BMPDecoder>(
    input: &mut io::Read, mut decoder: TDecoder ) -> Result<TDecoder::TResult> {

    // Read file header
    if input.read_u8()? != 0x42 || input.read_u8()? != 0x4D {
        return Err( DecodingError::new_io( "Invalid bitmap header." ) );
    }

    let _ = input.read_u32::<LittleEndian>()?; // File size
    let _ = input.read_u32::<LittleEndian>()?; // Reserved fields
    let offset = input.read_u32::<LittleEndian>()?; // Offset to bitmap data

    // Read BMP Version 2 header
    let header_size = input.read_u32::<LittleEndian>()?;
    if header_size != 12 { // Header size
        return Err( DecodingError::new_io(
            "Invalid bitmap header. Only BMP V2 is supported at this time." ) );
    }

    let w = input.read_i16::<LittleEndian>()?;
    let h = input.read_i16::<LittleEndian>()?;

    let width = w.checked_abs().ok_or(
        DecodingError::new_io( "Invalid bitmap width." ) )? as i32;

    let height = h.checked_abs().ok_or(
        DecodingError::new_io( "Invalid bitmap height." ) )? as i32;

    let direction = h.signum() as i8;

    if input.read_u16::<LittleEndian>()? != 1 { // Color planes
        return Err( DecodingError::new_io( "Invalid bitmap color plane." ) );
    }

    let bpp = match input.read_u16::<LittleEndian>()? {
        v @ 1
        | v @ 4
        | v @ 8
        | v @ 24 => v as i32,
        _ => return Err(
            DecodingError::new_io( "Invalid bitmap bits per pixel." ) ),
    };

    // Read palette
    let palette_size = ( ( offset - 14 - header_size ) / 3 ) as usize;
    let mut palette = Vec::with_capacity( palette_size );
    let mut palette_buffer : [u8; 3] = [0; 3];

    for _ in 0..palette_size {
        input.read_exact( &mut palette_buffer )?;

        palette.push( Color {
            r : palette_buffer[2],
            g : palette_buffer[1],
            b : palette_buffer[0],
            a : 255 } );
    }

    decoder.set_size( width as u32, height as u32 );
    // let pixel_size = ( width * height ) as usize;
    // let mut pixels = vec![ Color { r : 0, g : 0, b : 0, a : 255 }; pixel_size];

    let line_width = ( ( width * bpp + 31 ) / 32 ) * 4;
    let mut line_buffer = vec![0 as u8; line_width as usize];

    for y in 0..height {
        input.read_exact( &mut line_buffer )?; // read whole line

        let y = match direction {
            -1 => y,
            1 => height - y - 1,
            _ => return Err( DecodingError::new_io( "Invalid direction!" ) ),
        };

        let mut index = 0;
        let mut range = 0..line_width;
        loop {
            match range.next() {
                Some( mut x ) => {
                    match bpp {
                        1 => {
                            for i in (0..8).rev() {
                                let c = palette[((line_buffer[ x as usize ] >> i ) & 0x01) as usize];
                                decoder.set_pixel( index as u32, y as u32, c.r, c.g, c.b, c.a);

                                index += 1;

                                if i < 7 {
                                    if index >= width as usize {
                                        break;
                                    }
                                }
                            }
                        },
                        4 => {
                            let c1 = palette[((line_buffer[ x as usize ] >> 4 ) & 0x0F) as usize];
                            decoder.set_pixel( index as u32, y as u32, c1.r, c1.g, c1.b, c1.a);

                            index += 1;

                            if index >= width as usize {
                                break;
                            }

                            let c2 = palette[(line_buffer[ x as usize ] & 0x0F) as usize];
                            decoder.set_pixel( index as u32, y as u32, c2.r, c2.g, c2.b, c2.a);

                            index += 1;
                        },
                        8 => {
                            let c = palette[line_buffer[ x as usize ] as usize];
                            decoder.set_pixel( index as u32, y as u32, c.r, c.g, c.b, c.a);

                            index += 1;
                        },
                        24 => {
                            let b = line_buffer[ x as usize ];
                            if let Some( z ) = range.next() {
                                x = z;
                            } else { break }

                            let g = line_buffer[ x as usize ];
                            if let Some( z ) = range.next() {
                                x = z;
                            } else { break }

                            let r = line_buffer[ x as usize ];

                            decoder.set_pixel( index as u32, y as u32, r, g, b, 255);

                            index += 1;
                        },
                        _=> return Err(
                            DecodingError::new_io( "Invalid bitmap bits per pixel." ) ),
                    }
                },
                None => break,
            }

            if index >= width as usize {
                break;
            }
        }
    }

    decoder.build()
}

#[cfg( test )]
mod tests {
    use std::io;
    use std::error::Error;

    use super::DecodingError;

    #[test]
    fn decoding_error_new_io_test() {
        let error = DecodingError::new_io( "This is an error!" );
        let io_error = io::Error::new(
            io::ErrorKind::InvalidData, "This is an error!" );

        match error {
            DecodingError::IOError( error ) => {
                assert_eq!(
                    error.description(),
                    io_error.description() );
                assert_eq!(
                    error.kind(),
                    io_error.kind() );
            },
            _ => panic!( "No IO Error" ),
        }
    }

    #[test]
    fn decoding_error_fmt_test() {
        let error = DecodingError::new_io( "FooBar!" );

        assert_eq!( "IO error: FooBar!", format!( "{}", error ) );
    }

    #[test]
    fn decoding_error_from_io_error_test() {
        let io_error = io::Error::new(
            io::ErrorKind::InvalidData, "This is an error!" );

        let error : DecodingError = io_error.into();

        assert!( match error {
            DecodingError::IOError( error ) => true,
            _ => false,
        } );
    }
}
