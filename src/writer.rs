use crate::CupEncoding;
use crate::CupFile;
use crate::error::CupError;
use crate::types::*;
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

fn format_inline_waypoint_line(index: usize, waypoint: &Waypoint) -> Result<String, CupError> {
    // Format: Point=1,"Point_3",PNT_3,,4627.136N,01412.856E,0.0m,1,,,,,,,
    let pics = if waypoint.pics.is_empty() {
        String::new()
    } else {
        waypoint.pics.join(";")
    };

    // Create a CSV writer to properly format the waypoint data
    let mut output = Vec::new();
    {
        let mut csv_writer = Writer::from_writer(&mut output);
        csv_writer.write_record(&[
            &format!("Point={}", index),
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
        csv_writer.flush()?;
    }

    let waypoint_line = String::from_utf8(output).map_err(|e| CupError::Encoding(e.to_string()))?;
    Ok(waypoint_line.trim_end().to_string())
}

fn format_task(task: &Task) -> Result<String, CupError> {
    let mut result = String::new();
    
    // Write the task line with waypoint names
    {
        let mut output = Vec::new();
        let mut csv_writer = Writer::from_writer(&mut output);

        let mut record = vec![task.description.as_deref().unwrap_or("").to_string()];
        
        // Add all waypoint names to the task line
        for name in &task.waypoint_names {
            record.push(name.clone());
        }

        csv_writer.write_record(&record)?;
        csv_writer.flush()?;
        drop(csv_writer); // Explicitly drop to release borrow
        
        let task_line = String::from_utf8(output).map_err(|e| CupError::Encoding(e.to_string()))?;
        result.push_str(task_line.trim_end());
    }
    
    // Write inline waypoints as separate Point= lines
    for (idx, waypoint) in &task.points {
        result.push('\n');
        result.push_str(&format_inline_waypoint_line(*idx as usize, waypoint)?);
    }
    
    Ok(result)
}
