use crate::error::CupError;
use crate::types::*;
use crate::CupEncoding;
use crate::CupFile;
use csv::Writer;
use encoding_rs::{Encoding, UTF_8, WINDOWS_1252};
use std::io::Write;

pub fn write<W: Write>(
    cup_file: &CupFile,
    mut writer: W,
    encoding: CupEncoding,
) -> Result<(), CupError> {
    let content = format_cup_file(cup_file)?;

    let encoding_impl: &'static Encoding = match encoding {
        CupEncoding::Utf8 => UTF_8,
        CupEncoding::Windows1252 => WINDOWS_1252,
    };

    let (encoded_bytes, _, had_errors) = encoding_impl.encode(&content);
    if had_errors {
        return Err(CupError::Encoding(format!(
            "Failed to encode with {:?}",
            encoding
        )));
    }

    writer.write_all(&encoded_bytes)?;
    Ok(())
}

fn format_cup_file(cup_file: &CupFile) -> Result<String, CupError> {
    let mut output = Vec::new();
    let mut csv_writer = Writer::from_writer(&mut output);

    csv_writer.write_record(&[
        "name", "code", "country", "lat", "lon", "elev", "style", "rwdir", "rwlen", "rwwidth",
        "freq", "desc", "userdata", "pics",
    ])?;

    for waypoint in &cup_file.waypoints {
        write_waypoint(&mut csv_writer, waypoint)?;
    }

    csv_writer.flush()?;
    drop(csv_writer);

    let mut result = String::from_utf8(output).map_err(|e| CupError::Encoding(e.to_string()))?;

    if !cup_file.tasks.is_empty() {
        result.push_str("-----Related Tasks-----\n");

        for task in &cup_file.tasks {
            result.push_str(&format_task(task)?);
            result.push('\n');
        }
    }

    Ok(result)
}

fn write_waypoint<W: std::io::Write>(
    writer: &mut Writer<W>,
    waypoint: &Waypoint,
) -> Result<(), CupError> {
    let pics = if waypoint.pics.is_empty() {
        String::new()
    } else {
        waypoint.pics.join(";")
    };

    writer.write_record(&[
        &waypoint.name,
        &waypoint.code,
        &waypoint.country,
        &format_latitude(waypoint.lat),
        &format_longitude(waypoint.lon),
        &format_elevation(&waypoint.elev),
        &(waypoint.style as u8).to_string(),
        &waypoint
            .runway_dir
            .map(|d| format!("{:03}", d))
            .unwrap_or_default(),
        &waypoint
            .runway_len
            .as_ref()
            .map(format_runway_dimension)
            .unwrap_or_default(),
        &waypoint
            .runway_width
            .as_ref()
            .map(format_runway_dimension)
            .unwrap_or_default(),
        waypoint.freq.as_deref().unwrap_or(""),
        waypoint.desc.as_deref().unwrap_or(""),
        waypoint.userdata.as_deref().unwrap_or(""),
        &pics,
    ])?;

    Ok(())
}

fn format_latitude(lat: f64) -> String {
    let hemisphere = if lat >= 0.0 { 'N' } else { 'S' };
    let abs_lat = lat.abs();
    let degrees = abs_lat.floor() as u32;
    let minutes = (abs_lat - degrees as f64) * 60.0;
    format!("{:02}{:06.3}{}", degrees, minutes, hemisphere)
}

fn format_longitude(lon: f64) -> String {
    let hemisphere = if lon >= 0.0 { 'E' } else { 'W' };
    let abs_lon = lon.abs();
    let degrees = abs_lon.floor() as u32;
    let minutes = (abs_lon - degrees as f64) * 60.0;
    format!("{:03}{:06.3}{}", degrees, minutes, hemisphere)
}

fn format_elevation(elev: &Elevation) -> String {
    match elev {
        Elevation::Meters(m) => format!("{}m", m),
        Elevation::Feet(ft) => format!("{}ft", ft),
    }
}

fn format_runway_dimension(dim: &RunwayDimension) -> String {
    match dim {
        RunwayDimension::Meters(m) => format!("{}m", m),
        RunwayDimension::NauticalMiles(nm) => format!("{}nm", nm),
        RunwayDimension::StatuteMiles(mi) => format!("{}ml", mi),
    }
}

fn format_task(task: &Task) -> Result<String, CupError> {
    let mut output = Vec::new();
    {
        let mut csv_writer = Writer::from_writer(&mut output);

        let mut record = vec![task.description.as_deref().unwrap_or("")];

        for waypoint in &task.waypoints {
            match waypoint {
                TaskPoint::Reference(name) => record.push(name),
                TaskPoint::Inline(_) => record.push(""),
            }
        }

        csv_writer.write_record(&record)?;
        csv_writer.flush()?;
    }

    let result = String::from_utf8(output).map_err(|e| CupError::Encoding(e.to_string()))?;
    Ok(result.trim_end().to_string())
}
