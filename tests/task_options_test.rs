use claims::{assert_matches, assert_ok, assert_some, assert_some_eq};
use seeyou::{CupFile, Distance, Elevation, ObsZoneStyle, RunwayDimension, WaypointStyle};

#[test]
fn test_parse_options_line() {
    let input = r#"name,code,country,lat,lon,elev,style
"Start","S",XX,5147.809N,00405.003W,500m,2
"TP1","T1",XX,5148.000N,00406.000W,600m,1
"Finish","F",XX,5149.000N,00407.000W,700m,2
-----Related Tasks-----
"Task 1","Start","TP1","Finish"
Options,NoStart=12:34:56,TaskTime=01:45:12,WpDis=False,NearDis=0.7km,NearAlt=300.0m
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.tasks.len(), 1);

    let options = assert_some!(&cup.tasks[0].options);
    assert_some_eq!(&options.no_start, "12:34:56");
    assert_some_eq!(&options.task_time, "01:45:12");
    assert_some_eq!(options.wp_dis, false);
    assert_some!(&options.near_dis);
    assert_some!(&options.near_alt);
}

#[test]
fn test_nostart_time() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
Options,NoStart=08:30:00
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let options = assert_some!(&cup.tasks[0].options);
    assert_some_eq!(&options.no_start, "08:30:00");
}

#[test]
fn test_tasktime_duration() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
Options,TaskTime=02:30:45
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let options = assert_some!(&cup.tasks[0].options);
    assert_some_eq!(&options.task_time, "02:30:45");
}

#[test]
fn test_wpdis_boolean_true() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
Options,WpDis=True
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let options = assert_some!(&cup.tasks[0].options);
    assert_some_eq!(options.wp_dis, true);
}

#[test]
fn test_wpdis_boolean_false() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
Options,WpDis=False
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let options = assert_some!(&cup.tasks[0].options);
    assert_some_eq!(options.wp_dis, false);
}

#[test]
fn test_neardis_with_km() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
Options,NearDis=1.5km
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let options = assert_some!(&cup.tasks[0].options);
    let near_dis = assert_some!(&options.near_dis);
    assert_matches!(near_dis, Distance::Kilometers(v) if (*v - 1.5).abs() < 0.01);
}

#[test]
fn test_neardis_with_meters() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
Options,NearDis=500m
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let options = assert_some!(&cup.tasks[0].options);
    let near_dis = assert_some!(&options.near_dis);
    assert_matches!(near_dis, Distance::Meters(500.0));
}

#[test]
fn test_nearalt_with_unit() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
Options,NearAlt=300.0m
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let options = assert_some!(&cup.tasks[0].options);
    let near_alt = assert_some!(&options.near_alt);
    assert_matches!(near_alt, Elevation::Meters(v) if (*v - 300.0).abs() < 0.01);
}

#[test]
fn test_mindis_boolean() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
Options,MinDis=True
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let options = assert_some!(&cup.tasks[0].options);
    assert_some_eq!(options.min_dis, true);
}

#[test]
fn test_randomorder_boolean() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
Options,RandomOrder=True
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let options = assert_some!(&cup.tasks[0].options);
    assert_some_eq!(options.random_order, true);
}

#[test]
fn test_maxpts_number() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
Options,MaxPts=10
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let options = assert_some!(&cup.tasks[0].options);
    assert_some_eq!(options.max_pts, 10);
}

#[test]
fn test_beforepts_number() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
Options,BeforePts=2
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let options = assert_some!(&cup.tasks[0].options);
    assert_some_eq!(options.before_pts, 2);
}

#[test]
fn test_afterpts_number() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
Options,AfterPts=1
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let options = assert_some!(&cup.tasks[0].options);
    assert_some_eq!(options.after_pts, 1);
}

#[test]
fn test_bonus_number() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
Options,Bonus=50.5
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let options = assert_some!(&cup.tasks[0].options);
    let bonus = assert_some!(options.bonus);
    assert!((bonus - 50.5).abs() < 0.01);
}

#[test]
fn test_parse_obszone_line() {
    let input = r#"name,code,country,lat,lon,elev,style
"Start","S",XX,5147.809N,00405.003W,500m,2
"TP1","T1",XX,5148.000N,00406.000W,600m,1
-----Related Tasks-----
,"Start","TP1"
ObsZone=0,Style=2,R1=400m,A1=180,Line=1
ObsZone=1,Style=0,R1=35000m,A1=30,R2=12000m,A2=12,A12=123.4
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.tasks.len(), 1);
    assert_eq!(cup.tasks[0].observation_zones.len(), 2);

    let oz0 = &cup.tasks[0].observation_zones[0];
    assert_eq!(oz0.index, 0);
    assert_eq!(oz0.style, ObsZoneStyle::ToNextPoint);
    assert_matches!(&oz0.r1, Some(RunwayDimension::Meters(400.0)));
    assert_some_eq!(oz0.a1, 180.0);
    assert_some_eq!(oz0.line, true);
}

#[test]
fn test_obszone_index() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
ObsZone=0,Style=0,R1=1000m
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let oz = &cup.tasks[0].observation_zones[0];
    assert_eq!(oz.index, 0);
}

#[test]
fn test_obszone_all_styles() {
    for style in 0..=4 {
        let input = format!(
            r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
ObsZone=0,Style={},R1=1000m
"#,
            style
        );

        let cup = assert_ok!(CupFile::from_str(&input));
        let oz = &cup.tasks[0].observation_zones[0];
        assert_eq!(oz.style as u8, style);
    }
}

#[test]
fn test_obszone_r1_radius() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
ObsZone=0,Style=0,R1=500m
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let oz = &cup.tasks[0].observation_zones[0];
    assert_matches!(&oz.r1, Some(RunwayDimension::Meters(500.0)));
}

#[test]
fn test_obszone_a1_angle() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
ObsZone=0,Style=0,R1=500m,A1=90
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let oz = &cup.tasks[0].observation_zones[0];
    assert_some_eq!(oz.a1, 90.0);
}

#[test]
fn test_obszone_r2_radius() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
ObsZone=0,Style=0,R1=500m,R2=1000m
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let oz = &cup.tasks[0].observation_zones[0];
    assert_matches!(&oz.r2, Some(RunwayDimension::Meters(1000.0)));
}

#[test]
fn test_obszone_a2_angle() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
ObsZone=0,Style=0,R1=500m,A2=45
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let oz = &cup.tasks[0].observation_zones[0];
    assert_some_eq!(oz.a2, 45.0);
}

#[test]
fn test_obszone_a12_angle() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
ObsZone=0,Style=0,R1=500m,A12=123.4
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let oz = &cup.tasks[0].observation_zones[0];
    let a12 = assert_some!(oz.a12);
    assert!((a12 - 123.4).abs() < 0.01);
}

#[test]
fn test_obszone_line_boolean() {
    let input = r#"name,code,country,lat,lon,elev,style
"WP","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"WP"
ObsZone=0,Style=0,R1=500m,Line=1
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let oz = &cup.tasks[0].observation_zones[0];
    assert_some_eq!(oz.line, true);
}

#[test]
fn test_parse_starts_line() {
    let input = r#"name,code,country,lat,lon,elev,style
"Celovec","C",XX,5147.809N,00405.003W,500m,1
"Hodos","H",XX,5148.000N,00406.000W,600m,1
"Ratitovec","R",XX,5149.000N,00407.000W,700m,1
"Jamnik","J",XX,5150.000N,00408.000W,800m,1
-----Related Tasks-----
,"Celovec"
STARTS=Celovec,Hodos,Ratitovec,Jamnik
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.tasks.len(), 1);
    assert_eq!(cup.tasks[0].multiple_starts.len(), 4);
    assert_eq!(cup.tasks[0].multiple_starts[0], "Celovec");
    assert_eq!(cup.tasks[0].multiple_starts[1], "Hodos");
    assert_eq!(cup.tasks[0].multiple_starts[2], "Ratitovec");
    assert_eq!(cup.tasks[0].multiple_starts[3], "Jamnik");
}

#[test]
fn test_multiple_starts_waypoints_defined() {
    let input = r#"name,code,country,lat,lon,elev,style
"Start1","S1",XX,5147.809N,00405.003W,500m,2
"Start2","S2",XX,5148.000N,00406.000W,600m,2
"TP","T",XX,5149.000N,00407.000W,700m,1
-----Related Tasks-----
,"Start1","TP"
STARTS=Start1,Start2
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.tasks[0].multiple_starts.len(), 2);
    assert_eq!(cup.tasks[0].multiple_starts[0], "Start1");
    assert_eq!(cup.tasks[0].multiple_starts[1], "Start2");
}

#[test]
fn test_inline_waypoint_basic() {
    let input = r#"name,code,country,lat,lon,elev,style
"Start","S",XX,5147.809N,00405.003W,500m,2
"Finish","F",XX,5149.000N,00407.000W,700m,2
-----Related Tasks-----
"Test Task","Start","Finish"
Point=1,"TP1",T1,XX,5148.000N,00406.000W,600m,1
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.tasks.len(), 1);
    assert_eq!(cup.tasks[0].waypoint_names.len(), 2); // "Start" and "Finish"
    assert_eq!(cup.tasks[0].points.len(), 1); // One inline waypoint

    // Check that the inline waypoint was parsed correctly
    let (point_index, waypoint) = &cup.tasks[0].points[0];
    assert_eq!(*point_index, 1);
    assert_eq!(waypoint.name, "TP1");
    assert_eq!(waypoint.code, "T1");
    assert_eq!(waypoint.country, "XX");
    assert_eq!(waypoint.style, WaypointStyle::Waypoint);
}

#[test]
fn test_inline_waypoint_with_all_fields() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Start","S",XX,5147.809N,00405.003W,500m,2,,,,,,
-----Related Tasks-----
"Test","Start"
Point=1,"Airport",AIRP,SI,4621.379N,01410.467E,504.0m,5,144,1130.0m,30m,123.500,"Test Airport","User data","pic1.jpg;pic2.jpg"
"#;

    let cup = assert_ok!(CupFile::from_str(input));

    // Check that we have one inline waypoint
    assert_eq!(cup.tasks[0].points.len(), 1);
    let (point_index, waypoint) = &cup.tasks[0].points[0];
    assert_eq!(*point_index, 1);

    assert_eq!(waypoint.name, "Airport");
    assert_eq!(waypoint.code, "AIRP");
    assert_eq!(waypoint.country, "SI");
    assert_eq!(waypoint.style, WaypointStyle::SolidAirfield);
    assert_some_eq!(waypoint.runway_dir, 144);
    assert_some!(&waypoint.runway_len);
    assert_some!(&waypoint.runway_width);
    assert_some_eq!(&waypoint.freq, "123.500");
    assert_some_eq!(&waypoint.desc, "Test Airport");
    assert_some_eq!(&waypoint.userdata, "User data");
    assert_eq!(waypoint.pics, vec!["pic1.jpg", "pic2.jpg"]);
}

#[test]
fn test_mixed_inline_and_reference_waypoints() {
    let input = r#"name,code,country,lat,lon,elev,style
"Start","S",XX,5147.809N,00405.003W,500m,2
"Finish","F",XX,5149.000N,00407.000W,700m,2
-----Related Tasks-----
"Mixed Task","Start","TP1","TP2","Finish"
Point=1,"TP1",T1,XX,5148.000N,00406.000W,600m,1
Point=2,"TP2",T2,XX,5148.500N,00406.500W,650m,1
"#;

    let cup = assert_ok!(CupFile::from_str(input));

    // Check waypoint names (references)
    assert_eq!(cup.tasks[0].waypoint_names.len(), 4);
    assert_eq!(cup.tasks[0].waypoint_names[0], "Start");
    assert_eq!(cup.tasks[0].waypoint_names[1], "TP1");
    assert_eq!(cup.tasks[0].waypoint_names[2], "TP2");
    assert_eq!(cup.tasks[0].waypoint_names[3], "Finish");

    // Check inline waypoints
    assert_eq!(cup.tasks[0].points.len(), 2);

    // Check first inline waypoint (Point=1)
    let (idx1, waypoint1) = &cup.tasks[0].points[0];
    assert_eq!(*idx1, 1);
    assert_eq!(waypoint1.name, "TP1");
    assert_eq!(waypoint1.code, "T1");

    // Check second inline waypoint (Point=3)
    let (idx2, waypoint2) = &cup.tasks[0].points[1];
    assert_eq!(*idx2, 2);
    assert_eq!(waypoint2.name, "TP2");
    assert_eq!(waypoint2.code, "T2");
}

#[test]
fn test_inline_waypoint_roundtrip() {
    let input = r#"name,code,country,lat,lon,elev,style
"Start","S",XX,5147.809N,00405.003W,500m,2
"Finish","F",XX,5149.000N,00407.000W,700m,2
-----Related Tasks-----
"Test Task","Start","Finish"
Point=1,"TP1",T1,XX,5148.000N,00406.000W,600m,1
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    let output = assert_ok!(cup.to_string());
    let cup2 = assert_ok!(CupFile::from_str(&output));

    // Check that the roundtrip preserved the inline waypoint
    assert_eq!(cup.tasks[0].points.len(), cup2.tasks[0].points.len());
    assert_eq!(
        cup.tasks[0].waypoint_names.len(),
        cup2.tasks[0].waypoint_names.len()
    );

    // Check that the inline waypoint was preserved
    let (idx1, wp1) = &cup.tasks[0].points[0];
    let (idx2, wp2) = &cup2.tasks[0].points[0];

    assert_eq!(*idx1, *idx2);
    assert_eq!(wp1.name, wp2.name);
    assert_eq!(wp1.code, wp2.code);
    assert_eq!(wp1.country, wp2.country);
    assert_eq!(wp1.style, wp2.style);
    assert!((wp1.lat - wp2.lat).abs() < 0.0001);
    assert!((wp1.lon - wp2.lon).abs() < 0.0001);
}
