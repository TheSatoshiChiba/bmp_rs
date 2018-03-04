use std::io::{
    Result,
    Error,
    ErrorKind,
};

fn create_error( message: &str ) -> Error {
    Error::new( ErrorKind::InvalidData, message )
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
            x @ _ => Err( create_error( &format!( "Invalid file type {:X}", x ) ) ),
        }
    }
}
