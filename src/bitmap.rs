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
