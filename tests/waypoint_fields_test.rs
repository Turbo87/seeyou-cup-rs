use claims::{assert_err, assert_matches, assert_ok};
use insta::assert_debug_snapshot;
use seeyou_cup::{CupFile, Elevation, RunwayDimension, WaypointStyle};
use std::str::FromStr;

#[test]
fn test_parse_basic_waypoint() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Cross Hands","CSS",UK,5147.809N,00405.003W,525ft,1,,,,"Turn Point, A48/A476, Between Cross Hands and Gorslas, 9 NMl ESE of Camarthen."
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(cup.waypoints.len(), 1);
    assert_debug_snapshot!(cup.waypoints[0], @r#"
    Waypoint {
        name: "Cross Hands",
        code: "CSS",
        country: "UK",
        lat: 51.796816666666665,
        lon: -4.083383333333333,
        elev: Feet(
            525.0,
        ),
        style: Waypoint,
        runway_dir: None,
        runway_len: None,
        runway_width: None,
        freq: "Turn Point, A48/A476, Between Cross Hands and Gorslas, 9 NMl ESE of Camarthen.",
        desc: "",
        userdata: "",
        pics: [],
    }
    "#);
}

#[test]
fn test_parse_airport() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Lesce","LJBL",SI,4621.379N,01410.467E,504.0m,5,144,1130.0m,,123.500,"Home Airfield"
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(cup.waypoints.len(), 1);
    assert_debug_snapshot!(cup.waypoints[0], @r#"
    Waypoint {
        name: "Lesce",
        code: "LJBL",
        country: "SI",
        lat: 46.356316666666665,
        lon: 14.17445,
        elev: Meters(
            504.0,
        ),
        style: SolidAirfield,
        runway_dir: Some(
            144,
        ),
        runway_len: Some(
            Meters(
                1130.0,
            ),
        ),
        runway_width: None,
        freq: "123.500",
        desc: "Home Airfield",
        userdata: "",
        pics: [],
    }
    "#);
}

#[test]
fn test_parse_outlanding() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Aiton","O23L",FR,4533.517N,00614.050E,299.9m,3,110,300.0m,,"Page 222: O23L Large flat area. High crops. Sudden wind changes. Power lines N/S. S of road marked fields"
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(cup.waypoints.len(), 1);
    assert_debug_snapshot!(cup.waypoints[0], @r#"
    Waypoint {
        name: "Aiton",
        code: "O23L",
        country: "FR",
        lat: 45.558616666666666,
        lon: 6.234166666666667,
        elev: Meters(
            299.9,
        ),
        style: Outlanding,
        runway_dir: Some(
            110,
        ),
        runway_len: Some(
            Meters(
                300.0,
            ),
        ),
        runway_width: None,
        freq: "Page 222: O23L Large flat area. High crops. Sudden wind changes. Power lines N/S. S of road marked fields",
        desc: "",
        userdata: "",
        pics: [],
    }
    "#);
}

#[test]
fn test_empty_name_should_error() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"",CSS,UK,5147.809N,00405.003W,525ft,1
"#;

    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Name field cannot be empty");
}

#[test]
fn test_invalid_latitude_too_short() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,5147.8N,00405.003W,0m,1
"#;

    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Invalid latitude format: 5147.8N (expected 9 characters, got 7)");
}

#[test]
fn test_invalid_latitude_too_long() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,51247.809N,00405.003W,0m,1
"#;

    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Invalid latitude format: 51247.809N (expected 9 characters, got 10)");
}

#[test]
fn test_invalid_latitude_hemisphere() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,5147.809X,00405.003W,500m,1
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Invalid latitude hemisphere: X");
}

#[test]
fn test_latitude_out_of_range_positive() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,9100.000N,00405.003W,500m,1
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Latitude out of range: 91 (must be between -90 and 90)");
}

#[test]
fn test_latitude_out_of_range_negative() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,9100.000S,00405.003W,500m,1
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Latitude out of range: -91 (must be between -90 and 90)");
}

#[test]
fn test_invalid_longitude_too_short() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,5147.809N,0405.0W,0m,1
"#;

    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Invalid longitude format: 0405.0W (expected 10 characters, got 7)");
}

#[test]
fn test_invalid_longitude_too_long() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,5147.809N,000405.003W,0m,1
"#;

    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Invalid longitude format: 000405.003W (expected 10 characters, got 11)");
}

#[test]
fn test_invalid_longitude_hemisphere() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,5147.809N,00405.003Y,500m,1
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Invalid longitude hemisphere: Y");
}

#[test]
fn test_longitude_out_of_range_positive() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,5147.809N,18100.000E,500m,1
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Longitude out of range: 181 (must be between -180 and 180)");
}

#[test]
fn test_longitude_out_of_range_negative() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,5147.809N,18100.000W,500m,1
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Longitude out of range: -181 (must be between -180 and 180)");
}

#[test]
fn test_latitude_zero() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,0000.000N,00000.000E,0m,1
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(cup.waypoints[0].lat, 0.0);
}

#[test]
fn test_latitude_90_degrees() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,9000.000N,00000.000E,0m,1
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(cup.waypoints[0].lat, 90.0);
}

#[test]
fn test_latitude_90_degrees_south() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,9000.000S,00000.000E,0m,1
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(cup.waypoints[0].lat, -90.0);
}

#[test]
fn test_longitude_zero() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,0000.000N,00000.000E,0m,1
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(cup.waypoints[0].lon, 0.0);
}

#[test]
fn test_longitude_180_degrees() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,0000.000N,18000.000E,0m,1
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(cup.waypoints[0].lon, 180.0);
}

#[test]
fn test_longitude_180_degrees_west() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,0000.000N,18000.000W,0m,1
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(cup.waypoints[0].lon, -180.0);
}

#[test]
fn test_elevation_no_unit_defaults_to_meters() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,5147.809N,00405.003W,500,1
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_matches!(&cup.waypoints[0].elev, Elevation::Meters(500.0));
}

#[test]
fn test_elevation_decimal_separator_must_be_point() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,5147.809N,00405.003W,504.5m,1
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_matches!(&cup.waypoints[0].elev, Elevation::Meters(v) if (v - 504.5).abs() < 0.01);
}

#[test]
fn test_invalid_numeric_elevation() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,5147.809N,00405.003W,invalid,1
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Invalid elevation unit: invalid");
}

#[test]
fn test_invalid_elevation_unit() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,5147.809N,00405.003W,500km,1
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Invalid elevation: 500km");
}

#[test]
fn test_mixed_elevation_units_in_same_file() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test1",T1,XX,5147.809N,00405.003W,500m,1
"Test2",T2,XX,5147.809N,00405.003W,1640ft,1
"Test3",T3,XX,5147.809N,00405.003W,300,1
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints.len(), 3);
    assert_matches!(&cup.waypoints[0].elev, Elevation::Meters(500.0));
    assert_matches!(&cup.waypoints[1].elev, Elevation::Feet(1640.0));
    assert_matches!(&cup.waypoints[2].elev, Elevation::Meters(300.0));
}

#[test]
fn test_invalid_waypoint_style_defaults_to_unknown() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,5147.809N,00405.003W,0m,99
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints[0].style, WaypointStyle::Unknown);
}

#[test]
fn test_waypoint_style_greater_than_21_defaults_to_unknown() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,5147.809N,00405.003W,0m,25
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints[0].style, WaypointStyle::Unknown);
}

#[test]
fn test_all_valid_waypoint_styles() {
    for style_num in 0..=21 {
        let input = format!(
            r#"name,code,country,lat,lon,elev,style
"Test",T,XX,5147.809N,00405.003W,0m,{}
"#,
            style_num
        );

        let cup = CupFile::from_str(&input).unwrap();
        assert_eq!(cup.waypoints.len(), 1);
        assert_eq!(cup.waypoints[0].style as u8, style_num);
    }
}

#[test]
fn test_runway_direction_format() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq
"Test",LJBL,SI,4621.379N,01410.467E,504.0m,5,144,1130.0m
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(cup.waypoints[0].runway_dir, Some(144));
}

#[test]
fn test_runway_direction_000() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq
"Test",LJBL,SI,4621.379N,01410.467E,504.0m,5,000,1130.0m
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(cup.waypoints[0].runway_dir, Some(0));
}

#[test]
fn test_runway_direction_359() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq
"Test",LJBL,SI,4621.379N,01410.467E,504.0m,5,359,1130.0m
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(cup.waypoints[0].runway_dir, Some(359));
}

#[test]
fn test_invalid_numeric_runway_direction() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir
"Test",T,XX,5147.809N,00405.003W,500m,5,abc
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Invalid runway direction: abc");
}

#[test]
fn test_runway_length_no_unit_defaults_to_meters() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq
"Test",LJBL,SI,4621.379N,01410.467E,504.0m,5,144,1130
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_matches!(
        &cup.waypoints[0].runway_len,
        Some(RunwayDimension::Meters(1130.0))
    );
}

#[test]
fn test_runway_length_nautical_miles() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq
"Test",LJBL,SI,4621.379N,01410.467E,504.0m,5,144,1.5nm
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_matches!(
        &cup.waypoints[0].runway_len,
        Some(RunwayDimension::NauticalMiles(v)) if (v - 1.5).abs() < 0.01
    );
}

#[test]
fn test_runway_length_statute_miles() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq
"Test",LJBL,SI,4621.379N,01410.467E,504.0m,5,144,2.0ml
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_matches!(
        &cup.waypoints[0].runway_len,
        Some(RunwayDimension::StatuteMiles(v)) if (v - 2.0).abs() < 0.01
    );
}

#[test]
fn test_invalid_numeric_runway_length() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen
"Test",T,XX,5147.809N,00405.003W,500m,5,144,invalid
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Invalid runway dimension unit: invalid");
}

#[test]
fn test_invalid_runway_dimension_unit() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen
"Test",T,XX,5147.809N,00405.003W,500m,5,144,1130km
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Invalid runway dimension: 1130km");
}

#[test]
fn test_frequency_format() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq
"Test",LJBL,SI,4621.379N,01410.467E,504.0m,5,144,1130.0m,,123.500
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(&cup.waypoints[0].freq, "123.500");
}

#[test]
fn test_frequency_in_quotes() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq
"Test",LJBL,SI,4621.379N,01410.467E,504.0m,5,144,1130.0m,,"123.500"
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(&cup.waypoints[0].freq, "123.500");
}

#[test]
fn test_description_unlimited_length() {
    let long_desc = "A".repeat(1000);
    let input = format!(
        r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc
"Test",T,XX,5147.809N,00405.003W,0m,1,,,,,"{}"
"#,
        long_desc
    );

    let cup = CupFile::from_str(&input).unwrap();
    assert_eq!(&cup.waypoints[0].desc, &long_desc);
}

#[test]
fn test_pictures_semicolon_separated() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Test",T,XX,5147.809N,00405.003W,0m,1,,,,,,,pic1.jpg;pic2.jpg;pic3.jpg
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(
        cup.waypoints[0].pics,
        vec!["pic1.jpg", "pic2.jpg", "pic3.jpg"]
    );
}

#[test]
fn test_pictures_in_quotes_when_multiple() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Test",T,XX,5147.809N,00405.003W,0m,1,,,,,,,"pic1.jpg;pic2.jpg"
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_eq!(cup.waypoints[0].pics, vec!["pic1.jpg", "pic2.jpg"]);
}
