use crate::CupEncoding;
use crate::CupFile;
use crate::error::CupError;
use crate::types::*;
use csv::StringRecord;
use encoding_rs::{Encoding, UTF_8, WINDOWS_1252};
use std::borrow::Cow;
use std::io::Read;

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
    let (waypoint_section, task_section) = split_sections(content);

    let waypoints = parse_waypoints(waypoint_section)?;
    let tasks = parse_tasks(task_section)?;

    Ok(CupFile { waypoints, tasks })
}

fn split_sections(content: &str) -> (&str, Option<&str>) {
    if let Some(pos) = content.find("-----Related Tasks-----") {
        (&content[..pos], Some(&content[pos..]))
    } else {
        (content, None)
    }
}

fn parse_waypoints(section: &str) -> Result<Vec<Waypoint>, CupError> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(section.as_bytes());

    let headers = csv_reader.headers()?.clone();

    let mut waypoints = Vec::new();
    for result in csv_reader.records() {
        let record = result?;
        let waypoint = parse_waypoint(&headers, &record)?;
        waypoints.push(waypoint);
    }

    Ok(waypoints)
}

fn parse_tasks(section: Option<&str>) -> Result<Vec<Task>, CupError> {
    let Some(section) = section else {
        return Ok(Vec::new());
    };

    let mut tasks = Vec::new();
    let mut lines = section.lines().skip(1).peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("Options") || trimmed.starts_with("ObsZone=") || trimmed.starts_with("STARTS=") {
            continue;
        }

        let mut task = parse_task_line(line)?;

        // Look ahead for Options, ObsZone, and STARTS lines
        while let Some(next_line) = lines.peek() {
            let next_trimmed = next_line.trim();
            if next_trimmed.is_empty() {
                lines.next();
                continue;
            }

            if next_trimmed.starts_with("Options") {
                task.options = Some(parse_options_line(next_line)?);
                lines.next();
            } else if next_trimmed.starts_with("ObsZone=") {
                task.observation_zones.push(parse_obszone_line(next_line)?);
                lines.next();
            } else if next_trimmed.starts_with("STARTS=") {
                task.multiple_starts = parse_starts_line(next_line)?;
                lines.next();
            } else {
                break;
            }
        }

        tasks.push(task);
    }

    Ok(tasks)
}

fn parse_waypoint(headers: &StringRecord, record: &StringRecord) -> Result<Waypoint, CupError> {
    let get_field = |key: &str| -> Option<&str> {
        headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case(key))
            .and_then(|idx| record.get(idx))
    };

    let name = get_field("name")
        .ok_or_else(|| CupError::Parse("Missing 'name' field".to_string()))?;

    if name.is_empty() {
        return Err(CupError::Parse("Name field cannot be empty".to_string()));
    }

    let name = name.to_string();

    let code = get_field("code").unwrap_or_default().to_string();
    let country = get_field("country").unwrap_or_default().to_string();

    let lat_str =
        get_field("lat").ok_or_else(|| CupError::Parse("Missing 'lat' field".to_string()))?;
    let lat = parse_latitude(lat_str)?;

    let lon_str =
        get_field("lon").ok_or_else(|| CupError::Parse("Missing 'lon' field".to_string()))?;
    let lon = parse_longitude(lon_str)?;

    let elev_str =
        get_field("elev").ok_or_else(|| CupError::Parse("Missing 'elev' field".to_string()))?;
    let elev = parse_elevation(elev_str)?;

    let style_str =
        get_field("style").ok_or_else(|| CupError::Parse("Missing 'style' field".to_string()))?;
    let style = parse_waypoint_style(style_str)?;

    let runway_dir = get_field("rwdir")
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<u16>()
                .map_err(|_| CupError::Parse(format!("Invalid runway direction: {}", s)))
        })
        .transpose()?;

    let runway_len = get_field("rwlen")
        .filter(|s| !s.is_empty())
        .map(parse_runway_dimension)
        .transpose()?;

    let runway_width = get_field("rwwidth")
        .filter(|s| !s.is_empty())
        .map(parse_runway_dimension)
        .transpose()?;

    let freq = get_field("freq")
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let desc = get_field("desc")
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let userdata = get_field("userdata")
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let pics = get_field("pics")
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
        lat,
        lon,
        elev,
        style,
        runway_dir,
        runway_len,
        runway_width,
        freq,
        desc,
        userdata,
        pics,
    })
}

fn parse_latitude(s: &str) -> Result<f64, CupError> {
    if s.len() != 9 {
        return Err(CupError::Parse(format!(
            "Invalid latitude format: {} (expected 9 characters, got {})",
            s,
            s.len()
        )));
    }

    let hemisphere = s.chars().last().unwrap();
    let coords = &s[..s.len() - 1];

    let degrees: f64 = coords[0..2]
        .parse()
        .map_err(|_| CupError::Parse(format!("Invalid latitude degrees: {}", s)))?;

    let minutes: f64 = coords[2..]
        .parse()
        .map_err(|_| CupError::Parse(format!("Invalid latitude minutes: {}", s)))?;

    let mut decimal_degrees = degrees + minutes / 60.0;

    if hemisphere == 'S' {
        decimal_degrees = -decimal_degrees;
    }

    Ok(decimal_degrees)
}

fn parse_longitude(s: &str) -> Result<f64, CupError> {
    if s.len() != 10 {
        return Err(CupError::Parse(format!(
            "Invalid longitude format: {} (expected 10 characters, got {})",
            s,
            s.len()
        )));
    }

    let hemisphere = s.chars().last().unwrap();
    let coords = &s[..s.len() - 1];

    let degrees: f64 = coords[0..3]
        .parse()
        .map_err(|_| CupError::Parse(format!("Invalid longitude degrees: {}", s)))?;

    let minutes: f64 = coords[3..]
        .parse()
        .map_err(|_| CupError::Parse(format!("Invalid longitude minutes: {}", s)))?;

    let mut decimal_degrees = degrees + minutes / 60.0;

    if hemisphere == 'W' {
        decimal_degrees = -decimal_degrees;
    }

    Ok(decimal_degrees)
}

fn parse_elevation(s: &str) -> Result<Elevation, CupError> {
    let s = s.trim();

    if s.ends_with("ft") {
        let value_str = &s[..s.len() - 2];
        let value: f64 = value_str
            .parse()
            .map_err(|_| CupError::Parse(format!("Invalid elevation value: {}", s)))?;
        Ok(Elevation::Feet(value))
    } else if s.ends_with('m') {
        let value_str = &s[..s.len() - 1];
        let value: f64 = value_str
            .parse()
            .map_err(|_| CupError::Parse(format!("Invalid elevation value: {}", s)))?;
        Ok(Elevation::Meters(value))
    } else {
        let value: f64 = s
            .parse()
            .map_err(|_| CupError::Parse(format!("Invalid elevation value: {}", s)))?;
        Ok(Elevation::Meters(value))
    }
}

fn parse_runway_dimension(s: &str) -> Result<RunwayDimension, CupError> {
    let s = s.trim();

    if s.ends_with("nm") {
        let value_str = &s[..s.len() - 2];
        let value: f64 = value_str
            .parse()
            .map_err(|_| CupError::Parse(format!("Invalid runway dimension: {}", s)))?;
        Ok(RunwayDimension::NauticalMiles(value))
    } else if s.ends_with("ml") {
        let value_str = &s[..s.len() - 2];
        let value: f64 = value_str
            .parse()
            .map_err(|_| CupError::Parse(format!("Invalid runway dimension: {}", s)))?;
        Ok(RunwayDimension::StatuteMiles(value))
    } else if s.ends_with('m') {
        let value_str = &s[..s.len() - 1];
        let value: f64 = value_str
            .parse()
            .map_err(|_| CupError::Parse(format!("Invalid runway dimension: {}", s)))?;
        Ok(RunwayDimension::Meters(value))
    } else {
        let value: f64 = s
            .parse()
            .map_err(|_| CupError::Parse(format!("Invalid runway dimension: {}", s)))?;
        Ok(RunwayDimension::Meters(value))
    }
}

fn parse_waypoint_style(s: &str) -> Result<WaypointStyle, CupError> {
    let value: u8 = s
        .parse()
        .map_err(|_| CupError::Parse(format!("Invalid waypoint style: {}", s)))?;
    Ok(WaypointStyle::from_u8(value))
}

fn parse_task_line(line: &str) -> Result<Task, CupError> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(line.as_bytes());
    let mut records = csv_reader.records();

    let record = records
        .next()
        .ok_or_else(|| CupError::Parse("Empty task line".to_string()))??;

    if record.is_empty() {
        return Err(CupError::Parse("Empty task line".to_string()));
    }

    let description = if record.get(0).map(|s| s.is_empty()).unwrap_or(true) {
        None
    } else {
        Some(record.get(0).unwrap().to_string())
    };

    let waypoints = record
        .iter()
        .skip(1)
        .filter(|s| !s.is_empty())
        .map(|s| TaskPoint::Reference(s.to_string()))
        .collect();

    Ok(Task {
        description,
        waypoints,
        options: None,
        observation_zones: Vec::new(),
        multiple_starts: Vec::new(),
    })
}

fn parse_options_line(line: &str) -> Result<TaskOptions, CupError> {
    // Options,NoStart=12:34:56,TaskTime=01:45:12,WpDis=False,NearDis=0.7km,NearAlt=300.0m
    let parts: Vec<&str> = line.split(',').collect();

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

    for part in parts.iter().skip(1) {
        if let Some((key, value)) = part.split_once('=') {
            match key {
                "NoStart" => options.no_start = Some(value.to_string()),
                "TaskTime" => options.task_time = Some(value.to_string()),
                "WpDis" => options.wp_dis = Some(value.eq_ignore_ascii_case("true")),
                "NearDis" => options.near_dis = Some(parse_distance(value)?),
                "NearAlt" => options.near_alt = Some(parse_elevation(value)?),
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

fn parse_distance(s: &str) -> Result<Distance, CupError> {
    let s = s.trim();

    if s.ends_with("km") {
        let value_str = &s[..s.len() - 2];
        let value: f64 = value_str
            .parse()
            .map_err(|_| CupError::Parse(format!("Invalid distance value: {}", s)))?;
        Ok(Distance::Kilometers(value))
    } else if s.ends_with("nm") {
        let value_str = &s[..s.len() - 2];
        let value: f64 = value_str
            .parse()
            .map_err(|_| CupError::Parse(format!("Invalid distance value: {}", s)))?;
        Ok(Distance::NauticalMiles(value))
    } else if s.ends_with("ml") {
        let value_str = &s[..s.len() - 2];
        let value: f64 = value_str
            .parse()
            .map_err(|_| CupError::Parse(format!("Invalid distance value: {}", s)))?;
        Ok(Distance::StatuteMiles(value))
    } else if s.ends_with('m') {
        let value_str = &s[..s.len() - 1];
        let value: f64 = value_str
            .parse()
            .map_err(|_| CupError::Parse(format!("Invalid distance value: {}", s)))?;
        Ok(Distance::Meters(value))
    } else {
        let value: f64 = s
            .parse()
            .map_err(|_| CupError::Parse(format!("Invalid distance value: {}", s)))?;
        Ok(Distance::Meters(value))
    }
}

fn parse_obszone_line(line: &str) -> Result<ObservationZone, CupError> {
    // ObsZone=0,Style=2,R1=400m,A1=180,Line=1
    let parts: Vec<&str> = line.split(',').collect();

    let mut index = None;
    let mut style = None;
    let mut r1 = None;
    let mut a1 = None;
    let mut r2 = None;
    let mut a2 = None;
    let mut a12 = None;
    let mut line_val = None;

    for part in parts.iter() {
        if let Some((key, value)) = part.split_once('=') {
            match key {
                "ObsZone" => index = value.parse().ok(),
                "Style" => {
                    if let Ok(val) = value.parse::<u8>() {
                        style = ObsZoneStyle::from_u8(val);
                    }
                }
                "R1" => r1 = Some(parse_runway_dimension(value)?),
                "A1" => a1 = value.parse().ok(),
                "R2" => r2 = Some(parse_runway_dimension(value)?),
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

fn parse_starts_line(line: &str) -> Result<Vec<String>, CupError> {
    // STARTS=Celovec,Hodos,Ratitovec,Jamnik
    if let Some(starts_part) = line.strip_prefix("STARTS=") {
        Ok(starts_part
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    } else {
        Ok(Vec::new())
    }
}
