use crate::writer::basics::{format_latitude, format_longitude};
use crate::{CupError, Waypoint};
use csv::Writer;

pub fn write_waypoint<W: std::io::Write>(
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
