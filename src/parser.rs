use crate::error::CupError;
use crate::types::*;
use crate::CupEncoding;
use crate::CupFile;
use encoding_rs::{Encoding, UTF_8, WINDOWS_1252};
use std::collections::HashMap;
use std::io::Read;

pub fn parse<R: Read>(mut reader: R, encoding: CupEncoding) -> Result<CupFile, CupError> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    let encoding_impl: &'static Encoding = match encoding {
        CupEncoding::Utf8 => UTF_8,
        CupEncoding::Windows1252 => WINDOWS_1252,
    };

    let (content, _, had_errors) = encoding_impl.decode(&bytes);
    if had_errors {
        return Err(CupError::Encoding(format!(
            "Failed to decode with {:?}",
            encoding
        )));
    }

    parse_content(&content)
}

fn parse_content(content: &str) -> Result<CupFile, CupError> {
    let mut lines = content.lines();

    let header_line = lines
        .next()
        .ok_or_else(|| CupError::Parse("Empty file".to_string()))?;

    let headers = parse_csv_line(header_line);
    let column_map = build_column_map(&headers)?;

    let mut waypoints = Vec::new();
    let mut tasks = Vec::new();
    let mut in_tasks_section = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "-----Related Tasks-----" {
            in_tasks_section = true;
            continue;
        }

        if !in_tasks_section {
            let waypoint = parse_waypoint(line, &column_map)?;
            waypoints.push(waypoint);
        } else {
            if trimmed.starts_with("Options") || trimmed.starts_with("ObsZone=") || trimmed.starts_with("STARTS=") {
                continue;
            }
            let task = parse_task(line)?;
            tasks.push(task);
        }
    }

    Ok(CupFile { waypoints, tasks })
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' if !in_quotes => {
                in_quotes = true;
            }
            '"' if in_quotes => {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            }
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }

    fields.push(current.trim().to_string());
    fields
}

fn build_column_map(headers: &[String]) -> Result<HashMap<String, usize>, CupError> {
    let mut map = HashMap::new();
    for (idx, header) in headers.iter().enumerate() {
        map.insert(header.to_lowercase(), idx);
    }
    Ok(map)
}

fn parse_waypoint(line: &str, column_map: &HashMap<String, usize>) -> Result<Waypoint, CupError> {
    let fields = parse_csv_line(line);

    let get_field = |key: &str| -> Option<String> {
        column_map
            .get(key)
            .and_then(|&idx| fields.get(idx))
            .map(|s| s.to_string())
    };

    let name = get_field("name")
        .ok_or_else(|| CupError::Parse("Missing 'name' field".to_string()))?;

    let code = get_field("code").unwrap_or_default();
    let country = get_field("country").unwrap_or_default();

    let lat_str = get_field("lat")
        .ok_or_else(|| CupError::Parse("Missing 'lat' field".to_string()))?;
    let lat = parse_latitude(&lat_str)?;

    let lon_str = get_field("lon")
        .ok_or_else(|| CupError::Parse("Missing 'lon' field".to_string()))?;
    let lon = parse_longitude(&lon_str)?;

    let elev_str = get_field("elev")
        .ok_or_else(|| CupError::Parse("Missing 'elev' field".to_string()))?;
    let elev = parse_elevation(&elev_str)?;

    let style_str = get_field("style")
        .ok_or_else(|| CupError::Parse("Missing 'style' field".to_string()))?;
    let style = parse_waypoint_style(&style_str)?;

    let runway_dir = get_field("rwdir")
        .and_then(|s| if s.is_empty() { None } else { Some(s) })
        .map(|s| {
            s.parse::<u16>()
                .map_err(|_| CupError::Parse(format!("Invalid runway direction: {}", s)))
        })
        .transpose()?;

    let runway_len = get_field("rwlen")
        .and_then(|s| if s.is_empty() { None } else { Some(s) })
        .map(|s| parse_runway_dimension(&s))
        .transpose()?;

    let runway_width = get_field("rwwidth")
        .and_then(|s| if s.is_empty() { None } else { Some(s) })
        .map(|s| parse_runway_dimension(&s))
        .transpose()?;

    let freq = get_field("freq").and_then(|s| if s.is_empty() { None } else { Some(s) });

    let desc = get_field("desc").and_then(|s| if s.is_empty() { None } else { Some(s) });

    let userdata = get_field("userdata").and_then(|s| if s.is_empty() { None } else { Some(s) });

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

fn parse_task(line: &str) -> Result<Task, CupError> {
    let fields = parse_csv_line(line);

    if fields.is_empty() {
        return Err(CupError::Parse("Empty task line".to_string()));
    }

    let description = if fields[0].is_empty() {
        None
    } else {
        Some(fields[0].clone())
    };

    let waypoints = fields[1..]
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| TaskPoint::Reference(s.clone()))
        .collect();

    Ok(Task {
        description,
        waypoints,
        options: None,
        observation_zones: Vec::new(),
        multiple_starts: Vec::new(),
    })
}
