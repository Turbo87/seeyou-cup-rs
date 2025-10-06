mod basics;
mod column_map;
mod task;
mod waypoint;

use crate::CupFile;
use crate::Encoding;
use crate::error::{Error, ParseIssue, Warning};
use crate::parser::column_map::ColumnMap;
use crate::parser::task::parse_tasks;
use crate::parser::waypoint::parse_waypoints;
use encoding_rs::{Encoding as EncodingImpl, UTF_8, WINDOWS_1252};
use std::borrow::Cow;
use std::io::Read;

pub const TASK_SEPARATOR: &str = "-----Related Tasks-----";

pub fn parse<R: Read>(
    mut reader: R,
    encoding: Option<Encoding>,
) -> Result<(CupFile, Vec<Warning>), Error> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    let content = match encoding {
        Some(enc) => decode_with_encoding(&bytes, enc)?,
        None => decode_auto(&bytes)?,
    };

    parse_content(&content)
}

fn decode_with_encoding(bytes: &[u8], encoding: Encoding) -> Result<Cow<'_, str>, Error> {
    let encoding_impl: &'static EncodingImpl = match encoding {
        Encoding::Utf8 => UTF_8,
        Encoding::Windows1252 => WINDOWS_1252,
    };

    let (content, _, _had_errors) = encoding_impl.decode(bytes);
    Ok(content)
}

fn decode_auto(bytes: &[u8]) -> Result<Cow<'_, str>, Error> {
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

fn parse_content(content: &str) -> Result<(CupFile, Vec<Warning>), Error> {
    let content = content.trim();
    if content.is_empty() {
        return Err(ParseIssue::new("Empty file").into());
    }

    let mut warnings = Vec::new();

    let mut csv_reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(content.as_bytes());

    let headers = csv_reader.headers()?;
    let column_map = ColumnMap::try_from(headers)
        .map_err(|error| ParseIssue::new(error).with_record(headers))?;

    let mut csv_iter = csv_reader.records();
    let waypoints = parse_waypoints(&mut csv_iter, &column_map, &mut warnings)?;
    let tasks = parse_tasks(&mut csv_iter, &column_map, &mut warnings)?;

    Ok((CupFile { waypoints, tasks }, warnings))
}
