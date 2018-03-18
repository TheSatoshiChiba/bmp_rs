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

pub fn create_error<S>( message: S ) -> io::Error
    where S: Into<String> {

    io::Error::new( io::ErrorKind::InvalidData, message.into() )
}

#[derive( PartialEq, Eq, Clone, Copy )]
pub enum FileType {
    // DDB, // Denotes a device dependant bitmap file
    DIB, // Denotes a device independent bitmap file
    // BA, // Denotes a bitmap array
    // CI, // Denotes a color icon
    // CP, // Denotes a color pointer
    // IC, // Denotes a icon
    // PT, // Denotes a pointer
}

impl FileType {
    pub fn from_u16( value: u16 ) -> Result<FileType> {
        match value {
            // 0 => Ok( FileType::DDB ), // TODO: Enable for MS Version 1 Bitmaps
            0x4D42 => Ok( FileType::DIB ),
            // 0x4142 => Ok( FileType::BA ),
            // 0x4943 => Ok( FileType::CI ),
            // 0x5043 => Ok( FileType::CP ),
            // 0x4349 => Ok( FileType::IC ),
            // 0x5450 => Ok( FileType::PT ),
            x @ _ => Err( create_error( format!( "Invalid file type 0x{:X}", x ) ) ),
        }
    }
}

pub struct FileHeader {
    file_type: FileType,
    file_size: u32,
    data_offset: u32,
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

#[derive( PartialEq, Eq, Clone, Copy )]
pub enum Version {
    MICROSOFT2,
    MICROSOFT3,
    MICROSOFT4,
    MICROSOFT5,
}

impl Version {
    pub fn from_u32( value: u32 ) -> Result<Version> {
        match value {
            0x0C => Ok( Version::MICROSOFT2 ),
            0x28 => Ok( Version::MICROSOFT3 ),
            0x6C => Ok( Version::MICROSOFT4 ),
            0x7C => Ok( Version::MICROSOFT5 ),
            x @ _ => Err( create_error( format!( "Invalid header size 0x{:X}", x ) ) ),
        }
    }
}

#[derive( PartialEq, Eq, Clone, Copy )]
pub enum Compression {
    RLE8,
    RLE4,
    MASK,
}

impl Compression {
    pub fn from_u32( value: u32, bpp: u32 ) -> Result<Option<Compression>> {
        match value {
            0x00 => Ok( None ),
            0x01 if bpp == 8 => Ok( Some( Compression::RLE8 ) ),
            0x02 if bpp == 4 => Ok( Some( Compression::RLE4 ) ),
            0x03 if bpp == 16 || bpp == 32 => Ok( Some( Compression::MASK ) ),
            x @ _ => Err( create_error(
                format!( "Invalid compression 0x{:X} for {}-bit", x, bpp ) ) ),
        }
    }
}

pub struct InfoHeader {
    pub compression: Option<Compression>,
    pub image_size: u32,
    ppm_x: i32,
    ppm_y: i32,
    pub used_colors: u32,
    important_colors: u32,
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

pub struct BitfieldMask {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub alpha: u32,
}

impl BitfieldMask {
    pub fn from_reader( input: &mut Read, version: Version ) -> Result<BitfieldMask> {
        let red = input.read_u32::<LittleEndian>()?;
        let green = input.read_u32::<LittleEndian>()?;
        let blue = input.read_u32::<LittleEndian>()?;

        let alpha = match version {
            Version::MICROSOFT4 | Version::MICROSOFT5 => input.read_u32::<LittleEndian>()?,
            _ => 0,
        };

        Ok( BitfieldMask { red, green, blue, alpha } )
    }
}

pub struct BitmapHeader {
    pub version: Version,
    pub width: u32,
    pub height: u32,
    pub bpp: u32,
    planes: u16,
    pub top_down: bool,
}

fn read_dimensions( input: &mut Read, version: Version ) -> Result<( u32, u32, bool )> {
    let ( w, h ) = match version {
        Version::MICROSOFT2 => {
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
        .ok_or( create_error( format!( "Invalid image width {}", w ) ) )? as u32;
    let h = h.checked_abs()
        .ok_or( create_error( format!( "Invalid image height {}", h ) ) )? as u32;

    Ok( ( w, h, top_down ) )
}

impl BitmapHeader {
    pub fn from_reader( input: &mut Read ) -> Result<BitmapHeader> {
        let version = Version::from_u32( input.read_u32::<LittleEndian>()? )?;
        let ( width, height, top_down ) = read_dimensions( input, version )?;

        let planes = input.read_u16::<LittleEndian>()?;
        if planes != 1 {
            return Err( create_error( format!( "Invalid number of planes {}", planes ) ) );
        }

        let bpp = match input.read_u16::<LittleEndian>()? as u32 {
            x @ 1 | x @ 4 | x @ 8 | x @ 24 => x,
            x @ 16 | x @ 32 if version != Version::MICROSOFT2 => x,
            x @ _ => return Err( create_error( format!( "Invalid bits per pixel {}", x ) ) ),
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
