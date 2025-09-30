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

    let (waypoints, column_map) = parse_waypoints(waypoint_section)?;
    let tasks = parse_tasks(task_section, &column_map)?;

    Ok(CupFile { waypoints, tasks })
}

fn split_sections(content: &str) -> (&str, Option<&str>) {
    if let Some(pos) = content.find("-----Related Tasks-----") {
        (&content[..pos], Some(&content[pos..]))
    } else {
        (content, None)
    }
}

struct ColumnMap {
    name: usize,
    code: usize,
    country: usize,
    lat: usize,
    lon: usize,
    elev: usize,
    style: usize,
    rwdir: Option<usize>,
    rwlen: Option<usize>,
    rwwidth: Option<usize>,
    freq: Option<usize>,
    desc: Option<usize>,
    userdata: Option<usize>,
    pics: Option<usize>,
}

fn build_column_map(headers: &StringRecord) -> Result<ColumnMap, String> {
    let mut name = None;
    let mut code = None;
    let mut country = None;
    let mut lat = None;
    let mut lon = None;
    let mut elev = None;
    let mut style = None;
    let mut rwdir = None;
    let mut rwlen = None;
    let mut rwwidth = None;
    let mut freq = None;
    let mut desc = None;
    let mut userdata = None;
    let mut pics = None;

    for (idx, header) in headers.iter().enumerate() {
        match header.to_lowercase().as_str() {
            "name" => name = Some(idx),
            "code" => code = Some(idx),
            "country" => country = Some(idx),
            "lat" => lat = Some(idx),
            "lon" => lon = Some(idx),
            "elev" => elev = Some(idx),
            "style" => style = Some(idx),
            "rwdir" => rwdir = Some(idx),
            "rwlen" => rwlen = Some(idx),
            "rwwidth" => rwwidth = Some(idx),
            "freq" => freq = Some(idx),
            "desc" => desc = Some(idx),
            "userdata" => userdata = Some(idx),
            "pics" => pics = Some(idx),
            _ => {}
        }
    }

    Ok(ColumnMap {
        name: name.ok_or("Missing required column: name")?,
        code: code.ok_or("Missing required column: code")?,
        country: country.ok_or("Missing required column: country")?,
        lat: lat.ok_or("Missing required column: lat")?,
        lon: lon.ok_or("Missing required column: lon")?,
        elev: elev.ok_or("Missing required column: elev")?,
        style: style.ok_or("Missing required column: style")?,
        rwdir,
        rwlen,
        rwwidth,
        freq,
        desc,
        userdata,
        pics,
    })
}

fn parse_waypoints(section: &str) -> Result<(Vec<Waypoint>, ColumnMap), CupError> {
    if section.trim().is_empty() {
        return Err(CupError::Parse("Empty file".to_string()));
    }

    let mut csv_reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(section.as_bytes());

    let headers = csv_reader.headers()?.clone();
    let column_map = build_column_map(&headers).map_err(CupError::Parse)?;

    let mut waypoints = Vec::new();
    for result in csv_reader.records() {
        let record = result?;
        let waypoint = parse_waypoint(&column_map, &record).map_err(CupError::Parse)?;
        waypoints.push(waypoint);
    }

    Ok((waypoints, column_map))
}

fn parse_tasks(section: Option<&str>, column_map: &ColumnMap) -> Result<Vec<Task>, CupError> {
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

        if trimmed.starts_with("Options")
            || trimmed.starts_with("ObsZone=")
            || trimmed.starts_with("Point=")
            || trimmed.starts_with("STARTS=")
        {
            continue;
        }

        let mut task = parse_task_line(line)?;

        // Look ahead for Options, ObsZone, Point, and STARTS lines
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
            } else if next_trimmed.starts_with("Point=") {
                let (point_index, inline_waypoint) =
                    parse_inline_waypoint_line_with_index(next_line, column_map)?;
                // Add the inline waypoint to the points field
                task.points.push((point_index as u32, inline_waypoint));
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

fn parse_waypoint(column_map: &ColumnMap, record: &StringRecord) -> Result<Waypoint, String> {
    let name = record.get(column_map.name).ok_or("Missing 'name' field")?;

    if name.is_empty() {
        return Err("Name field cannot be empty".into());
    }

    let name = name.to_string();

    let code = record.get(column_map.code).unwrap_or_default().to_string();
    let country = record
        .get(column_map.country)
        .unwrap_or_default()
        .to_string();

    let lat_str = record.get(column_map.lat).ok_or("Missing 'lat' field")?;
    let lat = parse_latitude(lat_str)?;

    let lon_str = record.get(column_map.lon).ok_or("Missing 'lon' field")?;
    let lon = parse_longitude(lon_str)?;

    let elev_str = record.get(column_map.elev).ok_or("Missing 'elev' field")?;
    let elev = elev_str.parse()?;

    let style_str = record
        .get(column_map.style)
        .ok_or("Missing 'style' field")?;
    let style = parse_waypoint_style(style_str)?;

    let runway_dir = column_map
        .rwdir
        .and_then(|idx| record.get(idx))
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<u16>()
                .map_err(|_| format!("Invalid runway direction: {s}"))
        })
        .transpose()?;

    let runway_len = column_map
        .rwlen
        .and_then(|idx| record.get(idx))
        .filter(|s| !s.is_empty())
        .map(|s| s.parse())
        .transpose()?;

    let runway_width = column_map
        .rwwidth
        .and_then(|idx| record.get(idx))
        .filter(|s| !s.is_empty())
        .map(|s| s.parse())
        .transpose()?;

    let freq = column_map
        .freq
        .and_then(|idx| record.get(idx))
        .unwrap_or_default()
        .to_string();

    let desc = column_map
        .desc
        .and_then(|idx| record.get(idx))
        .unwrap_or_default()
        .to_string();

    let userdata = column_map
        .userdata
        .and_then(|idx| record.get(idx))
        .unwrap_or_default()
        .to_string();

    let pics = column_map
        .pics
        .and_then(|idx| record.get(idx))
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
                "NearAlt" => {
                    options.near_alt = Some(value.parse().map_err(CupError::Parse)?)
                }
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

    if let Some(value_str) = s.strip_suffix("km") {
        let value: f64 = value_str
            .parse()
            .map_err(|_| CupError::Parse(format!("Invalid distance value: {}", s)))?;
        Ok(Distance::Kilometers(value))
    } else if let Some(value_str) = s.strip_suffix("nm") {
        let value: f64 = value_str
            .parse()
            .map_err(|_| CupError::Parse(format!("Invalid distance value: {}", s)))?;
        Ok(Distance::NauticalMiles(value))
    } else if let Some(value_str) = s.strip_suffix("ml") {
        let value: f64 = value_str
            .parse()
            .map_err(|_| CupError::Parse(format!("Invalid distance value: {}", s)))?;
        Ok(Distance::StatuteMiles(value))
    } else if let Some(value_str) = s.strip_suffix('m') {
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
                "R1" => r1 = Some(parse_distance(value)?),
                "A1" => a1 = value.parse().ok(),
                "R2" => r2 = Some(parse_distance(value)?),
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

fn parse_inline_waypoint_line_with_index(
    line: &str,
    column_map: &ColumnMap,
) -> Result<(usize, Waypoint), CupError> {
    // Format: Point=1,"Point_3",PNT_3,,4627.136N,01412.856E,0.0m,1,,,,,,,

    // Create CSV reader to handle quoting and escaping properly
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(line.as_bytes());

    let record = match csv_reader.records().next() {
        Some(Ok(r)) => r,
        Some(Err(e)) => {
            return Err(CupError::Parse(format!(
                "Failed to parse inline waypoint line: {}",
                e
            )));
        }
        None => return Err(CupError::Parse("Empty inline waypoint line".to_string())),
    };

    // Check that it starts with Point=N
    if record.is_empty() || !record[0].starts_with("Point=") {
        return Err(CupError::Parse(format!(
            "Invalid inline waypoint format: {}",
            line
        )));
    }

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
