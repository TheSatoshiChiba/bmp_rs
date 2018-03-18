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
pub enum FileType {
    // DDB, // Denotes a device dependant bitmap file
    DeviceIndependentBitmap, // Denotes a device independent bitmap file
    // BA, // Denotes a bitmap array
    // CI, // Denotes a color icon
    // CP, // Denotes a color pointer
    // IC, // Denotes a icon
    // PT, // Denotes a pointer
}

#[derive( PartialEq, Eq, Clone, Copy )]
pub enum Version {
    Microsoft2,
    Microsoft3,
    Microsoft4,
    Microsoft5,
}

#[derive( PartialEq, Eq, Clone, Copy )]
pub enum Compression {
    RunLength8,
    RunLength4,
    Bitmask,
}

pub struct FileHeader {
    file_type: FileType,
    file_size: u32,
    data_offset: u32,
}

pub struct BitfieldMask {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub alpha: u32,
}

pub struct InfoHeader {
    pub compression: Option<Compression>,
    pub image_size: u32,
    ppm_x: i32,
    ppm_y: i32,
    pub used_colors: u32,
    important_colors: u32,
}

pub struct ExtraHeader {
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

pub struct ProfileHeader {
    intent: u32,
    data: u32,
    size: u32,
    reserved: u32,
}

pub struct BitmapHeader {
    pub version: Version,
    pub width: u32,
    pub height: u32,
    pub bpp: u32,
    planes: u16,
    pub top_down: bool,
}

impl FileType {
    pub fn from_u16( value: u16 ) -> Result<FileType> {
        match value {
            // 0 => Ok( FileType::DDB ), // TODO: Enable for MS Version 1 Bitmaps
            0x4D42 => Ok( FileType::DeviceIndependentBitmap ),
            // 0x4142 => Ok( FileType::BA ),
            // 0x4943 => Ok( FileType::CI ),
            // 0x5043 => Ok( FileType::CP ),
            // 0x4349 => Ok( FileType::IC ),
            // 0x5450 => Ok( FileType::PT ),
            x @ _ => Err( new_data_error( format!( "Invalid file type 0x{:X}", x ) ) ),
        }
    }
}

impl FileHeader {
    pub fn from_reader( input: &mut Read ) -> Result<FileHeader> {
        let file_type = FileType::from_u16( input.read_u16::<LittleEndian>()? )?;
        let file_size = input.read_u32::<LittleEndian>()?;

        input.read_u32::<LittleEndian>()?; // Reserved

        let data_offset = input.read_u32::<LittleEndian>()?;

        Ok( FileHeader {
            file_type,
            file_size,
            data_offset,
        } )
    }
}

impl Version {
    pub fn from_u32( value: u32 ) -> Result<Version> {
        match value {
            0x0C => Ok( Version::Microsoft2 ),
            0x28 => Ok( Version::Microsoft3 ),
            0x6C => Ok( Version::Microsoft4 ),
            0x7C => Ok( Version::Microsoft5 ),
            x @ _ => Err( new_data_error( format!( "Invalid header size 0x{:X}", x ) ) ),
        }
    }
}

impl Compression {
    pub fn from_u32( value: u32, bpp: u32 ) -> Result<Option<Compression>> {
        match value {
            0x00 => Ok( None ),
            0x01 if bpp == 8 => Ok( Some( Compression::RunLength8 ) ),
            0x02 if bpp == 4 => Ok( Some( Compression::RunLength4 ) ),
            0x03 if bpp == 16 || bpp == 32 => Ok( Some( Compression::Bitmask ) ),
            x @ _ => Err( new_data_error(
                format!( "Invalid compression 0x{:X} for {}-bit", x, bpp ) ) ),
        }
    }
}

impl InfoHeader {
    pub fn from_reader( input: &mut Read, bpp: u32 ) -> Result<InfoHeader> {
        let compression = Compression::from_u32( input.read_u32::<LittleEndian>()?, bpp )?;
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
    pub fn new() -> BitfieldMask {
        BitfieldMask {
            red: 0x00,
            green: 0x00,
            blue: 0x00,
            alpha: 0x00,
        }
    }

    pub fn from_bpp( bpp: u32 ) -> BitfieldMask {
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

    pub fn from_reader( input: &mut Read, version: Version ) -> Result<BitfieldMask> {
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
    pub fn from_reader( input: &mut Read ) -> Result<ExtraHeader> {
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
    pub fn from_reader( input: &mut Read ) -> Result<ProfileHeader> {
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

fn read_dimensions( input: &mut Read, version: Version ) -> Result<( u32, u32, bool )> {
    let ( w, h ) = match version {
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

    let top_down = h.is_negative();
    let w = w.checked_abs()
        .ok_or( new_data_error( format!( "Invalid image width {}", w ) ) )? as u32;
    let h = h.checked_abs()
        .ok_or( new_data_error( format!( "Invalid image height {}", h ) ) )? as u32;

    Ok( ( w, h, top_down ) )
}

impl BitmapHeader {
    pub fn from_reader( input: &mut Read ) -> Result<BitmapHeader> {
        let version = Version::from_u32( input.read_u32::<LittleEndian>()? )?;
        let ( width, height, top_down ) = read_dimensions( input, version )?;

        let planes = input.read_u16::<LittleEndian>()?;
        if planes != 1 {
            return Err( new_data_error( format!( "Invalid number of planes {}", planes ) ) );
        }

        let bpp = match input.read_u16::<LittleEndian>()? as u32 {
            x @ 1 | x @ 4 | x @ 8 | x @ 24 => x,
            x @ 16 | x @ 32 if version != Version::Microsoft2 => x,
            x @ _ => return Err( new_data_error( format!( "Invalid bits per pixel {}", x ) ) ),
        };

        Ok( BitmapHeader {
            version,
            width,
            height,
            bpp,
            planes,
            top_down,
        } )
    }
}
