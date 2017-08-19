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

use super::Color;

pub struct FileHeader {
    size : u32,
    offset : u32,
}

pub struct BitmapHeader {
    size : u32,
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
        let size = self.read_u32::<LittleEndian>()?;
        if size != 12 { // Header size
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

        Ok( BitmapHeader { size, width, height, bpp } )
    }

    fn read_color_palette(
        &mut self,
        file_header : &FileHeader,
        bmp_header : &BitmapHeader ) -> Result<Vec<Color>> {

        let size = ( ( file_header.offset - 14 - bmp_header.size ) / 3 ) as usize;
        let mut palette = Vec::with_capacity( size );
        let mut entry : [u8; 3] = [0; 3];

        for _ in 0..size {
            self.read_exact( &mut entry )?;
            palette.push( Color {
                b : entry[0],
                g : entry[1],
                r : entry[2],
                a : 255,
            } );
        }

        Ok( palette )
    }
}

impl <R: Read + ?Sized> ReadBmpExt for R {}
