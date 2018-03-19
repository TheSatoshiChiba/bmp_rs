extern crate byteorder;

use std::io;
use std::io::{
    Result,
    Read,
};

use byteorder::{
    ReadBytesExt,
    LittleEndian,
};

fn new_data_error<S>( message: S ) -> io::Error
    where S: Into<String> {

    io::Error::new( io::ErrorKind::InvalidData, message.into() )
}

#[derive( PartialEq, Eq, Clone, Copy )]
enum FileType {
    // DDB, // Denotes a device dependant bitmap file
    DeviceIndependentBitmap, // Denotes a device independent bitmap file
    // BA, // Denotes a bitmap array
    // CI, // Denotes a color icon
    // CP, // Denotes a color pointer
    // IC, // Denotes a icon
    // PT, // Denotes a pointer
}

#[derive( PartialEq, Eq, Clone, Copy )]
enum Version {
    Microsoft2,
    Microsoft3,
    Microsoft4,
    Microsoft5,
}

#[derive( PartialEq, Eq, Clone, Copy )]
enum Compression {
    RunLength8,
    RunLength4,
    Bitmask,
}

struct CoreHeader {
    version: Version,
    width: u32,
    height: u32,
    bpp: u32,
    planes: u16,
    top_down: bool,
}

struct InfoHeader {
    compression: Option<Compression>,
    image_size: u32,
    ppm_x: i32,
    ppm_y: i32,
    used_colors: u32,
    important_colors: u32,
}

struct BitfieldMask {
    red: u32,
    green: u32,
    blue: u32,
    alpha: u32,
}

struct ExtraHeader {
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

struct ProfileHeader {
    intent: u32,
    data: u32,
    size: u32,
    reserved: u32,
}

impl FileType {
    fn from_reader( input: &mut Read ) -> Result<FileType> {
        match input.read_u16::<LittleEndian>()? {
            // 0 => Ok( FileType::DDB ), // TODO: Enable for MS Version 1 Bitmaps
            0x4D42 => Ok( FileType::DeviceIndependentBitmap ),
            // 0x4142 => Ok( FileType::BA ),
            // 0x4943 => Ok( FileType::CI ),
            // 0x5043 => Ok( FileType::CP ),
            // 0x4349 => Ok( FileType::IC ),
            // 0x5450 => Ok( FileType::PT ),
            x @ _ => Err( new_data_error(
                format!( "Invalid file type 0x{:X}", x ) ) ),
        }
    }
}

impl Version {
    fn from_reader( input: &mut Read ) -> Result<Version> {
        match input.read_u32::<LittleEndian>()? {
            0x0C => Ok( Version::Microsoft2 ),
            0x28 => Ok( Version::Microsoft3 ),
            0x6C => Ok( Version::Microsoft4 ),
            0x7C => Ok( Version::Microsoft5 ),
            x @ _ => Err( new_data_error(
                format!( "Invalid header size 0x{:X}", x ) ) ),
        }
    }
}

impl Compression {
    fn from_reader( input: &mut Read, bpp: u32 ) -> Result<Option<Compression>> {
        match input.read_u32::<LittleEndian>()? {
            0x00 => Ok( None ),
            0x01 if bpp == 8 => Ok( Some( Compression::RunLength8 ) ),
            0x02 if bpp == 4 => Ok( Some( Compression::RunLength4 ) ),
            0x03 if bpp == 16 || bpp == 32 => Ok( Some( Compression::Bitmask ) ),
            x @ _ => Err( new_data_error(
                format!( "Invalid compression 0x{:X} for {}-bit", x, bpp ) ) ),
        }
    }
}

impl CoreHeader {
    fn from_reader( input: &mut Read, version: Version ) -> Result<CoreHeader> {
        let ( width, height ) = match version {
            Version::Microsoft2 => {
                let w = input.read_i16::<LittleEndian>()? as i32;
                let h = input.read_i16::<LittleEndian>()? as i32;

                ( w, h )
            },
            _ => {
                let w = input.read_i32::<LittleEndian>()?;
                let h = input.read_i32::<LittleEndian>()?;

                ( w, h )
            },
        };

        let top_down = height.is_negative();
        let width = width.checked_abs()
            .ok_or( new_data_error( format!( "Invalid image width {}", width ) ) )? as u32;
        let height = height.checked_abs()
            .ok_or( new_data_error( format!( "Invalid image height {}", height ) ) )? as u32;

        let planes = input.read_u16::<LittleEndian>()?;
        if planes != 1 {
            return Err( new_data_error( format!( "Invalid number of planes {}", planes ) ) );
        }

        let bpp = match input.read_u16::<LittleEndian>()? as u32 {
            x @ 1 | x @ 4 | x @ 8 | x @ 24 => x,
            x @ 16 | x @ 32 if version != Version::Microsoft2 => x,
            x @ _ => return Err( new_data_error( format!( "Invalid bits per pixel {}", x ) ) ),
        };

        Ok( CoreHeader {
            version,
            width,
            height,
            bpp,
            planes,
            top_down,
        } )
    }
}

impl InfoHeader {
    fn from_reader( input: &mut Read, compression: Option<Compression> ) -> Result<InfoHeader> {
        let image_size = input.read_u32::<LittleEndian>()?;
        let ppm_x = input.read_i32::<LittleEndian>()?;
        let ppm_y = input.read_i32::<LittleEndian>()?;
        let used_colors = input.read_u32::<LittleEndian>()?;
        let important_colors = input.read_u32::<LittleEndian>()?;

        Ok ( InfoHeader {
            compression,
            image_size,
            ppm_x,
            ppm_y,
            used_colors,
            important_colors,
        } )
    }
}

impl BitfieldMask {
    fn new() -> BitfieldMask {
        BitfieldMask {
            red: 0x00,
            green: 0x00,
            blue: 0x00,
            alpha: 0x00,
        }
    }

    fn from_bpp( bpp: u32 ) -> BitfieldMask {
        match bpp {
            16 => BitfieldMask {
                red: 0x7C00,
                green: 0x3E0,
                blue: 0x1F,
                alpha: 0x00,
            },
            32 => BitfieldMask {
                red: 0xFF0000,
                green: 0xFF00,
                blue: 0xFF,
                alpha: 0x00,
            },
            _ => BitfieldMask::new(),
        }
    }

    fn from_reader( input: &mut Read, version: Version ) -> Result<BitfieldMask> {
        let red = input.read_u32::<LittleEndian>()?;
        let green = input.read_u32::<LittleEndian>()?;
        let blue = input.read_u32::<LittleEndian>()?;

        let alpha = match version {
            Version::Microsoft4 | Version::Microsoft5 => input.read_u32::<LittleEndian>()?,
            _ => 0x00,
        };

        Ok( BitfieldMask { red, green, blue, alpha } )
    }
}

impl ExtraHeader {
    fn from_reader( input: &mut Read ) -> Result<ExtraHeader> {
        let color_space_type = input.read_u32::<LittleEndian>()?;
        let red_x = input.read_i32::<LittleEndian>()?;
        let red_y = input.read_i32::<LittleEndian>()?;
        let red_z = input.read_i32::<LittleEndian>()?;
        let green_x = input.read_i32::<LittleEndian>()?;
        let green_y = input.read_i32::<LittleEndian>()?;
        let green_z = input.read_i32::<LittleEndian>()?;
        let blue_x = input.read_i32::<LittleEndian>()?;
        let blue_y = input.read_i32::<LittleEndian>()?;
        let blue_z = input.read_i32::<LittleEndian>()?;
        let gamma_red = input.read_u32::<LittleEndian>()?;
        let gamma_green = input.read_u32::<LittleEndian>()?;
        let gamma_blue = input.read_u32::<LittleEndian>()?;

        Ok( ExtraHeader {
            color_space_type,
            red_x,
            red_y,
            red_z,
            green_x,
            green_y,
            green_z,
            blue_x,
            blue_y,
            blue_z,
            gamma_red,
            gamma_green,
            gamma_blue,
        } )
    }
}

impl ProfileHeader {
    fn from_reader( input: &mut Read ) -> Result<ProfileHeader> {
        let intent = input.read_u32::<LittleEndian>()?;
        let data = input.read_u32::<LittleEndian>()?;
        let size = input.read_u32::<LittleEndian>()?;
        let reserved = input.read_u32::<LittleEndian>()?;

        Ok( ProfileHeader {
            intent,
            data,
            size,
            reserved,
        } )
    }
}

fn read_palette( input: &mut Read, version: Version, bpp: u32, used_colors: u32 ) -> Result<Vec<u8>> {
    let size = match used_colors {
        0 if bpp < 16 => ( 1 << bpp ) as usize,
        _ => used_colors as usize,
    };
    if size > 0 {
        Ok( match version {
            Version::Microsoft2 => {
                let mut colors = Vec::with_capacity( size * 3 );
                for x in 0..size {
                    colors.push( input.read_u8()? ); // b
                    colors.push( input.read_u8()? ); // g
                    colors.push( input.read_u8()? ); // r
                }
                colors
            },
            _ => {
                let mut colors = Vec::with_capacity( size * 3 );
                for x in 0..size {
                    colors.push( input.read_u8()? ); // b
                    colors.push( input.read_u8()? ); // g
                    colors.push( input.read_u8()? ); // r

                    input.read_u8()?; // reserved
                    colors.push( 255 ); // a
                }
                colors
            },
        } )
    } else {
        Ok( Vec::new() )
    }
}

fn read_file_header( input: &mut Read ) -> Result<FileType> {
    let file_type = FileType::from_reader( input )?;
    let _file_size = input.read_u32::<LittleEndian>()?;
    // TODO: make sense of file_size (error when too big or small)

    input.read_u32::<LittleEndian>()?; // Reserved

    let _data_offset = input.read_u32::<LittleEndian>()?;
    // TODO: make sense of data_offset (error when too big or small)

    Ok( file_type )
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

pub( crate ) fn decode<TBuilder: super::Builder>(
    input: &mut Read, mut builder: TBuilder ) -> Result<TBuilder> {

    // Read file header
    let file_type = read_file_header( input )?;

    // Read core header
    let version = Version::from_reader( input )?;
    let core = CoreHeader::from_reader( input, version )?;

    // Read info header
    let ( compression, info, bitmask ) = match version {
        Version::Microsoft2 => ( None, None, BitfieldMask::new() ),
        _ => {
            let compression = Compression::from_reader( input, core.bpp )?;
            let info = InfoHeader::from_reader( input, compression )?;
            let bitmask = read_bitmask( input, version, compression, core.bpp )?;

            ( compression, Some( info ), bitmask )
        },
    };

    // Read extra header
    let extra = match version {
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
    let palette = match info {
        Some( ref i ) => read_palette( input, version, core.bpp, i.used_colors )?,
        None => read_palette( input, version, core.bpp, 0 )?,
    };

    // TODO: REWORK EVERYTHING FROM HERE ON
    // Set output size
    builder.set_size( core.width, core.height );

    // Read pixel data
    let size = ( ( core.width * core.bpp + 31 ) / 32 ) * 4;
    let mut buffer = vec![0; size as usize];
    let width = core.width;
    let height = core.height;
    let bpp = core.bpp;
    let color_width = match version {
        Version::Microsoft2 => 3,
        _ => 4,
    };

    let compression = match version {
        Version::Microsoft2 => false,
        _ if match info {
            Some( ref i ) => if i.compression.is_some() { true } else { false },
            None => false,
        } => true,
        _ => false,
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
        let mut y: u32 = if !core.top_down { height - 1 } else { 0 };
        let mut index: usize = 0;
        let row_mod: i32 = if !core.top_down { -1 } else { 1 };

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

                        let b = palette[ ( color_width * buffer[ index ] as usize ) ];
                        let g = palette[ ( color_width * buffer[ index ] as usize ) + 1 ];
                        let r = palette[ ( color_width * buffer[ index ] as usize ) + 2 ];

                        builder.set_pixel( x, y, r, g, b, 255 );
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
                let b = palette[ ( color_width * second ) ];
                let g = palette[ ( color_width * second ) + 1 ];
                let r = palette[ ( color_width * second ) + 2 ];

                for _ in 0..first {
                    if x >= width {
                        x = 0;
                        y = ( ( y as i32 ) + row_mod ) as u32;
                    }

                    builder.set_pixel( x, y, r, g, b, 255 );
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
        let mut y: u32 = if !core.top_down { height - 1 } else { 0 };
        let mut index: usize = 0;
        let row_mod: i32 = if !core.top_down { -1 } else { 1 };

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
                        let b = palette[ ( color_width * ( ( byte >> 4 ) & 0x0F ) as usize ) ];
                        let g = palette[ ( color_width * ( ( byte >> 4 ) & 0x0F ) as usize ) + 1 ];
                        let r = palette[ ( color_width * ( ( byte >> 4 ) & 0x0F ) as usize ) + 2 ];

                        builder.set_pixel( x, y, r, g, b, 255 );
                        x += 1;

                        if i < second_len - 1 {
                            if x >= width {
                                x = 0;
                                y = ( ( y as i32 ) + row_mod ) as u32;
                            }

                            let b = palette[ ( color_width * ( byte & 0x0F ) as usize ) ];
                            let g = palette[ ( color_width * ( byte & 0x0F ) as usize ) + 1 ];
                            let r = palette[ ( color_width * ( byte & 0x0F ) as usize ) + 2 ];

                            builder.set_pixel( x, y, r, g, b, 255 );
                            x += 1;
                        } else if even {
                            if x >= width {
                                x = 0;
                                y = ( ( y as i32 ) + row_mod ) as u32;
                            }

                            let b = palette[ ( color_width * ( byte & 0x0F ) as usize ) ];
                            let g = palette[ ( color_width * ( byte & 0x0F ) as usize ) + 1 ];
                            let r = palette[ ( color_width * ( byte & 0x0F ) as usize ) + 2 ];

                            builder.set_pixel( x, y, r, g, b, 255 );
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
                let b1 = palette[ ( color_width * ( ( second >> 4 ) & 0x0F ) as usize ) ];
                let g1 = palette[ ( color_width * ( ( second >> 4 ) & 0x0F ) as usize ) + 1 ];
                let r1 = palette[ ( color_width * ( ( second >> 4 ) & 0x0F ) as usize ) + 2 ];
                let b2 = palette[ ( color_width * ( second & 0x0F ) as usize ) ];
                let g2 = palette[ ( color_width * ( second & 0x0F ) as usize ) + 1 ];
                let r2 = palette[ ( color_width * ( second & 0x0F ) as usize ) + 2 ];

                let mut control = false;
                for _ in 0..first {
                    let ( b, g, r ) = match control {
                        true => {
                            control = false;
                            ( b2, g2, r2 )
                        },
                        false => {
                            control = true;
                            ( b1, g1, r1 )
                        },
                    };

                    if x >= width {
                        x = 0;
                        y = ( ( y as i32 ) + row_mod ) as u32;
                    }

                    builder.set_pixel( x, y, r, g, b, 255 );
                    x += 1;
                }
            }
        }
    } else {
        for y in 0..height {
            input.read_exact( &mut buffer )?;

            let row = if !core.top_down { height - y - 1 } else { y };

            decode_row( width, row, &buffer, version, &palette, &bitmask, &mut builder );
        }
    }

    Ok( builder )
}

fn decode_1bpp<TBuilder: super::Builder>(
    width: u32, row: u32, buf: &[u8], version: Version, palette: &[u8], mask: &BitfieldMask, builder: &mut TBuilder ) {

    let mut x: u32 = 0;
    let color_width = match version {
        Version::Microsoft2 => 3,
        _ => 4,
    };

    for byte in buf {
        for bit in (0..8).rev() {

            let b = palette[ ( color_width * ( ( *byte >> bit ) & 0x01 ) as usize ) ];
            let g = palette[ ( color_width * ( ( *byte >> bit ) & 0x01 ) as usize ) + 1 ];
            let r = palette[ ( color_width * ( ( *byte >> bit ) & 0x01 ) as usize ) + 2 ];
            builder.set_pixel( x, row, r, g, b, 255 );

            x += 1;
            if x >= width {
                return;
            }
        }

    }
}

fn decode_4bpp<TBuilder: super::Builder>(
    width: u32, row: u32, buf: &[u8], version: Version, palette: &[u8], mask: &BitfieldMask, builder: &mut TBuilder ) {

    let mut x: u32 = 0;
    let color_width = match version {
        Version::Microsoft2 => 3,
        _ => 4,
    };

    for byte in buf {
        let b = palette[ ( color_width * ( ( *byte >> 4 ) & 0x0F ) as usize ) ];
        let g = palette[ ( color_width * ( ( *byte >> 4 ) & 0x0F ) as usize ) + 1 ];
        let r = palette[ ( color_width * ( ( *byte >> 4 ) & 0x0F ) as usize ) + 2 ];
        builder.set_pixel( x, row, r, g, b, 255 );

        x += 1;
        if x >= width {
            break;
        }

        let b = palette[ ( color_width * ( *byte & 0x0F ) as usize ) ];
        let g = palette[ ( color_width * ( *byte & 0x0F ) as usize ) + 1 ];
        let r = palette[ ( color_width * ( *byte & 0x0F ) as usize ) + 2 ];
        builder.set_pixel( x, row, r, g, b, 255 );

        x += 1;
        if x >= width {
            break;
        }
    }
}

fn decode_8bpp<TBuilder: super::Builder>(
    width: u32, row: u32, buf: &[u8], version: Version, palette: &[u8], _mask: &BitfieldMask, builder: &mut TBuilder ) {

    let mut x: u32 = 0;
    let color_width = match version {
        Version::Microsoft2 => 3,
        _ => 4,
    };

    for byte in buf {
        let b = palette[ ( color_width * *byte as usize ) ];
        let g = palette[ ( color_width * *byte as usize ) + 1 ];
        let r = palette[ ( color_width * *byte as usize ) + 2 ];
        builder.set_pixel( x, row, r, g, b, 255 );

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

fn decode_16bpp<TBuilder: super::Builder>(
    width: u32, row: u32, buf: &[u8], version: Version, palette: &[u8], mask: &BitfieldMask, builder: &mut TBuilder ) {

    let mut x: u32 = 0;
    let color_width = match version {
        Version::Microsoft2 => 3,
        _ => 4,
    };

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

fn decode_24bpp<TBuilder: super::Builder>(
    width: u32, row: u32, buf: &[u8], version: Version, palette: &[u8], mask: &BitfieldMask, builder: &mut TBuilder ) {

    let mut x: u32 = 0;
    let color_width = match version {
        Version::Microsoft2 => 3,
        _ => 4,
    };

    for bytes in buf.chunks( 3 ) {
        builder.set_pixel( x, row, bytes[2], bytes[1], bytes[0], 255 );

        x += 1;
        if x >= width {
            break;
        }
    }
}

fn decode_32bpp<TBuilder: super::Builder>(
    width: u32, row: u32, buf: &[u8], version: Version, palette: &[u8], mask: &BitfieldMask, builder: &mut TBuilder ) {

    let mut x: u32 = 0;
    let color_width = match version {
        Version::Microsoft2 => 3,
        _ => 4,
    };

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

fn decode_nothing<TBuilder: super::Builder>(
    _: u32, _: u32, _: &[u8], _: Version, _: &[u8], _: &BitfieldMask, _: &mut TBuilder ) {
    // no-op
}
