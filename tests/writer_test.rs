use claims::{assert_ok, assert_some_eq};
use insta::assert_snapshot;
use seeyou_cup::{
    CupEncoding, CupFile, Distance, Elevation, ObsZoneStyle, ObservationZone, RunwayDimension,
    Task, TaskOptions, Waypoint, WaypointStyle,
};
use std::io::Cursor;
use std::str::FromStr;

#[test]
fn test_write_empty_cup_file() {
    let cup_file = CupFile::default();
    let output = assert_ok!(cup_file.to_string());
    assert_snapshot!(output);
}

#[test]
fn test_write_basic_waypoint() {
    let mut cup_file = CupFile::default();
    cup_file.waypoints.push(Waypoint {
        name: "Test Airport".to_string(),
        code: "TEST".to_string(),
        country: "US".to_string(),
        lat: 40.0,
        lon: -74.0,
        elev: Elevation::Meters(100.0),
        style: WaypointStyle::SolidAirfield,
        runway_dir: Some(90),
        runway_len: Some(RunwayDimension::Meters(1500.0)),
        runway_width: Some(RunwayDimension::Meters(30.0)),
        freq: "123.45".to_string(),
        desc: "Test description".to_string(),
        userdata: "user data".to_string(),
        pics: vec!["pic1.jpg".to_string(), "pic2.jpg".to_string()],
    });

    let output = assert_ok!(cup_file.to_string());
    assert_snapshot!(output);
}

#[test]
fn test_write_csv_escaping() {
    let mut cup_file = CupFile::default();

    cup_file.waypoints.push(Waypoint {
        name: "Airport, \"Special\" Name".to_string(),
        code: "A,B\"C".to_string(),
        country: "XX".to_string(),
        lat: 0.0,
        lon: 0.0,
        elev: Elevation::Meters(0.0),
        style: WaypointStyle::Unknown,
        runway_dir: None,
        runway_len: None,
        runway_width: None,
        freq: String::new(),
        desc: "Description with, comma and \"quotes\"".to_string(),
        userdata: String::new(),
        pics: vec![],
    });

    let output = assert_ok!(cup_file.to_string());
    assert_snapshot!(output);
}

#[test]
fn test_write_multiline_fields() {
    let mut cup_file = CupFile::default();

    cup_file.waypoints.push(Waypoint {
        name: "Multi\nLine\nName".to_string(),
        code: "MLN".to_string(),
        country: "XX".to_string(),
        lat: 0.0,
        lon: 0.0,
        elev: Elevation::Meters(0.0),
        style: WaypointStyle::Unknown,
        runway_dir: None,
        runway_len: None,
        runway_width: None,
        freq: String::new(),
        desc: "Line 1\nLine 2\nLine 3".to_string(),
        userdata: String::new(),
        pics: vec![],
    });

    let output = assert_ok!(cup_file.to_string());
    assert_snapshot!(output);
}

#[test]
fn test_coordinate_boundary_values() {
    let mut cup_file = CupFile::default();

    let test_cases = vec![
        ("north_pole", 90.0, 0.0),
        ("south_pole", -90.0, 0.0),
        ("prime_meridian", 0.0, 0.0),
        ("antimeridian", 0.0, 180.0),
        ("west_antimeridian", 0.0, -180.0),
        ("precision_test", 45.123456, -120.987654),
    ];

    for (name, lat, lon) in test_cases {
        cup_file.waypoints.clear();
        cup_file.waypoints.push(Waypoint {
            name: name.to_string(),
            code: "TST".to_string(),
            country: "XX".to_string(),
            lat,
            lon,
            elev: Elevation::Meters(0.0),
            style: WaypointStyle::Unknown,
            runway_dir: None,
            runway_len: None,
            runway_width: None,
            freq: String::new(),
            desc: String::new(),
            userdata: String::new(),
            pics: vec![],
        });

        let output = assert_ok!(cup_file.to_string());
        assert_snapshot!(format!("coordinate_boundary_values_{}", name), output);
    }
}

#[test]
fn test_all_waypoint_styles() {
    let styles = vec![
        WaypointStyle::Unknown,
        WaypointStyle::Waypoint,
        WaypointStyle::GrassAirfield,
        WaypointStyle::Outlanding,
        WaypointStyle::GlidingAirfield,
        WaypointStyle::SolidAirfield,
        WaypointStyle::MountainPass,
        WaypointStyle::MountainTop,
        WaypointStyle::TransmitterMast,
        WaypointStyle::Vor,
        WaypointStyle::Ndb,
        WaypointStyle::CoolingTower,
        WaypointStyle::Dam,
        WaypointStyle::Tunnel,
        WaypointStyle::Bridge,
        WaypointStyle::PowerPlant,
        WaypointStyle::Castle,
        WaypointStyle::Intersection,
        WaypointStyle::Marker,
        WaypointStyle::ControlPoint,
        WaypointStyle::PgTakeOff,
        WaypointStyle::PgLandingZone,
    ];

    let mut cup_file = CupFile::default();
    for style in styles {
        cup_file.waypoints.push(Waypoint {
            name: format!("Style_{:?}", style),
            code: "STY".to_string(),
            country: "XX".to_string(),
            lat: 45.0,
            lon: 10.0,
            elev: Elevation::Meters(500.0),
            style,
            runway_dir: None,
            runway_len: None,
            runway_width: None,
            freq: String::new(),
            desc: String::new(),
            userdata: String::new(),
            pics: vec![],
        });
    }

    let output = assert_ok!(cup_file.to_string());
    assert_snapshot!(output);
}

#[test]
fn test_task_basic() {
    let mut cup_file = CupFile::default();

    cup_file.waypoints.push(Waypoint {
        name: "Start".to_string(),
        code: "S".to_string(),
        country: "XX".to_string(),
        lat: 45.0,
        lon: 10.0,
        elev: Elevation::Meters(500.0),
        style: WaypointStyle::GrassAirfield,
        runway_dir: None,
        runway_len: None,
        runway_width: None,
        freq: String::new(),
        desc: String::new(),
        userdata: String::new(),
        pics: vec![],
    });

    cup_file.waypoints.push(Waypoint {
        name: "Finish".to_string(),
        code: "F".to_string(),
        country: "XX".to_string(),
        lat: 46.0,
        lon: 11.0,
        elev: Elevation::Meters(600.0),
        style: WaypointStyle::SolidAirfield,
        runway_dir: None,
        runway_len: None,
        runway_width: None,
        freq: String::new(),
        desc: String::new(),
        userdata: String::new(),
        pics: vec![],
    });

    cup_file.tasks.push(Task {
        description: Some("Test Task".to_string()),
        waypoint_names: vec!["Start".to_string(), "Finish".to_string()],
        options: None,
        observation_zones: vec![],
        points: vec![],
        multiple_starts: vec![],
    });

    let output = assert_ok!(cup_file.to_string());
    assert_snapshot!(output);
}

#[test]
fn test_task_with_all_features() {
    let mut cup_file = CupFile::default();

    cup_file.waypoints.push(Waypoint {
        name: "Start".to_string(),
        code: "S".to_string(),
        country: "XX".to_string(),
        lat: 45.0,
        lon: 10.0,
        elev: Elevation::Meters(500.0),
        style: WaypointStyle::Unknown,
        runway_dir: None,
        runway_len: None,
        runway_width: None,
        freq: String::new(),
        desc: String::new(),
        userdata: String::new(),
        pics: vec![],
    });

    let inline_waypoint = Waypoint {
        name: "Inline TP".to_string(),
        code: "ITP".to_string(),
        country: "XX".to_string(),
        lat: 46.0,
        lon: 11.0,
        elev: Elevation::Meters(600.0),
        style: WaypointStyle::Waypoint,
        runway_dir: None,
        runway_len: None,
        runway_width: None,
        freq: String::new(),
        desc: "Inline waypoint".to_string(),
        userdata: String::new(),
        pics: vec!["inline.jpg".to_string()],
    };

    cup_file.tasks.push(Task {
        description: Some("Complex Task".to_string()),
        waypoint_names: vec!["Start".to_string()],
        options: Some(TaskOptions {
            no_start: Some("08:30:00".to_string()),
            task_time: Some("05:00:00".to_string()),
            wp_dis: Some(true),
            near_dis: Some(Distance::Kilometers(1.5)),
            near_alt: Some(Elevation::Meters(300.0)),
            min_dis: Some(false),
            random_order: Some(true),
            max_pts: Some(10),
            before_pts: Some(2),
            after_pts: Some(3),
            bonus: Some(50.5),
        }),
        observation_zones: vec![ObservationZone {
            index: 0,
            style: ObsZoneStyle::Fixed,
            r1: Some(RunwayDimension::Meters(500.0)),
            a1: Some(90.0),
            r2: Some(RunwayDimension::Meters(1000.0)),
            a2: Some(45.0),
            a12: Some(123.4),
            line: Some(true),
        }],
        points: vec![(1, inline_waypoint)],
        multiple_starts: vec![
            "Start1".to_string(),
            "Start2".to_string(),
            "Start3".to_string(),
        ],
    });

    let output = assert_ok!(cup_file.to_string());
    assert_snapshot!(output);
}

#[test]
fn test_multiple_tasks() {
    let mut cup_file = CupFile::default();

    // Add waypoints used by tasks
    cup_file.waypoints.push(Waypoint {
        name: "Start A".to_string(),
        code: "SA".to_string(),
        country: "XX".to_string(),
        lat: 45.0,
        lon: 10.0,
        elev: Elevation::Meters(500.0),
        style: WaypointStyle::GrassAirfield,
        runway_dir: None,
        runway_len: None,
        runway_width: None,
        freq: String::new(),
        desc: String::new(),
        userdata: String::new(),
        pics: vec![],
    });

    cup_file.waypoints.push(Waypoint {
        name: "Turn Point".to_string(),
        code: "TP".to_string(),
        country: "XX".to_string(),
        lat: 46.0,
        lon: 11.0,
        elev: Elevation::Meters(600.0),
        style: WaypointStyle::Waypoint,
        runway_dir: None,
        runway_len: None,
        runway_width: None,
        freq: String::new(),
        desc: String::new(),
        userdata: String::new(),
        pics: vec![],
    });

    cup_file.waypoints.push(Waypoint {
        name: "Finish B".to_string(),
        code: "FB".to_string(),
        country: "XX".to_string(),
        lat: 47.0,
        lon: 12.0,
        elev: Elevation::Meters(700.0),
        style: WaypointStyle::SolidAirfield,
        runway_dir: None,
        runway_len: None,
        runway_width: None,
        freq: String::new(),
        desc: String::new(),
        userdata: String::new(),
        pics: vec![],
    });

    // First task - simple triangle
    cup_file.tasks.push(Task {
        description: Some("Triangle Task".to_string()),
        waypoint_names: vec![
            "Start A".to_string(),
            "Turn Point".to_string(),
            "Start A".to_string(),
        ],
        options: Some(TaskOptions {
            no_start: Some("09:00:00".to_string()),
            task_time: Some("03:00:00".to_string()),
            wp_dis: Some(true),
            near_dis: None,
            near_alt: None,
            min_dis: Some(false),
            random_order: None,
            max_pts: None,
            before_pts: None,
            after_pts: None,
            bonus: None,
        }),
        observation_zones: vec![ObservationZone {
            index: 0,
            style: ObsZoneStyle::Fixed,
            r1: Some(RunwayDimension::Meters(1000.0)),
            a1: Some(180.0),
            r2: None,
            a2: None,
            a12: None,
            line: Some(false),
        }],
        points: vec![],
        multiple_starts: vec![],
    });

    // Second task - out and return with options
    cup_file.tasks.push(Task {
        description: Some("Out and Return".to_string()),
        waypoint_names: vec![
            "Start A".to_string(),
            "Finish B".to_string(),
            "Start A".to_string(),
        ],
        options: Some(TaskOptions {
            no_start: None,
            task_time: Some("04:30:00".to_string()),
            wp_dis: Some(false),
            near_dis: Some(Distance::Kilometers(2.0)),
            near_alt: Some(Elevation::Meters(200.0)),
            min_dis: Some(true),
            random_order: Some(false),
            max_pts: Some(5),
            before_pts: Some(1),
            after_pts: Some(1),
            bonus: Some(25.0),
        }),
        observation_zones: vec![],
        points: vec![],
        multiple_starts: vec!["Start A".to_string(), "Turn Point".to_string()],
    });

    // Third task - minimal task with inline waypoint
    let inline_waypoint = Waypoint {
        name: "Inline Goal".to_string(),
        code: "IG".to_string(),
        country: "XX".to_string(),
        lat: 48.0,
        lon: 13.0,
        elev: Elevation::Meters(800.0),
        style: WaypointStyle::Outlanding,
        runway_dir: Some(270),
        runway_len: Some(RunwayDimension::Meters(800.0)),
        runway_width: Some(RunwayDimension::Meters(20.0)),
        freq: "122.5".to_string(),
        desc: "Emergency landing field".to_string(),
        userdata: "Private field".to_string(),
        pics: vec!["field1.jpg".to_string()],
    };

    cup_file.tasks.push(Task {
        description: None, // Test task without description
        waypoint_names: vec!["Start A".to_string()],
        options: None,
        observation_zones: vec![ObservationZone {
            index: 1,
            style: ObsZoneStyle::Symmetrical,
            r1: Some(RunwayDimension::Meters(500.0)),
            a1: None,
            r2: Some(RunwayDimension::Meters(2000.0)),
            a2: Some(30.0),
            a12: Some(45.0),
            line: Some(true),
        }],
        points: vec![(2, inline_waypoint)],
        multiple_starts: vec![],
    });

    let output = assert_ok!(cup_file.to_string());
    assert_snapshot!(output);

    // Verify round-trip works with multiple tasks
    let parsed = assert_ok!(CupFile::from_str(&output));
    assert_eq!(parsed.waypoints.len(), 3);
    assert_eq!(parsed.tasks.len(), 3);

    // Verify first task
    let task1 = &parsed.tasks[0];
    assert_some_eq!(&task1.description, "Triangle Task");
    assert_eq!(task1.waypoint_names.len(), 3);
    assert!(task1.options.is_some());
    assert_eq!(task1.observation_zones.len(), 1);

    // Verify second task
    let task2 = &parsed.tasks[1];
    assert_some_eq!(&task2.description, "Out and Return");
    assert_eq!(task2.multiple_starts.len(), 2);

    // Verify third task
    let task3 = &parsed.tasks[2];
    assert!(task3.description.is_none());
    assert_eq!(task3.points.len(), 1);
    assert_eq!(task3.observation_zones.len(), 1);
}

#[test]
fn test_encoding_windows1252_roundtrip() {
    let mut cup_file = CupFile::default();
    cup_file.waypoints.push(Waypoint {
        name: "Zürich".to_string(),
        code: "ZUR".to_string(),
        country: "CH".to_string(),
        lat: 47.3769,
        lon: 8.5417,
        elev: Elevation::Meters(408.0),
        style: WaypointStyle::Unknown,
        runway_dir: None,
        runway_len: None,
        runway_width: None,
        freq: String::new(),
        desc: "Passhöhe".to_string(),
        userdata: String::new(),
        pics: vec![],
    });

    let mut buffer = Vec::new();
    assert_ok!(cup_file.to_writer_with_encoding(&mut buffer, CupEncoding::Windows1252));

    // Parse it back to verify it worked
    let cursor = Cursor::new(buffer);
    let parsed = assert_ok!(CupFile::from_reader_with_encoding(
        cursor,
        CupEncoding::Windows1252
    ));
    assert_eq!(parsed.waypoints.len(), 1);
    assert_eq!(parsed.waypoints[0].name, "Zürich");
    assert_eq!(&parsed.waypoints[0].desc, "Passhöhe");
}

#[test]
fn test_comprehensive_roundtrip() {
    let mut cup_file = CupFile::default();

    // Add complex waypoint with all fields
    cup_file.waypoints.push(Waypoint {
        name: "Complex Airport, \"Test\"".to_string(),
        code: "CMPLX".to_string(),
        country: "US".to_string(),
        lat: 40.123456,
        lon: -74.987654,
        elev: Elevation::Feet(1250.5),
        style: WaypointStyle::SolidAirfield,
        runway_dir: Some(275),
        runway_len: Some(RunwayDimension::NauticalMiles(1.2)),
        runway_width: Some(RunwayDimension::Meters(45.0)),
        freq: "118.975".to_string(),
        desc: "Multi-line\ndescription with \"quotes\"".to_string(),
        userdata: "User data, with commas".to_string(),
        pics: vec!["pic1.jpg".to_string(), "pic2.png".to_string()],
    });

    let output = assert_ok!(cup_file.to_string());
    assert_snapshot!(output);

    // Verify round-trip works
    let parsed = assert_ok!(CupFile::from_str(&output));
    assert_eq!(parsed.waypoints.len(), 1);

    let wp = &parsed.waypoints[0];
    assert_eq!(wp.name, "Complex Airport, \"Test\"");
    assert_eq!(wp.code, "CMPLX");
    assert!((wp.lat - 40.123456).abs() < 0.001);
    assert!((wp.lon - (-74.987654)).abs() < 0.001);
}
