use claims::{assert_err, assert_ok, assert_some};
use seeyou_cup::CupFile;
use std::str::FromStr;

#[test]
fn test_empty_file() {
    let input = "";
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Empty file");
}

#[test]
fn test_header_only_no_waypoints() {
    let input = "name,code,country,lat,lon,elev,style\n";
    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints.len(), 0);
    assert_eq!(cup.tasks.len(), 0);
}

#[test]
fn test_missing_required_field_latitude() {
    let input = r#"name,code,country,lon,elev,style
"Test",T,XX,00405.003W,500m,1
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Missing required column: lat");
}

#[test]
fn test_missing_required_field_longitude() {
    let input = r#"name,code,country,lat,elev,style
"Test",T,XX,5147.809N,500m,1
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Missing required column: lon");
}

#[test]
fn test_missing_required_field_elevation() {
    let input = r#"name,code,country,lat,lon,style
"Test",T,XX,5147.809N,00405.003W,1
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Missing required column: elev");
}

#[test]
fn test_missing_required_field_style() {
    let input = r#"name,code,country,lat,lon,elev
"Test",T,XX,5147.809N,00405.003W,500m
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Missing required column: style");
}

#[test]
fn test_malformed_csv_unclosed_quotes() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test,T,XX,5147.809N,00405.003W,500m,1
"#;
    let err = assert_err!(CupFile::from_str(input));
    // CSV library error - exact message may vary
    assert!(format!("{}", err).contains("CSV error") || format!("{}", err).contains("Parse error"));
}

#[test]
fn test_truncated_file_incomplete_row() {
    let input = r#"name,code,country,lat,lon,elev,style
"Test",T,XX,5147.809N,00405.003W
"#;
    let err = assert_err!(CupFile::from_str(input));
    insta::assert_snapshot!(err, @"Parse error: Missing 'elev' field");
}

#[test]
fn test_crlf_line_endings() {
    let input =
        "name,code,country,lat,lon,elev,style\r\n\"Test\",T,XX,5147.809N,00405.003W,500m,1\r\n";
    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints.len(), 1);
    assert_eq!(cup.waypoints[0].name, "Test");
}

#[test]
fn test_leading_trailing_whitespace_in_field_values() {
    let input = r#"name,code,country,lat,lon,elev,style
"  Test  ","  T  ","  XX  ",5147.809N,00405.003W,500m,1
"#;
    let cup = assert_ok!(CupFile::from_str(input));
    // CSV parser should preserve whitespace within quoted fields
    assert_eq!(cup.waypoints[0].name, "  Test  ");
    assert_eq!(cup.waypoints[0].code, "  T  ");
    assert_eq!(cup.waypoints[0].country, "  XX  ");
}

#[test]
fn test_tab_characters_in_csv() {
    // Using tabs as separators (should still work with CSV parser)
    let input = "name\tcode\tcountry\tlat\tlon\telev\tstyle\n\"Test\"\t\"T\"\t\"XX\"\t5147.809N\t00405.003W\t500m\t1\n";
    let err = assert_err!(CupFile::from_str(input));
    // CSV parser expects commas by default, so this should fail
    assert!(format!("{}", err).contains("Parse error") || format!("{}", err).contains("Missing"));
}

#[test]
fn test_unicode_characters_beyond_ascii() {
    let input = r#"name,code,country,lat,lon,elev,style,desc
"Zürich ✈️","ZUR","CH",4723.033N,00829.967E,408m,5,"Airport with émojis and ümlaunts"
"#;
    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints.len(), 1);
    assert_eq!(cup.waypoints[0].name, "Zürich ✈️");
    assert_eq!(
        &cup.waypoints[0].description,
        "Airport with émojis and ümlaunts"
    );
}

#[test]
fn test_file_with_only_task_section_should_fail() {
    let input = r#"-----Related Tasks-----
"Task 1","Waypoint1","Waypoint2"
"#;
    // This should fail because waypoints are referenced but not defined
    let err = assert_err!(CupFile::from_str(input));
    // The exact error depends on how the parser handles missing waypoint section
    assert!(format!("{}", err).contains("Parse error") || format!("{}", err).contains("Missing"));
}

#[test]
fn test_task_with_inline_waypoint_definition() {
    let input = r#"name,code,country,lat,lon,elev,style
"Start","S","XX",5147.809N,00405.003W,500m,2
-----Related Tasks-----
"Task with inline","Start"
Point=1,"Inline TP","T1","XX",5148.000N,00406.000W,600m,1
"#;
    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints.len(), 1);
    assert_eq!(cup.tasks.len(), 1);

    // Check that inline waypoint is in the points field
    assert_eq!(cup.tasks[0].points.len(), 1);
    assert_eq!(cup.tasks[0].points[0].0, 1); // Point index
    assert_eq!(cup.tasks[0].points[0].1.name, "Inline TP");
}

#[test]
fn test_mixed_inline_and_referenced_waypoints() {
    let input = r#"name,code,country,lat,lon,elev,style
"Start","S","XX",5147.809N,00405.003W,500m,2
"Finish","F","XX",5149.000N,00407.000W,700m,2
-----Related Tasks-----
"Mixed Task","Start","Finish"
Point=1,"Inline TP","T1","XX",5148.000N,00406.000W,600m,1
"#;
    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints.len(), 2);
    assert_eq!(cup.tasks.len(), 1);

    // Task references both defined waypoints and has an inline waypoint
    assert_eq!(cup.tasks[0].waypoint_names, vec!["Start", "Finish"]);
    assert_eq!(cup.tasks[0].points.len(), 1);
    assert_eq!(cup.tasks[0].points[0].1.name, "Inline TP");
}

#[test]
fn test_preserve_all_task_fields_roundtrip() {
    let input = r#"name,code,country,lat,lon,elev,style
"Start","S","XX",5147.809N,00405.003W,500m,2
"TP1","T1","XX",5148.000N,00406.000W,600m,1
"Finish","F","XX",5149.000N,00407.000W,700m,2
-----Related Tasks-----
"Complete Task","Start","TP1","Finish"
Options,NoStart=12:00:00,TaskTime=03:00:00,WpDis=True,NearDis=0.5km,NearAlt=300m,MinDis=True,RandomOrder=False,MaxPts=1000,BeforePts=300,AfterPts=200,Bonus=100
ObsZone=0,Style=0,R1=3000m,A1=180,R2=1000m,A2=90,A12=45,Line=True
ObsZone=1,Style=1,R1=500m,A1=0,R2=0m,A2=0,A12=0,Line=False
STARTS="Start","Start2"
"#;
    let cup = assert_ok!(CupFile::from_str(input));

    // Write back to string
    let output = assert_ok!(cup.to_string());

    // Parse the output again
    let cup2 = assert_ok!(CupFile::from_str(&output));

    // Verify task preservation
    assert_eq!(cup2.tasks.len(), 1);
    let task = &cup2.tasks[0];

    // Check that task options are preserved (basic check)
    assert_some!(&task.options);

    // Check that observation zones are preserved
    assert_eq!(task.observation_zones.len(), 2);

    // Check that multiple starts are preserved
    assert!(!task.multiple_starts.is_empty());
}
