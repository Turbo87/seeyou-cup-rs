#![doc = include_str!("../README.md")]

mod error;
mod parser;
pub mod spec;
mod types;
mod writer;

pub use error::CupError;
pub use types::*;

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;

/// Character encoding for CUP files
#[derive(Debug, Clone, Copy)]
pub enum CupEncoding {
    /// UTF-8 encoding
    Utf8,
    /// Windows-1252 encoding (legacy)
    Windows1252,
}

/// SeeYou CUP file representation
#[derive(Debug, Default)]
pub struct CupFile {
    /// Waypoints defined in the file
    pub waypoints: Vec<Waypoint>,
    /// Tasks defined in the file
    pub tasks: Vec<Task>,
}

impl CupFile {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, CupError> {
        parser::parse(reader, None)
    }

    pub fn from_reader_with_encoding<R: Read>(
        reader: R,
        encoding: CupEncoding,
    ) -> Result<Self, CupError> {
        parser::parse(reader, Some(encoding))
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, CupError> {
        let file = File::open(path)?;
        Self::from_reader(file)
    }

    pub fn from_path_with_encoding<P: AsRef<Path>>(
        path: P,
        encoding: CupEncoding,
    ) -> Result<Self, CupError> {
        let file = File::open(path)?;
        Self::from_reader_with_encoding(file, encoding)
    }

    pub fn to_writer<W: Write>(&self, writer: W) -> Result<(), CupError> {
        self.to_writer_with_encoding(writer, CupEncoding::Utf8)
    }

    pub fn to_writer_with_encoding<W: Write>(
        &self,
        writer: W,
        encoding: CupEncoding,
    ) -> Result<(), CupError> {
        writer::write(self, writer, encoding)
    }

    pub fn to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), CupError> {
        self.to_path_with_encoding(path, CupEncoding::Utf8)
    }

    pub fn to_path_with_encoding<P: AsRef<Path>>(
        &self,
        path: P,
        encoding: CupEncoding,
    ) -> Result<(), CupError> {
        let file = File::create(path)?;
        self.to_writer_with_encoding(file, encoding)
    }

    pub fn to_string(&self) -> Result<String, CupError> {
        let mut buf = Vec::new();
        self.to_writer(&mut buf)?;
        String::from_utf8(buf).map_err(|e| CupError::Encoding(e.to_string()))
    }
}

impl FromStr for CupFile {
    type Err = CupError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_reader(s.as_bytes())
    }
}
