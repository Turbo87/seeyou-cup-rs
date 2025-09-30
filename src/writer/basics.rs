pub fn format_latitude(lat: f64) -> String {
    let hemisphere = if lat >= 0.0 { 'N' } else { 'S' };
    let abs_lat = lat.abs();
    let degrees = abs_lat.floor() as u32;
    let minutes = (abs_lat - degrees as f64) * 60.0;
    format!("{:02}{:06.3}{}", degrees, minutes, hemisphere)
}

pub fn format_longitude(lon: f64) -> String {
    let hemisphere = if lon >= 0.0 { 'E' } else { 'W' };
    let abs_lon = lon.abs();
    let degrees = abs_lon.floor() as u32;
    let minutes = (abs_lon - degrees as f64) * 60.0;
    format!("{:03}{:06.3}{}", degrees, minutes, hemisphere)
}
