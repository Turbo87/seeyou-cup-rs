use claims::{assert_none, assert_ok, assert_some_eq};
use seeyou::{CupFile, WaypointStyle};

#[test]
fn test_arbitrary_column_order() {
    let input = r#"lat,lon,elev,name,code,country,style,rwdir,rwlen,rwwidth,freq,desc
5147.809N,00405.003W,525ft,"Cross Hands","CSS",UK,1,,,,"Turn Point"
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints.len(), 1);
    assert_eq!(cup.waypoints[0].name, "Cross Hands");
    assert_eq!(cup.waypoints[0].code, "CSS");
    assert_eq!(cup.waypoints[0].country, "UK");
    assert_eq!(cup.waypoints[0].style, WaypointStyle::Waypoint);
}

#[test]
fn test_missing_optional_columns_after_style() {
    let input = r#"name,code,country,lat,lon,elev,style
"Cross Hands","CSS",UK,5147.809N,00405.003W,525ft,1
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints.len(), 1);
    assert_eq!(cup.waypoints[0].name, "Cross Hands");
    assert_eq!(cup.waypoints[0].runway_dir, None);
    assert_eq!(cup.waypoints[0].runway_len, None);
    assert_eq!(cup.waypoints[0].freq, None);
    assert_eq!(cup.waypoints[0].desc, None);
}

#[test]
fn test_header_with_subset_of_fields() {
    let input = r#"name,lat,lon,elev,style
"Waypoint1",5147.809N,00405.003W,500m,1
"Waypoint2",5148.000N,00406.000W,600m,1
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints.len(), 2);
    assert_eq!(cup.waypoints[0].name, "Waypoint1");
    assert_eq!(cup.waypoints[0].code, ""); // Empty string default
    assert_eq!(cup.waypoints[0].country, ""); // Empty string default
}

#[test]
fn test_escaped_quotes_within_quoted_fields() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc
"Test ""Quote""","T",XX,5147.809N,00405.003W,0m,1,,,,,"Description with ""quotes"""
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints[0].name, r#"Test "Quote""#);
    assert_some_eq!(&cup.waypoints[0].desc, r#"Description with "quotes""#);
}

#[test]
fn test_multiline_fields_with_newlines() {
    let input = "name,code,country,lat,lon,elev,style,desc
\"Test\",T,XX,5147.809N,00405.003W,0m,1,\"Line 1
Line 2
Line 3\"
";

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints[0].name, "Test");
    assert_some_eq!(&cup.waypoints[0].desc, "Line 1\nLine 2\nLine 3");
}

#[test]
fn test_case_insensitive_header_names() {
    let input = r#"Name,Code,Country,Lat,Lon,Elev,Style
"Test","T",XX,5147.809N,00405.003W,500m,1
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints.len(), 1);
    assert_eq!(cup.waypoints[0].name, "Test");
}

#[test]
fn test_fields_with_commas_are_quoted() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc
"Test","T",XX,5147.809N,00405.003W,0m,1,,,,,"Description with, comma"
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_some_eq!(&cup.waypoints[0].desc, "Description with, comma");
}

#[test]
fn test_empty_optional_fields() {
    let input = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Test",,,5147.809N,00405.003W,0m,1,,,,,,,
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints[0].name, "Test");
    assert_eq!(cup.waypoints[0].code, "");
    assert_eq!(cup.waypoints[0].country, "");
    assert_none!(cup.waypoints[0].runway_dir);
    assert_none!(&cup.waypoints[0].freq);
    assert_none!(&cup.waypoints[0].desc);
    assert_none!(&cup.waypoints[0].userdata);
    assert!(cup.waypoints[0].pics.is_empty());
}

#[test]
fn test_waypoints_only_file() {
    let input = r#"name,code,country,lat,lon,elev,style
"Waypoint1","W1",XX,5147.809N,00405.003W,500m,1
"Waypoint2","W2",XX,5148.000N,00406.000W,600m,1
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints.len(), 2);
    assert_eq!(cup.tasks.len(), 0);
}

#[test]
fn test_file_with_waypoints_and_tasks() {
    let input = r#"name,code,country,lat,lon,elev,style
"Start","S",XX,5147.809N,00405.003W,500m,2
"TP1","T1",XX,5148.000N,00406.000W,600m,1
"Finish","F",XX,5149.000N,00407.000W,700m,2
-----Related Tasks-----
"Task 1","Start","TP1","Finish"
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints.len(), 3);
    assert_eq!(cup.tasks.len(), 1);
    assert_some_eq!(&cup.tasks[0].description, "Task 1");
}

#[test]
fn test_related_tasks_separator() {
    let input = r#"name,code,country,lat,lon,elev,style
"Waypoint","W",XX,5147.809N,00405.003W,500m,1
-----Related Tasks-----
,"Waypoint","Waypoint"
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints.len(), 1);
    assert_eq!(cup.tasks.len(), 1);
    assert_eq!(cup.tasks[0].description, None);
}

#[test]
fn test_arbitrary_column_order_with_all_fields() {
    let input = r#"desc,style,elev,lon,lat,country,code,name,freq,rwdir,rwlen,rwwidth
"Airport desc",5,504.0m,01410.467E,4621.379N,SI,"LJBL","Lesce",123.500,144,1130.0m,30m
"#;

    let cup = assert_ok!(CupFile::from_str(input));
    assert_eq!(cup.waypoints[0].name, "Lesce");
    assert_eq!(cup.waypoints[0].code, "LJBL");
    assert_eq!(cup.waypoints[0].country, "SI");
    assert_eq!(cup.waypoints[0].style, WaypointStyle::SolidAirfield);
    assert_some_eq!(&cup.waypoints[0].desc, "Airport desc");
}
