use crate::CupFile;
use crate::error::CupError;
use crate::types::{Task, Waypoint};
use crate::{CupEncoding, ObservationZone, TaskOptions};
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

    csv_writer.write_record([
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
    let pics = if waypoint.pictures.is_empty() {
        String::new()
    } else {
        waypoint.pictures.join(";")
    };

    writer.write_record([
        &waypoint.name,
        &waypoint.code,
        &waypoint.country,
        &format_latitude(waypoint.latitude),
        &format_longitude(waypoint.longitude),
        &waypoint.elevation.to_string(),
        &(waypoint.style as u8).to_string(),
        &waypoint
            .runway_direction
            .map(|d| format!("{:03}", d))
            .unwrap_or_default(),
        &waypoint
            .runway_length
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
        &waypoint
            .runway_width
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
        &waypoint.frequency,
        &waypoint.description,
        &waypoint.userdata,
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

fn format_inline_waypoint_line(index: usize, waypoint: &Waypoint) -> Result<String, CupError> {
    // Format: Point=1,"Point_3",PNT_3,,4627.136N,01412.856E,0.0m,1,,,,,,,
    let pics = if waypoint.pictures.is_empty() {
        String::new()
    } else {
        waypoint.pictures.join(";")
    };

    // Create a CSV writer to properly format the waypoint data
    let mut output = Vec::new();
    {
        let mut csv_writer = Writer::from_writer(&mut output);
        csv_writer.write_record([
            &format!("Point={}", index),
            &waypoint.name,
            &waypoint.code,
            &waypoint.country,
            &format_latitude(waypoint.latitude),
            &format_longitude(waypoint.longitude),
            &waypoint.elevation.to_string(),
            &(waypoint.style as u8).to_string(),
            &waypoint
                .runway_direction
                .map(|d| format!("{:03}", d))
                .unwrap_or_default(),
            &waypoint
                .runway_length
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            &waypoint
                .runway_width
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            &waypoint.frequency,
            &waypoint.description,
            &waypoint.userdata,
            &pics,
        ])?;
        csv_writer.flush()?;
    }

    let waypoint_line = String::from_utf8(output).map_err(|e| CupError::Encoding(e.to_string()))?;
    Ok(waypoint_line.trim_end().to_string())
}

fn format_task_options(options: &TaskOptions) -> Result<String, CupError> {
    let mut parts = vec!["Options".to_string()];

    if let Some(no_start) = &options.no_start {
        parts.push(format!("NoStart={}", no_start));
    }
    if let Some(task_time) = &options.task_time {
        parts.push(format!("TaskTime={}", task_time));
    }
    if let Some(wp_dis) = options.wp_dis {
        parts.push(format!("WpDis={}", if wp_dis { "True" } else { "False" }));
    }
    if let Some(near_dis) = &options.near_dis {
        parts.push(format!("NearDis={near_dis}"));
    }
    if let Some(near_alt) = &options.near_alt {
        parts.push(format!("NearAlt={near_alt}"));
    }
    if let Some(min_dis) = options.min_dis {
        parts.push(format!("MinDis={}", if min_dis { "True" } else { "False" }));
    }
    if let Some(random_order) = options.random_order {
        parts.push(format!(
            "RandomOrder={}",
            if random_order { "True" } else { "False" }
        ));
    }
    if let Some(max_pts) = options.max_pts {
        parts.push(format!("MaxPts={}", max_pts));
    }
    if let Some(before_pts) = options.before_pts {
        parts.push(format!("BeforePts={}", before_pts));
    }
    if let Some(after_pts) = options.after_pts {
        parts.push(format!("AfterPts={}", after_pts));
    }
    if let Some(bonus) = options.bonus {
        parts.push(format!("Bonus={}", bonus));
    }

    Ok(parts.join(","))
}

fn format_observation_zone(obs_zone: &ObservationZone) -> Result<String, CupError> {
    let mut parts = vec![
        format!("ObsZone={}", obs_zone.index),
        format!("Style={}", obs_zone.style as u8),
    ];

    if let Some(r1) = &obs_zone.r1 {
        parts.push(format!("R1={r1}"));
    }
    if let Some(a1) = obs_zone.a1 {
        parts.push(format!("A1={}", a1));
    }
    if let Some(r2) = &obs_zone.r2 {
        parts.push(format!("R2={r2}"));
    }
    if let Some(a2) = obs_zone.a2 {
        parts.push(format!("A2={}", a2));
    }
    if let Some(a12) = obs_zone.a12 {
        parts.push(format!("A12={}", a12));
    }
    if let Some(line) = obs_zone.line {
        parts.push(format!("Line={}", if line { "True" } else { "False" }));
    }

    Ok(parts.join(","))
}

fn format_multiple_starts(starts: &[String]) -> Result<String, CupError> {
    // Format: STARTS="Start1","Start2","Start3"
    let quoted_starts: Vec<String> = starts.iter().map(|s| format!("\"{}\"", s)).collect();
    Ok(format!("STARTS={}", quoted_starts.join(",")))
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

    // Write task options if present
    if let Some(options) = &task.options {
        result.push('\n');
        result.push_str(&format_task_options(options)?);
    }

    // Write observation zones
    for obs_zone in &task.observation_zones {
        result.push('\n');
        result.push_str(&format_observation_zone(obs_zone)?);
    }

    // Write inline waypoints as separate Point= lines
    for (idx, waypoint) in &task.points {
        result.push('\n');
        result.push_str(&format_inline_waypoint_line(*idx as usize, waypoint)?);
    }

    // Write multiple starts if present
    if !task.multiple_starts.is_empty() {
        result.push('\n');
        result.push_str(&format_multiple_starts(&task.multiple_starts)?);
    }

    Ok(result)
}
