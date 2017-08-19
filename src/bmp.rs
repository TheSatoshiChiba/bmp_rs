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

use byteorder::{
    ReadBytesExt,
    LittleEndian,
};
