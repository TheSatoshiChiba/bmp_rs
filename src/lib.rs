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

mod bitmap;

pub trait Builder {
    type TResult;

    fn set_size( &mut self, width: u32, height: u32 );
    fn set_pixel( &mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8 );
    fn build( &mut self ) -> Result<Self::TResult>;
}

pub fn decode<TBuilder: Builder>(
    input: &mut Read, mut builder: TBuilder ) -> Result<TBuilder> {

    bitmap::decode( input, builder )
}
