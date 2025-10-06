mod basics;
mod task;
mod waypoint;

use crate::Encoding;
use crate::CupFile;
use crate::error::Error;
use crate::writer::task::format_task;
use crate::writer::waypoint::write_waypoint;
use csv::Writer;
use encoding_rs::{Encoding as EncodingImpl, UTF_8, WINDOWS_1252};
use std::io::Write;

pub fn write<W: Write>(
    cup_file: &CupFile,
    mut writer: W,
    encoding: Encoding,
) -> Result<(), Error> {
    let content = format_cup_file(cup_file)?;

    let encoding_impl: &'static EncodingImpl = match encoding {
        Encoding::Utf8 => UTF_8,
        Encoding::Windows1252 => WINDOWS_1252,
    };

    let (encoded_bytes, _, had_errors) = encoding_impl.encode(&content);
    if had_errors {
        return Err(Error::Encoding(format!(
            "Failed to encode with {:?}",
            encoding
        )));
    }

    writer.write_all(&encoded_bytes)?;
    Ok(())
}

fn format_cup_file(cup_file: &CupFile) -> Result<String, Error> {
    let mut output = Vec::new();
    let mut csv_writer = Writer::from_writer(&mut output);

    csv_writer.write_record([
        "name", "code", "country", "lat", "lon", "elev", "style", "rwdir", "rwlen", "rwwidth",
        "freq", "desc", "userdata", "pics",
    ])?;

    for waypoint in &cup_file.waypoints {
        write_waypoint(&mut csv_writer, waypoint)?;
    }

    csv_writer.flush()?;
    drop(csv_writer);

    let mut result = String::from_utf8(output).map_err(|e| Error::Encoding(e.to_string()))?;

    if !cup_file.tasks.is_empty() {
        result.push_str("-----Related Tasks-----\n");

        for task in &cup_file.tasks {
            result.push_str(&format_task(task)?);
            result.push('\n');
        }
    }

    Ok(result)
}
