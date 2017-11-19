//! # bmp_rs
//!
//! A bitmap file decoder for Microsoft *bmp* files.
//!
//! ## Example
//!
//! ```rust,no_run
//! use std::fs::File;
//! use bmp_rs::{
//!     Result,
//!     Builder,
//! };
//!
//! struct ImageBuilder {
//!     // Your builder type that is able to construct an image
//! }
//!
//! struct Image {
//!     // Your image type that represents a bitmap
//! }
//!
//! impl Builder for ImageBuilder {
//!     type TResult = Image; // Your image type
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
//!         Ok ( Image { } )
//!     }
//! }
//!
//! fn main() {
//!     let mut file = File::open( "image.bmp" ).unwrap();
//!     let image = bmp_rs::decode( &mut file, ImageBuilder { } );
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
    BigEndian,
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

pub trait Builder {
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
enum Version {
    Microsoft2 = MSVERSION2_SIZE,
    Microsoft3 = MSVERSION3_SIZE,
    Microsoft4 = MSVERSION4_SIZE,
    Microsoft5 = MSVERSION5_SIZE,
}

impl Version { // TODO: Replace with TryFrom when available.
    fn from_isize( size: isize ) -> Result<Version> {
        match size {
            MSVERSION2_SIZE => Ok( Version::Microsoft2 ),
            MSVERSION3_SIZE => Ok( Version::Microsoft3 ),
            // MSVERSION4_SIZE => Ok( Version::Microsoft4 ),
            // MSVERSION5_SIZE => Ok( Version::Microsoft5 ),
            _ => Err( DecodingError::new_io(
                    &format!( "Invalid bitmap header {},", size ) ) ),
        }
    }
}

struct Core {
    width: u32,
    height: u32,
    bpp: u32,
    planes: u16,
    bottom_up: bool,
}

impl Core {
    fn from_buffer( buf: &[u8], version: Version ) -> Result<Core> {
        let mut cursor = io::Cursor::new( buf );

        let ( width, height ) =
            match version {
                Version::Microsoft2 => {
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
                if version == Version::Microsoft2
                    && ( bpp == 16 || bpp == 32 ) {

                    return Err( DecodingError::new_io(
                        &format!( "Invalid bits per pixel {}.", bpp ) ) );
                }
            },
            _ => return Err( DecodingError::new_io(
                &format!( "Invalid bits per pixel {}.", bpp ) ) ),
        }

        Ok( Core { width, height, bpp, planes, bottom_up } )
    }
}

struct Palette {
    colors: Vec<Color>,
}

impl Palette {
    fn from_buffer( buf: &[u8], size: usize, color_size: usize ) -> Result<Palette> {
        let iter = buf.chunks( color_size );
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
        Ok( Palette { colors } )
    }
}

#[derive( PartialEq, Eq, Clone, Copy )]
enum Compression {
    RLE8Bit = 1,
    RLE4Bit = 2,
    Bitfield = 3,
}

struct Info {
    compression: Option<Compression>,
    image_size: u32,
    ppm_x: i32,
    ppm_y: i32,
    used_colors: u32,
    important_colors: u32,
}

impl Info {
    fn from_buffer( buf: &[u8], bpp: u32 ) -> Result<Info> {
        let mut cursor = io::Cursor::new( buf );

        let compression = match cursor.read_u32::<LittleEndian>()? {
            0 => None,
            1 if bpp == 8 => Some( Compression::RLE8Bit ),
            2 if bpp == 4 => Some( Compression::RLE4Bit ),
            3 if bpp == 16 || bpp == 32 => Some( Compression::Bitfield ),
            v @ _ => return Err( DecodingError::new_io(
                &format!( "Invalid compression {} for {}-bit", v, bpp ) ) ),
        };

        let image_size = cursor.read_u32::<LittleEndian>()?;
        let ppm_x = cursor.read_i32::<LittleEndian>()?;
        let ppm_y = cursor.read_i32::<LittleEndian>()?;
        let used_colors = cursor.read_u32::<LittleEndian>()?;
        let important_colors = cursor.read_u32::<LittleEndian>()?;

        Ok ( Info {
            compression,
            image_size,
            ppm_x,
            ppm_y,
            used_colors,
            important_colors,
        } )
    }
}

struct BitfieldMask {
    red: u32,
    green: u32,
    blue: u32,
}

impl BitfieldMask {
    fn from_buffer( buf: &[u8] ) -> Result<BitfieldMask> {
        let mut cursor = io::Cursor::new( buf );

        let red = cursor.read_u32::<BigEndian>()?;
        let green = cursor.read_u32::<BigEndian>()?;
        let blue = cursor.read_u32::<BigEndian>()?;

        Ok( BitfieldMask { red, green, blue } )
    }
}

struct Header {
    version: Version,
    core: Core,
    info: Option<Info>,
    palette: Option<Palette>,
    bitmask: Option<BitfieldMask>,
}

impl Header {
    fn from_reader( input: &mut io::Read ) -> Result<Header> {
        let version = Version::from_isize(
            input.read_u32::<LittleEndian>()? as isize )?;

        // Read core header
        let mut buffer = vec![0; ( version as usize ) - 4];

        input.read_exact( &mut buffer )?;

        let core = Core::from_buffer( &buffer, version )?;
        let info = match version {
            Version::Microsoft3
                => Some( Info::from_buffer( &buffer[12..], core.bpp )? ),
            _ => None,
        };

        // Read Bitmask
        let bitmask = match info {
            Some( ref i ) => {
                match i.compression {
                    Some( Compression::Bitfield ) => {
                        let mut buffer = vec![0; 12];
                        input.read_exact( &mut buffer )?;

                        Some( BitfieldMask::from_buffer( &buffer )? )
                    },
                    _ => None,
                }
            }
            None => None,
        };

        // Read palette
        let palette_size = match info {
            Some( ref i ) if i.used_colors == 0 && core.bpp < 16 => 1 << core.bpp,
            Some( ref i ) => i.used_colors,
            None => 1 << core.bpp,
        };

        // TODO: Check if the size is sensible with the bitmap offset

        let palette = if palette_size > 0 {
            match core.bpp {
                1 | 4 | 8 => {
                    let palette_size = palette_size as usize;
                    let color_size = match version {
                        Version::Microsoft2 => 3,
                        _ => 4,
                    } as usize;

                    let mut buffer = vec![0; palette_size * color_size];
                    input.read_exact( &mut buffer )?;

                    Some( Palette::from_buffer( &buffer, palette_size, color_size )? )
                },
                _ => return Err( DecodingError::new_io(
                    &format!( "Unexpected color palette of size {}.", palette_size ) ) ),
            }
        } else {
            None
        };

        Ok ( Header {
            version,
            core,
            info,
            palette,
            bitmask,
        } )
    }
}

fn decode_1bpp<TBuilder: Builder>(
    width: u32, row: u32, buf: &[u8], palette: &[Color], mask: &BitfieldMask, builder: &mut TBuilder ) {

    let mut x: u32 = 0;

    for byte in buf {
        for bit in (0..8).rev() {

            let color: Color = palette[ ( ( *byte >> bit ) & 0x01 ) as usize ];
            builder.set_pixel( x, row, color.r, color.g, color.b, color.a );

            x += 1;
            if x >= width {
                return;
            }
        }
    }
}

fn decode_4bpp<TBuilder: Builder>(
    width: u32, row: u32, buf: &[u8], palette: &[Color], mask: &BitfieldMask, builder: &mut TBuilder ) {

    let mut x: u32 = 0;

    for byte in buf {
        let color = palette[ ( ( *byte >> 4 ) & 0x0F ) as usize ];
        builder.set_pixel( x, row, color.r, color.g, color.b, color.a );

        x += 1;
        if x >= width {
            break;
        }

        let color = palette[ ( *byte & 0x0F ) as usize ];
        builder.set_pixel( x, row, color.r, color.g, color.b, color.a );

        x += 1;
        if x >= width {
            break;
        }
    }
}

fn decode_8bpp<TBuilder: Builder>(
    width: u32, row: u32, buf: &[u8], palette: &[Color], mask: &BitfieldMask, builder: &mut TBuilder ) {

    let mut x: u32 = 0;

    for byte in buf {
        let color = palette[ *byte as usize ];
        builder.set_pixel( x, row, color.r, color.g, color.b, color.a );

        x += 1;
        if x >= width {
            break;
        }
    }
}

fn decode_16bpp<TBuilder: Builder>(
    width: u32, row: u32, buf: &[u8], palette: &[Color], mask: &BitfieldMask, builder: &mut TBuilder ) {
}

fn decode_24bpp<TBuilder: Builder>(
    width: u32, row: u32, buf: &[u8], palette: &[Color], mask: &BitfieldMask, builder: &mut TBuilder ) {

    let mut x: u32 = 0;

    for bytes in buf.chunks( 3 ) {
        builder.set_pixel( x, row, bytes[2], bytes[1], bytes[0], 255 );

        x += 1;
        if x >= width {
            break;
        }
    }
}

fn decode_32bpp<TBuilder: Builder>(
    width: u32, row: u32, buf: &[u8], palette: &[Color], mask: &BitfieldMask, builder: &mut TBuilder ) {
}

fn decode_nothing<TBuilder: Builder>(
    width: u32, row: u32, buf: &[u8], palette: &[Color], mask: &BitfieldMask, builder: &mut TBuilder ) {
    // no-op
}

pub fn decode<TBuilder: Builder>(
    input: &mut io::Read, mut builder: TBuilder ) -> Result<TBuilder::TResult> {

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
    let header = Header::from_reader( input )?;

    // Set output size
    builder.set_size( header.core.width, header.core.height );

    // Read pixel data
    let size = ( ( header.core.width * header.core.bpp + 31 ) / 32 ) * 4;
    let mut buffer = vec![0; size as usize];
    let width = header.core.width;
    let height = header.core.height;
    let bpp = header.core.bpp;
    let compression = match header.version {
        Version::Microsoft2 => false,
        _ if header.info.unwrap().compression.is_some() => true,
        _ => false,
    };
    let palette = match bpp {
        1 | 4 | 8 => header.palette.unwrap().colors,
        _ => Vec::new(),
    };

    let mask = match bpp {
        16 | 32 => header.bitmask.unwrap(),
        _ => BitfieldMask { red: 0, green: 0, blue: 0 }
    };

    let decode_row = match bpp {
        1 => decode_1bpp::<TBuilder>,
        4 => decode_4bpp::<TBuilder>,
        8 => decode_8bpp::<TBuilder>,
        16 => decode_16bpp::<TBuilder>,
        24 => decode_24bpp::<TBuilder>,
        32 => decode_32bpp::<TBuilder>,
        _ => decode_nothing::<TBuilder>,
    };

    for y in 0..height {
        input.read_exact( &mut buffer )?;

        let row = if header.core.bottom_up { height - y - 1 } else { y };

        decode_row( width, row, &buffer, &palette, &mask, &mut builder );
    }

    builder.build()
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
