use seeyou::{CupEncoding, CupFile, Elevation, WaypointStyle};
use std::path::Path;

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
    assert!(matches!(cup.waypoints[0].elev, Elevation::Feet(_)));
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
    assert!(matches!(wp.elev, Elevation::Meters(_)));
    assert_eq!(wp.style, WaypointStyle::SolidAirfield);
    assert_eq!(wp.runway_dir, Some(144));
    assert!(wp.runway_len.is_some());
    assert_eq!(wp.freq, Some("123.500".to_string()));
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
fn test_parse_task() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Lesce","LJBL",SI,4621.379N,01410.467E,504.0m,5,144,1130.0m,,123.500,"Home Airfield"
"Waypoint1","WP1",SI,4622.000N,01411.000E,600m,1,,,,,"Test waypoint"
-----Related Tasks-----
"Test Task","LJBL","WP1","LJBL"
"#;

    let cup = CupFile::from_str(input).unwrap();

    assert_eq!(cup.waypoints.len(), 2);
    assert_eq!(cup.tasks.len(), 1);

    let task = &cup.tasks[0];
    assert_eq!(task.description, Some("Test Task".to_string()));
    assert_eq!(task.waypoints.len(), 3);
}

#[test]
fn test_roundtrip() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Lesce","LJBL",SI,4621.379N,01410.467E,504.0m,5,144,1130.0m,,123.500,"Home Airfield"
"#;

    let cup = CupFile::from_str(input).unwrap();
    let output = cup.to_string().unwrap();
    let cup2 = CupFile::from_str(&output).unwrap();

    assert_eq!(cup.waypoints.len(), cup2.waypoints.len());
    assert_eq!(cup.waypoints[0].name, cup2.waypoints[0].name);
    assert_eq!(cup.waypoints[0].code, cup2.waypoints[0].code);
}

#[test]
fn test_latitude_conversion() {
    assert!((parse_lat("5147.809N") - 51.7968166).abs() < 0.0001);
    assert!((parse_lat("5147.809S") - (-51.7968166)).abs() < 0.0001);
}

#[test]
fn test_longitude_conversion() {
    assert!((parse_lon("01410.467E") - 14.1744500).abs() < 0.0001);
    assert!((parse_lon("00405.003W") - (-4.0833833)).abs() < 0.0001);
}

fn parse_lat(s: &str) -> f64 {
    let input = format!(
        r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Test","T",XX,{},00000.000E,0m,1
"#,
        s
    );
    let cup = CupFile::from_str(&input).unwrap();
    cup.waypoints[0].lat
}

fn parse_lon(s: &str) -> f64 {
    let input = format!(
        r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Test","T",XX,0000.000N,{},0m,1
"#,
        s
    );
    let cup = CupFile::from_str(&input).unwrap();
    cup.waypoints[0].lon
}

#[test]
fn test_fixture_schwarzwald() {
    let path = Path::new("tests/fixtures/2018_schwarzwald_landefelder.cup");
    let cup = CupFile::from_path(path).unwrap();

    assert_eq!(cup.waypoints.len(), 64);
    assert_eq!(cup.tasks.len(), 0);

    assert_eq!(cup.waypoints[0].name, "LF_Aichelberg");
    assert_eq!(cup.waypoints[0].code, "NL A07");
    assert_eq!(cup.waypoints[0].country, "de");
}

#[test]
fn test_fixture_hotzenwaldwettbewerb() {
    let path = Path::new("tests/fixtures/2018_Hotzenwaldwettbewerb_V3.cup");
    let cup = CupFile::from_path_with_encoding(path, CupEncoding::Windows1252).unwrap();

    assert_eq!(cup.waypoints.len(), 252);
    assert_eq!(cup.tasks.len(), 0);

    assert_eq!(cup.waypoints[0].name, "000_Huetten Hotz");
    assert_eq!(cup.waypoints[0].code, "0");
}

#[test]
fn test_fixture_ec25() {
    let path = Path::new("tests/fixtures/EC25.cup");
    let cup = CupFile::from_path(path).unwrap();

    assert_eq!(cup.waypoints.len(), 221);
    assert_eq!(cup.tasks.len(), 0);

    assert_eq!(cup.waypoints[0].name, "001 Aachen AB Kreuz");
    assert_eq!(cup.waypoints[0].code, "001ACKRE");
}

#[test]
fn test_fixture_with_task() {
    let path = Path::new("tests/fixtures/709-km-Dreieck-DMSt-Aachen-Stolberg-TV.cup");
    let cup = CupFile::from_path(path).unwrap();

    assert_eq!(cup.waypoints.len(), 4);
    assert_eq!(cup.tasks.len(), 1);

    let task = &cup.tasks[0];
    assert_eq!(
        task.description.as_deref(),
        Some("709 km · Dreieck · DMSt · Aachen Stolberg TV_282915")
    );
    assert_eq!(task.waypoints.len(), 5);
}
