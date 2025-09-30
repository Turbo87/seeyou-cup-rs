use claims::{assert_ok, assert_some_eq};
use seeyou_cup::{CupEncoding, CupFile};
use std::path::Path;

#[test]
fn test_parse_task() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Lesce","LJBL",SI,4621.379N,01410.467E,504.0m,5,144,1130.0m,,123.500,"Home Airfield"
"Waypoint1","WP1",SI,4622.000N,01411.000E,600m,1,,,,,"Test waypoint"
-----Related Tasks-----
"Test Task","LJBL","WP1","LJBL"
"#;

    let cup = assert_ok!(CupFile::from_str(&input));

    assert_eq!(cup.waypoints.len(), 2);
    assert_eq!(cup.tasks.len(), 1);

    let task = &cup.tasks[0];
    assert_some_eq!(&task.description, "Test Task");
    assert_eq!(task.waypoint_names.len(), 3);
}

#[test]
fn test_roundtrip() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Lesce","LJBL",SI,4621.379N,01410.467E,504.0m,5,144,1130.0m,,123.500,"Home Airfield"
"#;

    let cup = assert_ok!(CupFile::from_str(&input));
    let output = assert_ok!(cup.to_string());
    let cup2 = assert_ok!(CupFile::from_str(&output));

    assert_eq!(cup.waypoints.len(), cup2.waypoints.len());
    assert_eq!(cup.waypoints[0].name, cup2.waypoints[0].name);
    assert_eq!(cup.waypoints[0].code, cup2.waypoints[0].code);
}

#[test]
fn test_fixture_schwarzwald() {
    let path = Path::new("tests/fixtures/2018_schwarzwald_landefelder.cup");
    let cup = assert_ok!(CupFile::from_path(path));

    assert_eq!(cup.waypoints.len(), 64);
    assert_eq!(cup.tasks.len(), 0);

    assert_eq!(cup.waypoints[0].name, "LF_Aichelberg");
    assert_eq!(cup.waypoints[0].code, "NL A07");
    assert_eq!(cup.waypoints[0].country, "de");
}

#[test]
fn test_fixture_hotzenwaldwettbewerb() {
    let path = Path::new("tests/fixtures/2018_Hotzenwaldwettbewerb_V3.cup");
    let cup = CupFile::from_path_with_encoding(path, CupEncoding::Windows1252);
    let cup = assert_ok!(cup);

    assert_eq!(cup.waypoints.len(), 252);
    assert_eq!(cup.tasks.len(), 0);

    assert_eq!(cup.waypoints[0].name, "000_Huetten Hotz");
    assert_eq!(cup.waypoints[0].code, "0");
}

#[test]
fn test_fixture_ec25() {
    let path = Path::new("tests/fixtures/EC25.cup");
    let cup = assert_ok!(CupFile::from_path(path));

    assert_eq!(cup.waypoints.len(), 221);
    assert_eq!(cup.tasks.len(), 0);

    assert_eq!(cup.waypoints[0].name, "001 Aachen AB Kreuz");
    assert_eq!(cup.waypoints[0].code, "001ACKRE");
}

#[test]
fn test_fixture_with_task() {
    let path = Path::new("tests/fixtures/709-km-Dreieck-DMSt-Aachen-Stolberg-TV.cup");
    let cup = assert_ok!(CupFile::from_path(path));

    assert_eq!(cup.waypoints.len(), 4);
    assert_eq!(cup.tasks.len(), 1);

    let task = &cup.tasks[0];
    assert_some_eq!(
        &task.description,
        "709 km · Dreieck · DMSt · Aachen Stolberg TV_282915"
    );
    assert_eq!(task.waypoint_names.len(), 5);
}
#[test]
fn test_fixture_windows1252() {
    let path = Path::new("tests/fixtures/2018_schwarzwald_landefelder.cup");
    let cup = CupFile::from_path_with_encoding(path, CupEncoding::Windows1252);
    let cup = assert_ok!(cup);

    assert!(!cup.waypoints.is_empty());
}
