pub fn parse_latitude(s: &str) -> Result<f64, String> {
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

pub fn parse_longitude(s: &str) -> Result<f64, String> {
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
