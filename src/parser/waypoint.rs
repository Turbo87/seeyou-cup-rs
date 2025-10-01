use crate::error::ParseIssue;
use crate::parser::TASK_SEPARATOR;
use crate::parser::basics::{parse_latitude, parse_longitude};
use crate::parser::column_map::ColumnMap;
use crate::{CupError, Waypoint, WaypointStyle};
use csv::StringRecord;

pub fn parse_waypoints(
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

        let waypoint = parse_waypoint(column_map, &record)
            .map_err(|error| ParseIssue::new(error).with_record(&record))?;

        waypoints.push(waypoint);
    }

    Ok(waypoints)
}

pub fn parse_waypoint(column_map: &ColumnMap, record: &StringRecord) -> Result<Waypoint, String> {
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
    let style = parse_waypoint_style(style_str);

    let runway_direction = column_map.rwdir.and_then(|idx| record.get(idx));
    let runway_direction = runway_direction.filter(|s| !s.is_empty());
    let runway_direction = runway_direction.map(parse_runway_direction).transpose()?;

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
    let pictures = pictures.map(parse_pictures).unwrap_or_default();

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

fn parse_waypoint_style(s: &str) -> WaypointStyle {
    match s {
        "1" => WaypointStyle::Waypoint,
        "2" => WaypointStyle::GrassAirfield,
        "3" => WaypointStyle::Outlanding,
        "4" => WaypointStyle::GlidingAirfield,
        "5" => WaypointStyle::SolidAirfield,
        "6" => WaypointStyle::MountainPass,
        "7" => WaypointStyle::MountainTop,
        "8" => WaypointStyle::TransmitterMast,
        "9" => WaypointStyle::Vor,
        "10" => WaypointStyle::Ndb,
        "11" => WaypointStyle::CoolingTower,
        "12" => WaypointStyle::Dam,
        "13" => WaypointStyle::Tunnel,
        "14" => WaypointStyle::Bridge,
        "15" => WaypointStyle::PowerPlant,
        "16" => WaypointStyle::Castle,
        "17" => WaypointStyle::Intersection,
        "18" => WaypointStyle::Marker,
        "19" => WaypointStyle::ControlPoint,
        "20" => WaypointStyle::PgTakeOff,
        "21" => WaypointStyle::PgLandingZone,
        _ => WaypointStyle::Unknown,
    }
}

fn parse_runway_direction(s: &str) -> Result<u16, String> {
    s.parse()
        .map_err(|_| format!("Invalid runway direction: {s}"))
}

fn parse_pictures(s: &str) -> Vec<String> {
    s.split(';')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}
