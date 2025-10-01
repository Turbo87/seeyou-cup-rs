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

    if !(0.0..60.0).contains(&minutes) {
        return Err(format!(
            "Latitude minutes out of range: {minutes} (must be between 0 and 60)",
        ));
    }

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

    if !(0.0..60.0).contains(&minutes) {
        return Err(format!(
            "Longitude minutes out of range: {minutes} (must be between 0 and 60)",
        ));
    }

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
    use claims::assert_err;
    use proptest::proptest;

    #[test]
    fn test_latitude() {
        let cases = [
            ("5147.809N", 51.7968166),
            ("5147.809S", -51.7968166),
            ("0000.000N", 0.0),
            ("0000.000S", 0.0),
            ("9000.000N", 90.0),
            ("9000.000S", -90.0),
        ];

        for (input, expected) in cases {
            assert!((parse_latitude(input).unwrap() - expected).abs() < 0.0001);
        }
    }

    #[test]
    fn test_latitude_proptest() {
        proptest!(|(s in "\\PC*")| { let _ = parse_latitude(&s); });
    }

    #[test]
    fn test_latitude_errors() {
        insta::assert_snapshot!(assert_err!(parse_latitude("123N")), @"Invalid latitude format: 123N (expected 9 characters, got 4)");
        insta::assert_snapshot!(assert_err!(parse_latitude("123456789N")), @"Invalid latitude format: 123456789N (expected 9 characters, got 10)");
        insta::assert_snapshot!(assert_err!(parse_latitude("5147.809X")), @"Invalid latitude hemisphere: X");
        insta::assert_snapshot!(assert_err!(parse_latitude("5147.809E")), @"Invalid latitude hemisphere: E");
        insta::assert_snapshot!(assert_err!(parse_latitude("XX47.809N")), @"Invalid latitude degrees: XX47.809N");
        insta::assert_snapshot!(assert_err!(parse_latitude("5147.XXXN")), @"Invalid latitude minutes: 5147.XXXN");
        insta::assert_snapshot!(assert_err!(parse_latitude("5160.000N")), @"Latitude minutes out of range: 60 (must be between 0 and 60)");
        insta::assert_snapshot!(assert_err!(parse_latitude("51123456N")), @"Latitude minutes out of range: 123456 (must be between 0 and 60)");
        insta::assert_snapshot!(assert_err!(parse_latitude("9100.000N")), @"Latitude out of range: 91 (must be between -90 and 90)");
        insta::assert_snapshot!(assert_err!(parse_latitude("5147.809Ñ")), @"Invalid latitude format: 5147.809Ñ (contains non-ASCII characters)");
    }

    #[test]
    fn test_longitude() {
        let cases = [
            ("01410.467E", 14.1744500),
            ("00405.003W", -4.0833833),
            ("00000.000E", 0.0),
            ("00000.000W", 0.0),
            ("18000.000E", 180.0),
            ("18000.000W", -180.0),
        ];

        for (input, expected) in cases {
            assert!((parse_longitude(input).unwrap() - expected).abs() < 0.0001);
        }
    }

    #[test]
    fn test_longitude_proptest() {
        proptest!(|(s in "\\PC*")| { let _ = parse_longitude(&s); });
    }

    #[test]
    fn test_longitude_errors() {
        insta::assert_snapshot!(assert_err!(parse_longitude("123E")), @"Invalid longitude format: 123E (expected 10 characters, got 4)");
        insta::assert_snapshot!(assert_err!(parse_longitude("12345678901E")), @"Invalid longitude format: 12345678901E (expected 10 characters, got 12)");
        insta::assert_snapshot!(assert_err!(parse_longitude("01410.467X")), @"Invalid longitude hemisphere: X");
        insta::assert_snapshot!(assert_err!(parse_longitude("01410.467N")), @"Invalid longitude hemisphere: N");
        insta::assert_snapshot!(assert_err!(parse_longitude("XXX10.467E")), @"Invalid longitude degrees: XXX10.467E");
        insta::assert_snapshot!(assert_err!(parse_longitude("01410.XXXE")), @"Invalid longitude minutes: 01410.XXXE");
        insta::assert_snapshot!(assert_err!(parse_longitude("01460.000E")), @"Longitude minutes out of range: 60 (must be between 0 and 60)");
        insta::assert_snapshot!(assert_err!(parse_longitude("014123456E")), @"Longitude minutes out of range: 123456 (must be between 0 and 60)");
        insta::assert_snapshot!(assert_err!(parse_longitude("18100.000E")), @"Longitude out of range: 181 (must be between -180 and 180)");
        insta::assert_snapshot!(assert_err!(parse_longitude("01410.467É")), @"Invalid longitude format: 01410.467É (contains non-ASCII characters)");
    }
}
