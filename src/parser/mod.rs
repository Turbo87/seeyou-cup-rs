mod column_map;

use crate::CupFile;
use crate::error::CupError;
use crate::parser::column_map::ColumnMap;
use crate::types::{Task, Waypoint};
use crate::{CupEncoding, ObsZoneStyle, ObservationZone, TaskOptions, WaypointStyle};
use csv::StringRecord;
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

fn parse_waypoints(
    csv_iter: &mut csv::StringRecordsIter<&[u8]>,
    column_map: &ColumnMap,
) -> Result<Vec<Waypoint>, CupError> {
    let mut waypoints = Vec::new();
    for result in csv_iter {
        let record = result?;

        let line = record.as_slice();
        if line == TASK_SEPARATOR {
            break;
        }

        let waypoint = parse_waypoint(column_map, &record).map_err(CupError::Parse)?;
        waypoints.push(waypoint);
    }

    Ok(waypoints)
}

fn parse_tasks(
    csv_iter: &mut csv::StringRecordsIter<&[u8]>,
    column_map: &ColumnMap,
) -> Result<Vec<Task>, CupError> {
    let mut tasks = Vec::new();

    let mut csv_iter = csv_iter.peekable();
    'outer: while let Some(result) = csv_iter.next() {
        let record = result?;

        let line = record.as_byte_record().as_slice();
        if line.starts_with(b"Options")
            || line.starts_with(b"ObsZone=")
            || line.starts_with(b"Point=")
            || line.starts_with(b"STARTS=")
        {
            continue;
        }

        let mut task = parse_task_line(&record)?;

        // Look ahead for Options, ObsZone, Point, and STARTS lines
        while let Some(result) = csv_iter.peek() {
            let Ok(record) = result else {
                break 'outer;
            };

            let next_line = record.as_byte_record().as_slice();

            if next_line.starts_with(b"Options") {
                task.options = Some(parse_options_line(record)?);
                csv_iter.next();
            } else if next_line.starts_with(b"ObsZone=") {
                task.observation_zones.push(parse_obszone_line(record)?);
                csv_iter.next();
            } else if next_line.starts_with(b"Point=") {
                let (point_index, inline_waypoint) =
                    parse_inline_waypoint_line_with_index(record, column_map)?;
                // Add the inline waypoint to the points field
                task.points.push((point_index as u32, inline_waypoint));
                csv_iter.next();
            } else if next_line.starts_with(b"STARTS=") {
                task.multiple_starts = parse_starts_line(record)?;
                csv_iter.next();
            } else {
                break;
            }
        }

        tasks.push(task);
    }

    Ok(tasks)
}

fn parse_waypoint(column_map: &ColumnMap, record: &StringRecord) -> Result<Waypoint, String> {
    let name = record.get(column_map.name).unwrap_or_default();
    if name.is_empty() {
        return Err("Name field cannot be empty".into());
    }

    let name = name.to_string();

    let code = record.get(column_map.code).unwrap_or_default().to_string();
    let country = record
        .get(column_map.country)
        .unwrap_or_default()
        .to_string();

    let lat_str = record.get(column_map.lat).unwrap_or_default();
    let latitude = parse_latitude(lat_str)?;

    let lon_str = record.get(column_map.lon).unwrap_or_default();
    let longitude = parse_longitude(lon_str)?;

    let elev_str = record.get(column_map.elev).unwrap_or_default();
    let elevation = elev_str.parse()?;

    let style_str = record.get(column_map.style).unwrap_or_default();
    let style = parse_waypoint_style(style_str)?;

    let runway_direction = column_map.rwdir.and_then(|idx| record.get(idx));
    let runway_direction = runway_direction.filter(|s| !s.is_empty());
    let runway_direction = runway_direction
        .map(|s| {
            s.parse()
                .map_err(|_| format!("Invalid runway direction: {s}"))
        })
        .transpose()?;

    let runway_length = column_map.rwlen.and_then(|idx| record.get(idx));
    let runway_length = runway_length.filter(|s| !s.is_empty());
    let runway_length = runway_length.map(|s| s.parse()).transpose()?;

    let runway_width = column_map.rwwidth.and_then(|idx| record.get(idx));
    let runway_width = runway_width.filter(|s| !s.is_empty());
    let runway_width = runway_width.map(|s| s.parse()).transpose()?;

    let frequency = column_map.freq.and_then(|idx| record.get(idx));
    let frequency = frequency.unwrap_or_default().to_string();

    let description = column_map.desc.and_then(|idx| record.get(idx));
    let description = description.unwrap_or_default().to_string();

    let userdata = column_map.userdata.and_then(|idx| record.get(idx));
    let userdata = userdata.unwrap_or_default().to_string();

    let pictures = column_map.pics.and_then(|idx| record.get(idx));
    let pictures = pictures
        .map(|s| {
            s.split(';')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect()
        })
        .unwrap_or_default();

    Ok(Waypoint {
        name,
        code,
        country,
        latitude,
        longitude,
        elevation,
        style,
        runway_direction,
        runway_length,
        runway_width,
        frequency,
        description,
        userdata,
        pictures,
    })
}

fn parse_latitude(s: &str) -> Result<f64, String> {
    if !s.is_ascii() {
        return Err(format!(
            "Invalid latitude format: {s} (contains non-ASCII characters)"
        ));
    }

    if s.len() != 9 {
        return Err(format!(
            "Invalid latitude format: {s} (expected 9 characters, got {})",
            s.len()
        ));
    }

    let hemisphere = s.chars().last().unwrap();
    let coords = &s[..8];

    // Validate hemisphere
    if hemisphere != 'N' && hemisphere != 'S' {
        return Err(format!("Invalid latitude hemisphere: {hemisphere}"));
    }

    let degrees: f64 = coords[0..2]
        .parse()
        .map_err(|_| format!("Invalid latitude degrees: {s}"))?;

    let minutes: f64 = coords[2..]
        .parse()
        .map_err(|_| format!("Invalid latitude minutes: {s}"))?;

    let mut decimal_degrees = degrees + minutes / 60.0;

    if hemisphere == 'S' {
        decimal_degrees = -decimal_degrees;
    }

    // Validate range
    if !(-90.0..=90.0).contains(&decimal_degrees) {
        return Err(format!(
            "Latitude out of range: {} (must be between -90 and 90)",
            decimal_degrees
        ));
    }

    Ok(decimal_degrees)
}

fn parse_longitude(s: &str) -> Result<f64, String> {
    if !s.is_ascii() {
        return Err(format!(
            "Invalid longitude format: {s} (contains non-ASCII characters)"
        ));
    }

    if s.len() != 10 {
        return Err(format!(
            "Invalid longitude format: {s} (expected 10 characters, got {})",
            s.len()
        ));
    }

    let hemisphere = s.chars().last().unwrap();
    let coords = &s[..9];

    // Validate hemisphere
    if hemisphere != 'E' && hemisphere != 'W' {
        return Err(format!("Invalid longitude hemisphere: {}", hemisphere));
    }

    let degrees: f64 = coords[0..3]
        .parse()
        .map_err(|_| format!("Invalid longitude degrees: {}", s))?;

    let minutes: f64 = coords[3..]
        .parse()
        .map_err(|_| format!("Invalid longitude minutes: {}", s))?;

    let mut decimal_degrees = degrees + minutes / 60.0;

    if hemisphere == 'W' {
        decimal_degrees = -decimal_degrees;
    }

    // Validate range
    if !(-180.0..=180.0).contains(&decimal_degrees) {
        return Err(format!(
            "Longitude out of range: {} (must be between -180 and 180)",
            decimal_degrees
        ));
    }

    Ok(decimal_degrees)
}

fn parse_waypoint_style(s: &str) -> Result<WaypointStyle, String> {
    let value: u8 = s
        .parse()
        .map_err(|_| format!("Invalid waypoint style: {s}"))?;
    Ok(WaypointStyle::from_u8(value))
}

fn parse_task_line(record: &StringRecord) -> Result<Task, CupError> {
    if record.is_empty() {
        return Err(CupError::Parse("Empty task line".to_string()));
    }

    let description = if record.get(0).map(|s| s.is_empty()).unwrap_or(true) {
        None
    } else {
        Some(record.get(0).unwrap().to_string())
    };

    let waypoint_names = record
        .iter()
        .skip(1)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(Task {
        description,
        waypoint_names,
        options: None,
        observation_zones: Vec::new(),
        points: Vec::new(),
        multiple_starts: Vec::new(),
    })
}

fn parse_options_line(record: &StringRecord) -> Result<TaskOptions, CupError> {
    // Options,NoStart=12:34:56,TaskTime=01:45:12,WpDis=False,NearDis=0.7km,NearAlt=300.0m
    let mut options = TaskOptions {
        no_start: None,
        task_time: None,
        wp_dis: None,
        near_dis: None,
        near_alt: None,
        min_dis: None,
        random_order: None,
        max_pts: None,
        before_pts: None,
        after_pts: None,
        bonus: None,
    };

    for part in record.iter().skip(1) {
        if let Some((key, value)) = part.split_once('=') {
            match key {
                "NoStart" => options.no_start = Some(value.to_string()),
                "TaskTime" => options.task_time = Some(value.to_string()),
                "WpDis" => options.wp_dis = Some(value.eq_ignore_ascii_case("true")),
                "NearDis" => options.near_dis = Some(value.parse().map_err(CupError::Parse)?),
                "NearAlt" => options.near_alt = Some(value.parse().map_err(CupError::Parse)?),
                "MinDis" => options.min_dis = Some(value.eq_ignore_ascii_case("true")),
                "RandomOrder" => options.random_order = Some(value.eq_ignore_ascii_case("true")),
                "MaxPts" => options.max_pts = value.parse().ok(),
                "BeforePts" => options.before_pts = value.parse().ok(),
                "AfterPts" => options.after_pts = value.parse().ok(),
                "Bonus" => options.bonus = value.parse().ok(),
                _ => {}
            }
        }
    }

    Ok(options)
}

fn parse_obszone_line(record: &StringRecord) -> Result<ObservationZone, CupError> {
    // ObsZone=0,Style=2,R1=400m,A1=180,Line=1
    let mut index = None;
    let mut style = None;
    let mut r1 = None;
    let mut a1 = None;
    let mut r2 = None;
    let mut a2 = None;
    let mut a12 = None;
    let mut line_val = None;

    for part in record.iter() {
        if let Some((key, value)) = part.split_once('=') {
            match key {
                "ObsZone" => index = value.parse().ok(),
                "Style" => {
                    if let Ok(val) = value.parse::<u8>() {
                        style = ObsZoneStyle::from_u8(val);
                    }
                }
                "R1" => r1 = Some(value.parse().map_err(CupError::Parse)?),
                "A1" => a1 = value.parse().ok(),
                "R2" => r2 = Some(value.parse().map_err(CupError::Parse)?),
                "A2" => a2 = value.parse().ok(),
                "A12" => a12 = value.parse().ok(),
                "Line" => line_val = Some(value == "1" || value.eq_ignore_ascii_case("true")),
                _ => {}
            }
        }
    }

    let index = index.ok_or_else(|| CupError::Parse("Missing ObsZone index".to_string()))?;
    let style = style.ok_or_else(|| CupError::Parse("Missing ObsZone style".to_string()))?;

    Ok(ObservationZone {
        index,
        style,
        r1,
        a1,
        r2,
        a2,
        a12,
        line: line_val,
    })
}

fn parse_starts_line(record: &StringRecord) -> Result<Vec<String>, CupError> {
    // STARTS=Celovec,Hodos,Ratitovec,Jamnik
    Ok(record
        .iter()
        .enumerate()
        .map(|(i, start)| {
            if i == 0 {
                start.strip_prefix("STARTS=").unwrap_or(start)
            } else {
                start
            }
        })
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect())
}

fn parse_inline_waypoint_line_with_index(
    record: &StringRecord,
    column_map: &ColumnMap,
) -> Result<(usize, Waypoint), CupError> {
    // Format: Point=1,"Point_3",PNT_3,,4627.136N,01412.856E,0.0m,1,,,,,,,

    // Extract the point index
    let point_idx_str = record[0].trim_start_matches("Point=");
    let point_index = point_idx_str
        .parse::<usize>()
        .map_err(|_| CupError::Parse(format!("Invalid point index: {}", point_idx_str)))?;

    // Skip the Point=N field and create a proper waypoint record
    let waypoint_record = StringRecord::from(record.iter().skip(1).collect::<Vec<_>>());

    // Parse as a normal waypoint using the same headers as the waypoint section
    let waypoint = parse_waypoint(column_map, &waypoint_record).map_err(CupError::Parse)?;

    Ok((point_index, waypoint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::proptest;

    #[test]
    fn test_latitude_conversion() {
        assert!((parse_latitude("5147.809N").unwrap() - 51.7968166).abs() < 0.0001);
        assert!((parse_latitude("5147.809S").unwrap() - (-51.7968166)).abs() < 0.0001);

        proptest!(|(s in "\\PC*")| { let _ = parse_latitude(&s); });
    }

    #[test]
    fn test_longitude_conversion() {
        assert!((parse_longitude("01410.467E").unwrap() - 14.1744500).abs() < 0.0001);
        assert!((parse_longitude("00405.003W").unwrap() - (-4.0833833)).abs() < 0.0001);

        proptest!(|(s in "\\PC*")| { let _ = parse_longitude(&s); });
    }
}
