use crate::error::ParseIssue;
use crate::parser::column_map::ColumnMap;
use crate::parser::waypoint;
use crate::{Error, ObsZoneStyle, ObservationZone, Task, TaskOptions, Warning, Waypoint};
use csv::StringRecord;

pub fn parse_tasks(
    csv_iter: &mut csv::StringRecordsIter<&[u8]>,
    column_map: &ColumnMap,
    warnings: &mut Vec<Warning>,
) -> Result<Vec<Task>, Error> {
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
                    parse_inline_waypoint_line_with_index(record, column_map, warnings)?;
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

fn parse_task_line(record: &StringRecord) -> Result<Task, Error> {
    if record.is_empty() {
        return Err(ParseIssue::new("Empty task line").into());
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

fn parse_options_line(record: &StringRecord) -> Result<TaskOptions, Error> {
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
                "NearDis" => options.near_dis = Some(value.parse().map_err(ParseIssue::new)?),
                "NearAlt" => options.near_alt = Some(value.parse().map_err(ParseIssue::new)?),
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

fn parse_obszone_line(record: &StringRecord) -> Result<ObservationZone, Error> {
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
                "R1" => r1 = Some(value.parse().map_err(ParseIssue::new)?),
                "A1" => a1 = value.parse().ok(),
                "R2" => r2 = Some(value.parse().map_err(ParseIssue::new)?),
                "A2" => a2 = value.parse().ok(),
                "A12" => a12 = value.parse().ok(),
                "Line" => line_val = Some(value == "1" || value.eq_ignore_ascii_case("true")),
                _ => {}
            }
        }
    }

    let index = index.ok_or_else(|| ParseIssue::new("Missing ObsZone index"))?;
    let style = style.ok_or_else(|| ParseIssue::new("Missing ObsZone style"))?;

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

fn parse_starts_line(record: &StringRecord) -> Result<Vec<String>, Error> {
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
    warnings: &mut Vec<Warning>,
) -> Result<(usize, Waypoint), Error> {
    // Format: Point=1,"Point_3",PNT_3,,4627.136N,01412.856E,0.0m,1,,,,,,,

    // Extract the point index
    let point_idx_str = record[0].trim_start_matches("Point=");
    let point_index = point_idx_str
        .parse::<usize>()
        .map_err(|_| ParseIssue::new(format!("Invalid point index: '{point_idx_str}'")))?;

    // Skip the Point=N field and create a proper waypoint record
    let waypoint_record = StringRecord::from(record.iter().skip(1).collect::<Vec<_>>());

    // Parse as a normal waypoint using the same headers as the waypoint section
    let waypoint = waypoint::parse_waypoint(column_map, &waypoint_record, warnings)
        .map_err(|error| ParseIssue::new(error).with_record(&waypoint_record))?;

    Ok((point_index, waypoint))
}
