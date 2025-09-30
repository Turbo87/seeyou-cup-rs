use claims::{assert_err, assert_matches, assert_ok, assert_some, assert_some_eq};
use seeyou::{CupFile, Elevation, RunwayDimension, WaypointStyle};

#[test]
fn test_parse_basic_waypoint() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Cross Hands","CSS",UK,5147.809N,00405.003W,525ft,1,,,,"Turn Point, A48/A476, Between Cross Hands and Gorslas, 9 NMl ESE of Camarthen."
"#;

    let cup = CupFile::from_str(input).unwrap();

    assert_eq!(cup.waypoints.len(), 1);
    assert_eq!(cup.waypoints[0].name, "Cross Hands");
    assert_eq!(cup.waypoints[0].code, "CSS");
    assert_eq!(cup.waypoints[0].country, "UK");
    assert_matches!(&cup.waypoints[0].elev, Elevation::Feet(_));
    assert_eq!(cup.waypoints[0].style, WaypointStyle::Waypoint);
}

#[test]
fn test_parse_airport() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Lesce","LJBL",SI,4621.379N,01410.467E,504.0m,5,144,1130.0m,,123.500,"Home Airfield"
"#;

    let cup = CupFile::from_str(input).unwrap();

    assert_eq!(cup.waypoints.len(), 1);
    let wp = &cup.waypoints[0];
    assert_eq!(wp.name, "Lesce");
    assert_eq!(wp.code, "LJBL");
    assert_eq!(wp.country, "SI");
    assert_matches!(&wp.elev, Elevation::Meters(_));
    assert_eq!(wp.style, WaypointStyle::SolidAirfield);
    assert_some_eq!(wp.runway_dir, 144);
    assert_some!(&wp.runway_len);
    assert_some_eq!(&wp.freq, "123.500");
}

#[test]
fn test_parse_outlanding() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Aiton","O23L",FR,4533.517N,00614.050E,299.9m,3,110,300.0m,,"Page 222: O23L Large flat area. High crops. Sudden wind changes. Power lines N/S. S of road marked fields"
"#;

    let cup = CupFile::from_str(input).unwrap();

    assert_eq!(cup.waypoints.len(), 1);
    let wp = &cup.waypoints[0];
    assert_eq!(wp.name, "Aiton");
    assert_eq!(wp.style, WaypointStyle::Outlanding);
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
fn test_frequency_format() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq
"Test",LJBL,SI,4621.379N,01410.467E,504.0m,5,144,1130.0m,,123.500
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_some_eq!(&cup.waypoints[0].freq, "123.500");
}

#[test]
fn test_frequency_in_quotes() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq
"Test",LJBL,SI,4621.379N,01410.467E,504.0m,5,144,1130.0m,,"123.500"
"#;

    let cup = CupFile::from_str(input).unwrap();
    assert_some_eq!(&cup.waypoints[0].freq, "123.500");
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
    assert_some_eq!(&cup.waypoints[0].desc, &long_desc);
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
