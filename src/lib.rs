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
//!     let builder = ImageBuilder { }; // Create a builder instance
//!     let mut file = File::open( "image.bmp" ).unwrap(); // Open a file stream
//!     let builder = bmp_rs::decode( &mut file, builder ); // decode file
//!     let image = builder.build(); // build the final image
//!     // Do something with your image
//! }
//! ```
//!
extern crate byteorder;

use std::io::{
    Result,
    Read,
};

use byteorder::{
    ReadBytesExt,
    LittleEndian,
};

mod bitmap;

use bitmap::{
    Version,
    CoreHeader,
    InfoHeader,
    Compression,
    BitfieldMask,
    ExtraHeader,
    ProfileHeader,
};

pub trait Builder {
    type TResult;

    fn set_size( &mut self, width: u32, height: u32 );
    fn set_pixel( &mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8 );
    fn build( &mut self ) -> Result<Self::TResult>;
}

#[derive( Clone, Copy )]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

struct Palette {
    colors: Vec<Color>,
}

impl Palette {
    fn from_buffer(
        buf: &[u8], size: usize, color_size: usize, has_alpha: bool ) -> Result<Palette> {

        let iter = buf.chunks( color_size );
        let mut colors = Vec::with_capacity( size );

        for x in iter {
            let a = match has_alpha {
                true => x[3],
                false => 255,
            };

            colors.push(
                Color {
                    b: x[0],
                    g: x[1],
                    r: x[2],
                    a,
                } );
        }
        Ok( Palette { colors } )
    }
}

struct Header {
    core: CoreHeader,
    info: Option<InfoHeader>,
    bitmask: BitfieldMask,
    extra: Option<ExtraHeader>,
    profile: Option<ProfileHeader>,

    palette: Option<Palette>,
}

fn read_bitmask( input: &mut Read, version: Version, compression: Option<Compression>, bpp: u32 )
    -> Result<BitfieldMask> {

    match version {
        Version::Microsoft3 if compression == Some( Compression::Bitmask )
            => BitfieldMask::from_reader( input, version ),
        Version::Microsoft3 if compression == None
            => Ok( BitfieldMask::from_bpp( bpp ) ),
        Version::Microsoft4 | Version::Microsoft5
            => BitfieldMask::from_reader( input, version ),
        _ => Ok( BitfieldMask::new() ),
    }
}

impl Header {
    fn from_reader( input: &mut Read ) -> Result<Header> {
        // Read core header
        let core = CoreHeader::from_reader( input )?;

        // Read Info header & Bitmask
        let ( info, bitmask ) = match core.version {
            Version::Microsoft2 => ( None, BitfieldMask::new() ),
            _ => {
                let i = InfoHeader::from_reader( input, core.bpp )?;
                let m = read_bitmask( input, core.version, i.compression, core.bpp )?;
                ( Some( i ), m )
            },
        };

        // Read Extra header
        let extra = match core.version {
            Version::Microsoft4 | Version::Microsoft5
                => Some( ExtraHeader::from_reader( input )? ),
            _ => None,
        };

        // Read profile header
        let profile = match core.version {
            Version::Microsoft5
                => Some( ProfileHeader::from_reader( input )? ),
            _ => None,
        };

        // Read palette
        let palette_size = match info {
            Some( ref i ) if i.used_colors == 0 && core.bpp < 16 => 1 << core.bpp,
            Some( ref i ) => i.used_colors,
            None => 1 << core.bpp,
        };

        // TODO: Check if the size is sensible with the bitmap offset

        let palette = if palette_size > 0 {
            let palette_size = palette_size as usize;
            let color_size = match core.version {
                Version::Microsoft2 => 3,
                _ => 4,
            } as usize;

            let mut buffer = vec![0; palette_size * color_size];
            input.read_exact( &mut buffer )?;

            // TODO: Last parameter indicated real alpha values in the bitmap.
            // I have yet to find one where this is anything else than 0.
            Some( Palette::from_buffer( &buffer, palette_size, color_size, false )? )
        } else {
            None
        };

        Ok ( Header {
            core,
            info,
            palette,
            bitmask,
            extra,
            profile,
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
    width: u32, row: u32, buf: &[u8], palette: &[Color], _mask: &BitfieldMask, builder: &mut TBuilder ) {

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

fn clamp8bit( value: u32, mask: u32, shr_count: u32, mask_max: u32, default: u8 ) -> u8 {
    match mask_max {
        0 => default,
        max @ _ => ( ( 255 * ( ( value & mask ) >> shr_count ) ) / max ) as u8,
    }
}

fn decode_16bpp<TBuilder: Builder>(
    width: u32, row: u32, buf: &[u8], palette: &[Color], mask: &BitfieldMask, builder: &mut TBuilder ) {

    let mut x: u32 = 0;

    let alpha_shift = mask.alpha.trailing_zeros();
    let red_shift = mask.red.trailing_zeros();
    let green_shift = mask.green.trailing_zeros();
    let blue_shift = mask.blue.trailing_zeros();

    let alpha_max = mask.alpha.checked_shr( alpha_shift ).unwrap_or( 0 );
    let red_max = mask.red.checked_shr( red_shift ).unwrap_or( 0 );
    let green_max = mask.green.checked_shr( green_shift ).unwrap_or( 0 );
    let blue_max = mask.blue.checked_shr( blue_shift ).unwrap_or( 0 );

    for mut bytes in buf.chunks( 2 ) {
        let color = bytes.read_u16::<LittleEndian>().unwrap() as u32;

        builder.set_pixel(
            x,
            row,
            clamp8bit( color, mask.red, red_shift, red_max, 0 ),
            clamp8bit( color, mask.green, green_shift, green_max, 0 ),
            clamp8bit( color, mask.blue, blue_shift, blue_max, 0 ),
            clamp8bit( color, mask.alpha, alpha_shift, alpha_max, 255 ) );

        x += 1;
        if x >= width {
            break;
        }
    }
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

    let mut x: u32 = 0;

    let alpha_shift = mask.red.trailing_zeros();
    let red_shift = mask.red.trailing_zeros();
    let green_shift = mask.green.trailing_zeros();
    let blue_shift = mask.blue.trailing_zeros();

    let alpha_max = mask.alpha.checked_shr( alpha_shift ).unwrap_or( 0 );
    let red_max = mask.red.checked_shr( red_shift ).unwrap_or( 0 );
    let green_max = mask.green.checked_shr( green_shift ).unwrap_or( 0 );
    let blue_max = mask.blue.checked_shr( blue_shift ).unwrap_or( 0 );

    for mut bytes in buf.chunks( 4 ) {
        let color = bytes.read_u32::<LittleEndian>().unwrap() as u32;

        builder.set_pixel(
            x,
            row,
            clamp8bit( color, mask.red, red_shift, red_max, 0 ),
            clamp8bit( color, mask.green, green_shift, green_max, 0 ),
            clamp8bit( color, mask.blue, blue_shift, blue_max, 0 ),
            clamp8bit( color, mask.alpha, alpha_shift, alpha_max, 255 ) );

        x += 1;
        if x >= width {
            break;
        }
    }
}

fn decode_nothing<TBuilder: Builder>(
    _: u32, _: u32, _: &[u8], _: &[Color], _: &BitfieldMask, _: &mut TBuilder ) {
    // no-op
}

pub fn decode<TBuilder: Builder>(
    input: &mut Read, mut builder: TBuilder ) -> Result<TBuilder> {

    // Read file header
    builder = bitmap::decode( input, builder )?;

    // TODO: Make sensible decisions about ridiculous big files
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
    let info = header.info;
    let compression = match header.core.version {
        Version::Microsoft2 => false,
        _ if match info {
            Some( ref i ) => if i.compression.is_some() { true } else { false },
            None => false,
        } => true,
        _ => false,
    };
    let palette = match bpp {
        1 | 4 | 8 => header.palette.unwrap().colors,
        _ => Vec::new(),
    };

    let decode_row = match bpp {
        1 => decode_1bpp::<TBuilder>,
        4 if !compression => decode_4bpp::<TBuilder>,
        8 if !compression => decode_8bpp::<TBuilder>,
        16 => decode_16bpp::<TBuilder>,
        24 => decode_24bpp::<TBuilder>,
        32 => decode_32bpp::<TBuilder>,
        _ => decode_nothing::<TBuilder>,
    };

    if bpp == 8 && compression {
        let count = info.unwrap().image_size as usize;
        if count == 0 {
            panic!( "Image size in bytes can't be null when using RLE8 compression" );
        }
        let mut buffer = vec![0; count];
        input.read_exact( &mut buffer )?;
        let buffer = buffer;

        let mut x: u32 = 0;
        let mut y: u32 = if !header.core.top_down { height - 1 } else { 0 };
        let mut index: usize = 0;
        let row_mod: i32 = if !header.core.top_down { -1 } else { 1 };

        loop {
            if index >= count {
                break;
            }

            let first = buffer[ index ] as usize;
            let second = buffer[ index + 1 ] as usize;
            index += 2;

            if first == 0 {
                if second == 0 {
                    x = 0;
                    y = ( ( y as i32 ) + row_mod ) as u32;

                } else if second == 1 {
                    break;

                } else if second == 2 {
                    let dx = buffer[ index ] as u32;
                    let dy = buffer[ index + 1 ] as i32 * row_mod;
                    index += 2;

                    x += dx;
                    y = ( y as i32 + dy ) as u32;

                } else {
                    for _ in 0..second {
                        if x >= width {
                            x = 0;
                            y = ( ( y as i32 ) + row_mod ) as u32;
                        }

                        let color = palette[ buffer[ index ] as usize ];

                        builder.set_pixel( x, y, color.r, color.g, color.b, color.a );
                        x += 1;

                        index += 1;
                        if index >= count {
                            break;
                        }
                    }
                    index += match second % 2 {
                        0 => 0,
                        _ => 1,
                    };
                }

            } else {
                let color = palette[ second ];

                for _ in 0..first {
                    if x >= width {
                        x = 0;
                        y = ( ( y as i32 ) + row_mod ) as u32;
                    }

                    builder.set_pixel( x, y, color.r, color.g, color.b, color.a );
                    x += 1;
                }
            }
        }

    } else if bpp == 4 && compression {
        let count = info.unwrap().image_size as usize;
        if count == 0 {
            panic!( "Image size in bytes can't be null when using RLE4 compression" );
        }
        let mut buffer = vec![0; count];
        input.read_exact( &mut buffer )?;
        let buffer = buffer;

        let mut x: u32 = 0;
        let mut y: u32 = if !header.core.top_down { height - 1 } else { 0 };
        let mut index: usize = 0;
        let row_mod: i32 = if !header.core.top_down { -1 } else { 1 };

        loop {
            if index >= count {
                break;
            }
            let first = buffer[ index ] as usize;
            let second = buffer[ index + 1 ];
            index += 2;

            if first == 0 {
                if second == 0 {
                    x = 0;
                    y = ( ( y as i32 ) + row_mod ) as u32;

                } else if second == 1 {
                    break;

                } else if second == 2 {
                    let dx = buffer[ index ] as u32;
                    let dy = buffer[ index + 1 ] as i32 * row_mod;
                    index += 2;

                    x += dx;
                    y = ( y as i32 + dy ) as u32;

                } else {
                    let even = second % 2 == 0;
                    let second_len = if !even {
                        second as usize + 1
                    } else {
                        second as usize
                    } / 2;

                    for i in 0..second_len {
                        if x >= width {
                            x = 0;
                            y = ( ( y as i32 ) + row_mod ) as u32;
                        }

                        let byte = buffer[ index ];
                        let color = palette[ ( ( byte >> 4 ) & 0x0F ) as usize ];

                        builder.set_pixel( x, y, color.r, color.g, color.b, color.a );
                        x += 1;

                        if i < second_len - 1 {
                            if x >= width {
                                x = 0;
                                y = ( ( y as i32 ) + row_mod ) as u32;
                            }

                            let color = palette[ ( byte & 0x0F ) as usize ];

                            builder.set_pixel( x, y, color.r, color.g, color.b, color.a );
                            x += 1;
                        } else if even {
                            if x >= width {
                                x = 0;
                                y = ( ( y as i32 ) + row_mod ) as u32;
                            }

                            let color = palette[ ( byte & 0x0F ) as usize ];

                            builder.set_pixel( x, y, color.r, color.g, color.b, color.a );
                            x += 1;
                        }

                        index += 1;
                        if index >= count {
                            break;
                        }
                    }
                    index += match second_len % 2 {
                        0 => 0,
                        _ => 1,
                    };
                }

            } else {
                let color1 = palette[ ( ( second >> 4 ) & 0x0F ) as usize ];
                let color2 = palette[ ( second & 0x0F ) as usize ];

                let mut control = false;
                for _ in 0..first {
                    let color = match control {
                        true => {
                            control = false;
                            &color2
                        },
                        false => {
                            control = true;
                            &color1
                        },
                    };

                    if x >= width {
                        x = 0;
                        y = ( ( y as i32 ) + row_mod ) as u32;
                    }

                    builder.set_pixel( x, y, color.r, color.g, color.b, color.a );
                    x += 1;
                }
            }
        }
    } else {
        for y in 0..height {
            input.read_exact( &mut buffer )?;

            let row = if !header.core.top_down { height - y - 1 } else { y };

            decode_row( width, row, &buffer, &palette, &header.bitmask, &mut builder );
        }
    }

    Ok( builder )
}
