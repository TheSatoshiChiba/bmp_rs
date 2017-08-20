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
    pub width : i32,
    pub height : i32,
    bpp : i32,
    direction : i8,
}

impl BitmapHeader {
    fn offset( &self, y : usize ) -> usize {
        match self.direction {
            -1 => (y * self.width as usize ),
            1 => ( ( self.height as usize - 1 ) * self.width as usize ) - (y * self.width as usize ),
            _ => panic!( "Invalid direction!" ),
        }
    }
}

pub trait ReadBmpExt : Read {
    fn read_file_header( &mut self ) -> Result<FileHeader> {
        if self.read_u8()? != 0x42 || self.read_u8()? != 0x4D {
            return Err( Error::new(
                ErrorKind::InvalidData,
                "Invalid bitmap header." ) );
        }

        let size = self.read_u32::<LittleEndian>()?; // File size
        let _ = self.read_u32::<LittleEndian>()?; // Reserved fields
        let offset = self.read_u32::<LittleEndian>()?; // Offset to bitmap data

        Ok( FileHeader { size, offset } )
    }

    fn read_bitmap_header( &mut self ) -> Result<BitmapHeader> {
        // Read BMP Version 2 header
        let size = self.read_u32::<LittleEndian>()?;
        println!("BMP HEADER SIZE {:?}",size );
        if size != 12 { // Header size
            return Err( Error::new(
                ErrorKind::InvalidData,
                "Invalid bitmap header size. Only BMP Version 2 is supported at this time." ) );
        }

        let w = self.read_i16::<LittleEndian>()?;
        let h = self.read_i16::<LittleEndian>()?;

        let width = w.checked_abs().ok_or(
            Error::new( ErrorKind::InvalidData, "Invalid bitmap width." ) )? as i32;

        let height = h.checked_abs().ok_or(
            Error::new( ErrorKind::InvalidData, "Invalid bitmap height." ) )? as i32;

        let direction = h.signum() as i8;

        if self.read_u16::<LittleEndian>()? != 1 { // Color planes
            return Err( Error::new(
                ErrorKind::InvalidData,
                "Invalid bitmap color plane." ) );
        }

        let bpp = match self.read_u16::<LittleEndian>()? {
            v @ 1
            | v @ 4
            | v @ 8
            | v @ 24 => v as i32,
            _ => return Err( Error::new(
                    ErrorKind::InvalidData,
                    "Invalid bitmap bits per pixel." ) ),
        };

        Ok( BitmapHeader { size, width, height, bpp, direction } )
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

    fn read_pixel_data( &mut self, bmp_header : &BitmapHeader, palette : &[Color] ) -> Result<Vec<Color>> {
        let size = ( bmp_header.width * bmp_header.height ) as usize;
        let mut pixels = vec![Color::default(); size];

        let line_width = ( ( bmp_header.width * bmp_header.bpp + 31 ) / 32 ) * 4;
        let mut line = vec![0 as u8; line_width as usize];

        for y in 0..bmp_header.height {
            self.read_exact( &mut line )?; // read whole line

            let offset = bmp_header.offset( y as usize );
            let mut index = offset;
            let mut range = 0..line_width;
            loop {
                match range.next() {
                    Some( mut x ) => {
                        match bmp_header.bpp {
                            1 => {
                                for i in (0..8).rev() {
                                    pixels[ index ] = palette[((line[ x as usize ] >> i ) & 0x01) as usize];
                                    index += 1;

                                    if i < 7 {
                                        if index >= offset + bmp_header.width as usize {
                                            break;
                                        }
                                    }
                                }
                            },
                            4 => {
                                pixels[ index ] = palette[((line[ x as usize ] >> 4 ) & 0x0F) as usize];
                                index += 1;

                                if index >= offset + bmp_header.width as usize {
                                    break;
                                }

                                pixels[ index ] = palette[(line[ x as usize ] & 0x0F) as usize];
                                index += 1;
                            },
                            8 => {
                                pixels[ index ] = palette[line[ x as usize ] as usize];
                                index += 1;
                            },
                            24 => {
                                let b = line[ x as usize ];
                                if let Some( z ) = range.next() {
                                    x = z;
                                } else { break }

                                let g = line[ x as usize ];
                                if let Some( z ) = range.next() {
                                    x = z;
                                } else { break }

                                let r = line[ x as usize ];

                                pixels[ index ] = Color { r, g, b, a : 255 };
                                index += 1;
                            },
                            _=> return Err( Error::new(
                                ErrorKind::InvalidData,
                                "Invalid bitmap bits per pixel." ) ),
                        }
                    },
                    None => break,
                }

                if index >= offset + bmp_header.width as usize {
                    break;
                }
            }
        }

        Ok( pixels )
    }
}

impl <R: Read + ?Sized> ReadBmpExt for R {}
