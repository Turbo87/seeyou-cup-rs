#![doc = include_str!("../README.md")]

mod error;
mod parser;
pub mod spec;
mod types;
mod writer;

pub use error::{Error, ParseIssue};
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
    pub fn from_reader<R: Read>(reader: R) -> Result<(Self, Vec<ParseIssue>), Error> {
        parser::parse(reader, None)
    }

    pub fn from_reader_with_encoding<R: Read>(
        reader: R,
        encoding: CupEncoding,
    ) -> Result<(Self, Vec<ParseIssue>), Error> {
        parser::parse(reader, Some(encoding))
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<(Self, Vec<ParseIssue>), Error> {
        let file = File::open(path)?;
        Self::from_reader(file)
    }

    pub fn from_path_with_encoding<P: AsRef<Path>>(
        path: P,
        encoding: CupEncoding,
    ) -> Result<(Self, Vec<ParseIssue>), Error> {
        let file = File::open(path)?;
        Self::from_reader_with_encoding(file, encoding)
    }

    // The trait can't be implemented for `(Self, Vec<ParseIssue>)`
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<(Self, Vec<ParseIssue>), Error> {
        Self::from_reader(s.as_bytes())
    }

    pub fn to_writer<W: Write>(&self, writer: W) -> Result<(), Error> {
        self.to_writer_with_encoding(writer, CupEncoding::Utf8)
    }

    pub fn to_writer_with_encoding<W: Write>(
        &self,
        writer: W,
        encoding: CupEncoding,
    ) -> Result<(), Error> {
        writer::write(self, writer, encoding)
    }

    pub fn to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        self.to_path_with_encoding(path, CupEncoding::Utf8)
    }

    pub fn to_path_with_encoding<P: AsRef<Path>>(
        &self,
        path: P,
        encoding: CupEncoding,
    ) -> Result<(), Error> {
        let file = File::create(path)?;
        self.to_writer_with_encoding(file, encoding)
    }

    pub fn to_string(&self) -> Result<String, Error> {
        let mut buf = Vec::new();
        self.to_writer(&mut buf)?;
        String::from_utf8(buf).map_err(|e| Error::Encoding(e.to_string()))
    }
}
