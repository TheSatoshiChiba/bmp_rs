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
    IOError( io::Error ),
}

pub type Result<TResult> = std::result::Result<TResult, DecodingError>;

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
            DecodingError::IOError( ref error ) => error.description(),
        }
    }

    fn cause( &self ) -> Option<&error::Error> {
        match *self {
            DecodingError::IOError( ref error ) => Some( error ),
        }
    }
}

impl From<io::Error> for DecodingError {
    fn from( error: io::Error ) -> Self {
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

const MSVERSION2_SIZE: isize = 12;
const MSVERSION3_SIZE: isize = 40;
const MSVERSION4_SIZE: isize = 108;
const MSVERSION5_SIZE: isize = 124;

#[derive( PartialEq, Eq, Clone, Copy )]
enum BMPVersion {
    Microsoft2 = MSVERSION2_SIZE,
    Microsoft3 = MSVERSION3_SIZE,
    Microsoft4 = MSVERSION4_SIZE,
    Microsoft5 = MSVERSION5_SIZE,
}

impl BMPVersion { // TODO: Replace with TryFrom when available.
    fn from_isize( size: isize ) -> Result<BMPVersion> {
        match size {
            MSVERSION2_SIZE => Ok( BMPVersion::Microsoft2 ),
            MSVERSION3_SIZE => Ok( BMPVersion::Microsoft3 ),
            MSVERSION4_SIZE => Ok( BMPVersion::Microsoft4 ),
            MSVERSION5_SIZE => Ok( BMPVersion::Microsoft5 ),
            _ => Err( DecodingError::new_io(
                    &format!( "Invalid bitmap header {},", size ) ) ),
        }
    }
}

struct BMPCore {
    width: u32,
    height: u32,
    bpp: u32,
    planes: u16,
    bottom_up: bool,
}

impl BMPCore {
    fn from_buffer( buf: &[u8], version: BMPVersion ) -> Result<BMPCore> {
        let mut cursor = io::Cursor::new( buf );

        let ( width, height ) =
            match version {
                BMPVersion::Microsoft2 => {
                    let mut dimension: [i16; 2] = [0; 2];
                    cursor.read_i16_into::<LittleEndian>( &mut dimension )?;

                    ( dimension[0] as i32, dimension[1] as i32 )
                },
                _ => {
                    let mut dimension: [i32; 2] = [0; 2];
                    cursor.read_i32_into::<LittleEndian>( &mut dimension )?;

                    ( dimension[0], dimension[1] )
                },
            };

        let bottom_up = if height.signum() == 1 { true } else { false };
        let width = width.checked_abs()
            .ok_or( DecodingError::new_io( "Invalid width." ) )? as u32;
        let height = height.checked_abs()
            .ok_or( DecodingError::new_io( "Invalid height." ) )? as u32;

        let planes = cursor.read_u16::<LittleEndian>()?;
        if planes != 1 {
            return Err( DecodingError::new_io(
                &format!( "Invalid number of color planes {}.", planes ) ) );
        }

        let bpp = cursor.read_u16::<LittleEndian>()? as u32;
        match bpp {
            1 | 4 | 8 | 16 | 24 | 32 => {
                if version == BMPVersion::Microsoft2
                    && ( bpp == 16 || bpp == 32 ) {

                    return Err( DecodingError::new_io(
                        &format!( "Invalid bits per pixel {}.", bpp ) ) );
                }
            },
            _ => return Err( DecodingError::new_io(
                &format!( "Invalid bits per pixel {}.", bpp ) ) ),
        }

        Ok( BMPCore { width, height, bpp, planes, bottom_up } )
    }
}

struct BMPPalette {
    colors: Vec<Color>,
}

impl BMPPalette {
    fn from_buffer( buf: &[u8], size: usize ) -> Result<BMPPalette> {
        let iter = buf.chunks( 3 );
        let mut colors = Vec::with_capacity( size );

        for x in iter {
            colors.push(
                Color {
                    b: x[0],
                    g: x[1],
                    r: x[2],
                    a: 255,
                } );
        }
        Ok( BMPPalette { colors } )
    }
}

struct BMPHeader {
    version: BMPVersion,
    core: BMPCore,
    palette: Option<BMPPalette>,
}

impl BMPHeader {
    fn from_reader( input: &mut io::Read ) -> Result<BMPHeader> {
        let version = BMPVersion::from_isize(
            input.read_u32::<LittleEndian>()? as isize )?;

        // Read core header
        let mut buffer = vec![0; ( version as usize ) - 4];

        input.read_exact( &mut buffer )?;

        let core = BMPCore::from_buffer( &buffer, version )?;

        // Read palette
        let palette_size = 1 << core.bpp;
        // TODO: Check if the size is sensible with the bitmap offset

        let palette = if palette_size > 0 {
            match core.bpp {
                1 | 4 | 8 => {
                    let palette_size = palette_size as usize;
                    let mut buffer = vec![0; palette_size * 3];
                    input.read_exact( &mut buffer )?;

                    Some( BMPPalette::from_buffer( &buffer, palette_size )? )
                },
                _ => return Err( DecodingError::new_io(
                    &format!( "Unexpected color palette of size {}.", palette_size ) ) ),
            }
        } else {
            None
        };

        Ok ( BMPHeader { version, core, palette } )
    }
}

fn decode_1bpp<TDecoder: BMPDecoder>(
    y: u32, width: u32, buf: &[u8], palette: &[Color], decoder: &mut TDecoder ) {

    let mut x: u32 = 0;

    for byte in buf {
        for bit in (0..8).rev() {

            let color: Color = palette[ ( ( *byte >> bit ) & 0x01 ) as usize ];
            decoder.set_pixel( x, y, color.r, color.g, color.b, color.a );

            x += 1;
            if x >= width {
                return;
            }
        }
    }
}

fn decode_4bpp<TDecoder: BMPDecoder>(
    y: u32, width: u32, buf: &[u8], palette: &[Color], decoder: &mut TDecoder ) {

    let mut x: u32 = 0;

    for byte in buf {
        let color = palette[ ( ( *byte >> 4 ) & 0x0F ) as usize ];
        decoder.set_pixel( x, y, color.r, color.g, color.b, color.a );

        x += 1;
        if x >= width {
            break;
        }

        let color = palette[ ( *byte & 0x0F ) as usize ];
        decoder.set_pixel( x, y, color.r, color.g, color.b, color.a );

        x += 1;
        if x >= width {
            break;
        }
    }
}

fn decode_8bpp<TDecoder: BMPDecoder>(
    y: u32, width: u32, buf: &[u8], palette: &[Color], decoder: &mut TDecoder ) {

    let mut x: u32 = 0;

    for byte in buf {
        let color = palette[ *byte as usize ];
        decoder.set_pixel( x, y, color.r, color.g, color.b, color.a );

        x += 1;
        if x >= width {
            break;
        }
    }
}

fn decode_24bpp<TDecoder: BMPDecoder>(
    y: u32, width: u32, buf: &[u8], decoder: &mut TDecoder ) {

    let mut x: u32 = 0;

    for bytes in buf.chunks( 3 ) {
        decoder.set_pixel( x, y, bytes[2], bytes[1], bytes[0], 255 );

        x += 1;
        if x >= width {
            break;
        }
    }
}

pub fn decode<TDecoder: BMPDecoder>(
    input: &mut io::Read, mut decoder: TDecoder ) -> Result<TDecoder::TResult> {

    // Read file header
    let mut header: [u8; 14] = [0; 14];
    input.read_exact( &mut header )?;

    let mut cursor = io::Cursor::new( header );
    if header[0] != 0x42 && header[1] != 0x4D {
        return Err( DecodingError::new_io( "Invalid bitmap file." ) );
    }

    // TODO: Make sensible decisions about ridiculous big files

    cursor.set_position( 10 );
    let _ = cursor.read_u32::<LittleEndian>()?; // Offset

    // TODO: Make sensible decisions about the offset to the pixel data

    // Read bitmap header
    let header = BMPHeader::from_reader( input )?;

    // Set output size
    decoder.set_size( header.core.width, header.core.height );

    // Read pixel data
    let size = ( ( header.core.width * header.core.bpp + 31 ) / 32 ) * 4;
    let mut buffer = vec![0; size as usize];
    let width = header.core.width;
    let height = header.core.height;
    let bpp = header.core.bpp;

    let palette = match bpp {
        1 | 4 | 8 => header.palette.unwrap().colors,
        _ => Vec::new(),
    };

    for y in 0..height {
        input.read_exact( &mut buffer )?;

        // Apply bottom-up correction
        let y = if header.core.bottom_up {
            height - y - 1
        } else {
            y
        };

        // Decode pixels
        match bpp {
            1 => decode_1bpp( y, width, &buffer, &palette, &mut decoder ),
            4 => decode_4bpp( y, width, &buffer, &palette, &mut decoder ),
            8 => decode_8bpp( y, width, &buffer, &palette, &mut decoder ),
            24 => decode_24bpp( y, width, &buffer, &mut decoder ),
            v => panic!( "Unexpected bits per pixel {}", v ),
        }
    }

    decoder.build()
}













enum Compression {
    RLE8Bit,
    RLE4Bit,
    Bitfield,
}

struct BMPInfo {
    compression: Option<Compression>,
    size: u32,
    resolution_width: i32,
    resolution_height: i32,
    colors: u32,
    important_colors: u32,
}

struct BitfieldMask {
    red: u32,
    green: u32,
    blue: u32,
    alpha: u32,
}

struct BMPExtra {
    color_space_type: u32,
    red_x: i32,
    red_y: i32,
    red_z: i32,
    green_x: i32,
    green_y: i32,
    green_z: i32,
    blue_x: i32,
    blue_y: i32,
    blue_z: i32,
    gamma_red: u32,
    gamma_green: u32,
    gamma_blue: u32,
}

struct BMPProfile {
    intent: u32,
    data: u32,
    size: u32,
    reserved: u32,
}

impl BMPInfo {
    fn from_reader( input: &mut io::Read ) -> Result<BMPInfo> {
        let compression = match input.read_u32::<LittleEndian>()? {
            0 => None,
            1 => Some( Compression::RLE8Bit ),
            2 => Some( Compression::RLE4Bit ),
            3 => Some( Compression::Bitfield ),
            v @ _ => return Err( DecodingError::new_io(
                &format!( "Invalid compression {}", v ) ) ),
        };

        Ok( BMPInfo {
            compression,
            size: 0,
            resolution_width: 0,
            resolution_height: 0,
            colors: 0,
            important_colors: 0,
        } )
    }
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
