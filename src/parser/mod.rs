mod basics;
mod column_map;
mod task;
mod waypoint;

use crate::CupEncoding;
use crate::CupFile;
use crate::error::CupError;
use crate::parser::column_map::ColumnMap;
use crate::parser::task::parse_tasks;
use crate::parser::waypoint::parse_waypoints;
use encoding_rs::{Encoding, UTF_8, WINDOWS_1252};
use std::borrow::Cow;
use std::io::Read;

pub const TASK_SEPARATOR: &str = "-----Related Tasks-----";

pub fn parse<R: Read>(mut reader: R, encoding: Option<CupEncoding>) -> Result<CupFile, CupError> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    let content = match encoding {
        Some(enc) => decode_with_encoding(&bytes, enc)?,
        None => decode_auto(&bytes)?,
    };

    parse_content(&content)
}

fn decode_with_encoding(bytes: &[u8], encoding: CupEncoding) -> Result<Cow<'_, str>, CupError> {
    let encoding_impl: &'static Encoding = match encoding {
        CupEncoding::Utf8 => UTF_8,
        CupEncoding::Windows1252 => WINDOWS_1252,
    };

    let (content, _, had_errors) = encoding_impl.decode(bytes);
    if had_errors {
        return Err(CupError::Encoding(format!(
            "Failed to decode with {:?}",
            encoding
        )));
    }

    Ok(content)
}

fn decode_auto(bytes: &[u8]) -> Result<Cow<'_, str>, CupError> {
    // Try UTF-8 first (strict)
    match std::str::from_utf8(bytes) {
        Ok(s) => Ok(s.into()),
        Err(_) => {
            // Fall back to Windows-1252 (never fails, maps all bytes)
            let (content, _, _) = WINDOWS_1252.decode(bytes);
            Ok(content)
        }
    }
}

fn parse_content(content: &str) -> Result<CupFile, CupError> {
    let content = content.trim();
    if content.is_empty() {
        return Err(CupError::Parse("Empty file".to_string()));
    }

    let mut csv_reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(content.as_bytes());

    let headers = csv_reader.headers()?;
    let column_map = ColumnMap::try_from(headers).map_err(CupError::Parse)?;

    let mut csv_iter = csv_reader.records();
    let waypoints = parse_waypoints(&mut csv_iter, &column_map)?;
    let tasks = parse_tasks(&mut csv_iter, &column_map)?;

    Ok(CupFile { waypoints, tasks })
}
