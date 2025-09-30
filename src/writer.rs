use crate::error::CupError;
use crate::types::*;
use crate::CupEncoding;
use crate::CupFile;
use encoding_rs::{Encoding, UTF_8, WINDOWS_1252};
use std::io::Write;

pub fn write<W: Write>(cup_file: &CupFile, mut writer: W, encoding: CupEncoding) -> Result<(), CupError> {
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
    let mut output = String::new();

    output.push_str("name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics\n");

    for waypoint in &cup_file.waypoints {
        output.push_str(&format_waypoint(waypoint));
        output.push('\n');
    }

    if !cup_file.tasks.is_empty() {
        output.push_str("-----Related Tasks-----\n");

        for task in &cup_file.tasks {
            output.push_str(&format_task(task));
            output.push('\n');
        }
    }

    Ok(output)
}

fn format_waypoint(waypoint: &Waypoint) -> String {
    let mut fields = Vec::new();

    fields.push(quote_if_needed(&waypoint.name));
    fields.push(quote_if_needed(&waypoint.code));
    fields.push(quote_if_needed(&waypoint.country));
    fields.push(format_latitude(waypoint.lat));
    fields.push(format_longitude(waypoint.lon));
    fields.push(format_elevation(&waypoint.elev));
    fields.push((waypoint.style as u8).to_string());
    fields.push(
        waypoint
            .runway_dir
            .map(|d| format!("{:03}", d))
            .unwrap_or_default(),
    );
    fields.push(
        waypoint
            .runway_len
            .as_ref()
            .map(format_runway_dimension)
            .unwrap_or_default(),
    );
    fields.push(
        waypoint
            .runway_width
            .as_ref()
            .map(format_runway_dimension)
            .unwrap_or_default(),
    );
    fields.push(waypoint.freq.as_deref().unwrap_or("").to_string());
    fields.push(quote_if_needed(waypoint.desc.as_deref().unwrap_or("")));
    fields.push(quote_if_needed(waypoint.userdata.as_deref().unwrap_or("")));

    let pics = if waypoint.pics.is_empty() {
        String::new()
    } else {
        quote_if_needed(&waypoint.pics.join(";"))
    };
    fields.push(pics);

    fields.join(",")
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

fn quote_if_needed(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }

    if s.contains(',') || s.contains('"') || s.contains('\n') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}

fn format_task(task: &Task) -> String {
    let mut fields = Vec::new();

    fields.push(quote_if_needed(task.description.as_deref().unwrap_or("")));

    for waypoint in &task.waypoints {
        match waypoint {
            TaskPoint::Reference(name) => fields.push(quote_if_needed(name)),
            TaskPoint::Inline(_) => {
                fields.push(String::new());
            }
        }
    }

    fields.join(",")
}
