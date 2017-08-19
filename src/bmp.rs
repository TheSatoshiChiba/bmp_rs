//! # bmp
//!
//! A module to read and write bmp files
extern crate byteorder;

use std::io::{
    Read,
    Result,
    Error,
    ErrorKind,
};

use self::byteorder::{
    ReadBytesExt,
    LittleEndian,
};

pub struct FileHeader {
    size : u32,
    offset : u32,
}

pub struct BitmapHeader {
    width : i16,
    height : i16,
    bpp : u16,
}

pub trait ReadBmpExt : Read {
    fn read_file_header( &mut self ) -> Result<FileHeader> {
        if self.read_u8()? != 0x42 || self.read_u8()? != 0x4D {
            return Err( Error::new(
                ErrorKind::InvalidData,
                "Invalid bitmap header." ) );
        }

        let size = self.read_u32::<LittleEndian>()?; // File size
        let _ = self.read_u16::<LittleEndian>()?; // Reserved fields
        let offset = self.read_u32::<LittleEndian>()?; // Offset to bitmap data

        Ok( FileHeader { size, offset } )
    }

    fn read_bitmap_header( &mut self ) -> Result<BitmapHeader> {
        // Read BMP Version 2 header
        if self.read_u32::<LittleEndian>()? != 12 { // Header size
            return Err( Error::new(
                ErrorKind::InvalidData,
                "Invalid bitmap header size. Only BMP Version 2 is supported at this time." ) );
        }

        let width = self.read_i16::<LittleEndian>()?
            .checked_abs()
            .ok_or( Error::new( ErrorKind::InvalidData, "Invalid bitmap width." ) )?;

        let height = self.read_i16::<LittleEndian>()?
            .checked_abs()
            .ok_or( Error::new( ErrorKind::InvalidData, "Invalid bitmap height." ) )?;

        if self.read_u16::<LittleEndian>()? != 1 { // Color planes
            return Err( Error::new(
                ErrorKind::InvalidData,
                "Invalid bitmap color plane." ) );
        }

        let bpp = match self.read_u16::<LittleEndian>()? {
            v @ 1
            | v @ 4
            | v @ 8
            | v @ 24 => v,
            _ => return Err( Error::new(
                    ErrorKind::InvalidData,
                    "Invalid bitmap bits per pixel." ) ),
        };

        Ok( BitmapHeader { width, height, bpp } )
    }
}

impl <R: Read + ?Sized> ReadBmpExt for R {}
