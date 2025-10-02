pub fn parse_latitude(s: &str) -> Result<f64, String> {
    let bytes = s.as_bytes();
    let bytes_len = bytes.len();

    if bytes_len < 9 {
        return Err(format!(
            "Invalid latitude format: '{s}' (expected 9 characters, got {bytes_len})",
        ));
    }

    let hemisphere = bytes[bytes_len - 1];

    if !bytes[0..4].iter().all(u8::is_ascii_digit)
        || bytes[4] != b'.'
        || !bytes[5..bytes_len - 1].iter().all(u8::is_ascii_digit)
        || (hemisphere != b'N' && hemisphere != b'S')
    {
        return Err(format!(
            "Invalid latitude format: '{s}' (unexpected character)"
        ));
    }

    let degrees: u8 = s[0..2].parse().unwrap();
    let minutes: f64 = s[2..bytes_len - 1].parse().unwrap();
    if !(0.0..60.0).contains(&minutes) {
        return Err(format!(
            "Latitude minutes out of range: '{minutes}' (must be between 0 and 60)",
        ));
    }

    let mut decimal_degrees = degrees as f64 + minutes / 60.0;

    if hemisphere == b'S' {
        decimal_degrees = -decimal_degrees;
    }

    // Validate range
    if !(-90.0..=90.0).contains(&decimal_degrees) {
        return Err(format!(
            "Latitude out of range: '{decimal_degrees}' (must be between -90 and 90)",
        ));
    }

    Ok(decimal_degrees)
}

pub fn parse_longitude(s: &str) -> Result<f64, String> {
    let bytes = s.as_bytes();
    let bytes_len = bytes.len();

    if bytes_len < 10 {
        return Err(format!(
            "Invalid longitude format: '{s}' (expected 10 characters, got {bytes_len})",
        ));
    }

    let hemisphere = bytes[bytes_len - 1];

    if !bytes[0..5].iter().all(u8::is_ascii_digit)
        || bytes[5] != b'.'
        || !bytes[6..bytes_len - 1].iter().all(u8::is_ascii_digit)
        || (hemisphere != b'E' && hemisphere != b'W')
    {
        return Err(format!(
            "Invalid longitude format: '{s}' (unexpected character)"
        ));
    }

    let degrees: u8 = s[0..3].parse().unwrap();
    let minutes: f64 = s[3..bytes_len - 1].parse().unwrap();
    if !(0.0..60.0).contains(&minutes) {
        return Err(format!(
            "Longitude minutes out of range: '{minutes}' (must be between 0 and 60)",
        ));
    }

    let mut decimal_degrees = degrees as f64 + minutes / 60.0;

    if hemisphere == b'W' {
        decimal_degrees = -decimal_degrees;
    }

    // Validate range
    if !(-180.0..=180.0).contains(&decimal_degrees) {
        return Err(format!(
            "Longitude out of range: '{decimal_degrees}' (must be between -180 and 180)",
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
            ("1234.56789N", 12.5761315),
        ];

        for (input, expected) in cases {
            let output = parse_latitude(input).unwrap();
            assert!((output - expected).abs() < 0.0001);
        }
    }

    #[test]
    fn test_latitude_proptest() {
        proptest!(|(s in "\\PC*")| { let _ = parse_latitude(&s); });
    }

    #[test]
    fn test_latitude_errors() {
        insta::assert_snapshot!(assert_err!(parse_latitude("123N")), @"Invalid latitude format: '123N' (expected 9 characters, got 4)");
        insta::assert_snapshot!(assert_err!(parse_latitude("123456789N")), @"Invalid latitude format: '123456789N' (unexpected character)");
        insta::assert_snapshot!(assert_err!(parse_latitude("5147.809X")), @"Invalid latitude format: '5147.809X' (unexpected character)");
        insta::assert_snapshot!(assert_err!(parse_latitude("5147.809E")), @"Invalid latitude format: '5147.809E' (unexpected character)");
        insta::assert_snapshot!(assert_err!(parse_latitude("XX47.809N")), @"Invalid latitude format: 'XX47.809N' (unexpected character)");
        insta::assert_snapshot!(assert_err!(parse_latitude("5147.XXXN")), @"Invalid latitude format: '5147.XXXN' (unexpected character)");
        insta::assert_snapshot!(assert_err!(parse_latitude("5160.000N")), @"Latitude minutes out of range: '60' (must be between 0 and 60)");
        insta::assert_snapshot!(assert_err!(parse_latitude("51123456N")), @"Invalid latitude format: '51123456N' (unexpected character)");
        insta::assert_snapshot!(assert_err!(parse_latitude("9100.000N")), @"Latitude out of range: '91' (must be between -90 and 90)");
        insta::assert_snapshot!(assert_err!(parse_latitude("5147.809Ñ")), @"Invalid latitude format: '5147.809Ñ' (unexpected character)");
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
            ("12345.6789W", -123.761315),
        ];

        for (input, expected) in cases {
            let output = parse_longitude(input).unwrap();
            assert!((output - expected).abs() < 0.0001);
        }
    }

    #[test]
    fn test_longitude_proptest() {
        proptest!(|(s in "\\PC*")| { let _ = parse_longitude(&s); });
    }

    #[test]
    fn test_longitude_errors() {
        insta::assert_snapshot!(assert_err!(parse_longitude("123E")), @"Invalid longitude format: '123E' (expected 10 characters, got 4)");
        insta::assert_snapshot!(assert_err!(parse_longitude("12345678901E")), @"Invalid longitude format: '12345678901E' (unexpected character)");
        insta::assert_snapshot!(assert_err!(parse_longitude("01410.467X")), @"Invalid longitude format: '01410.467X' (unexpected character)");
        insta::assert_snapshot!(assert_err!(parse_longitude("01410.467N")), @"Invalid longitude format: '01410.467N' (unexpected character)");
        insta::assert_snapshot!(assert_err!(parse_longitude("XXX10.467E")), @"Invalid longitude format: 'XXX10.467E' (unexpected character)");
        insta::assert_snapshot!(assert_err!(parse_longitude("01410.XXXE")), @"Invalid longitude format: '01410.XXXE' (unexpected character)");
        insta::assert_snapshot!(assert_err!(parse_longitude("01460.000E")), @"Longitude minutes out of range: '60' (must be between 0 and 60)");
        insta::assert_snapshot!(assert_err!(parse_longitude("014123456E")), @"Invalid longitude format: '014123456E' (unexpected character)");
        insta::assert_snapshot!(assert_err!(parse_longitude("18100.000E")), @"Longitude out of range: '181' (must be between -180 and 180)");
        insta::assert_snapshot!(assert_err!(parse_longitude("01410.467É")), @"Invalid longitude format: '01410.467É' (unexpected character)");
    }
}
