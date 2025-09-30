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
    for line in section.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("Options")
            || trimmed.starts_with("ObsZone=")
            || trimmed.starts_with("STARTS=")
        {
            continue;
        }

        let task = parse_task_line(line)?;
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
        .ok_or_else(|| CupError::Parse("Missing 'name' field".to_string()))?
        .to_string();

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
    if s.len() < 9 {
        return Err(CupError::Parse(format!("Invalid latitude format: {}", s)));
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
    if s.len() < 10 {
        return Err(CupError::Parse(format!("Invalid longitude format: {}", s)));
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
