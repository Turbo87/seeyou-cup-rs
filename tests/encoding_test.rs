use claims::{assert_ok, assert_some, assert_some_eq};
use seeyou_cup::{CupEncoding, CupFile};
use std::path::Path;

const FIXTURES: [(&str, CupEncoding); 4] = [
    ("2018_schwarzwald_landefelder.cup", CupEncoding::Utf8),
    ("2018_Hotzenwaldwettbewerb_V3.cup", CupEncoding::Windows1252),
    (
        "709-km-Dreieck-DMSt-Aachen-Stolberg-TV.cup",
        CupEncoding::Utf8,
    ),
    ("EC25.cup", CupEncoding::Utf8),
];

#[test]
fn test_encoding_auto_detect_utf8() {
    let path = Path::new("tests/fixtures/EC25.cup");
    let cup = assert_ok!(CupFile::from_path(path));

    assert_eq!(cup.waypoints.len(), 221);
    assert_eq!(cup.waypoints[0].name, "001 Aachen AB Kreuz");
}

#[test]
fn test_encoding_auto_detect_windows1252() {
    let path = Path::new("tests/fixtures/2018_Hotzenwaldwettbewerb_V3.cup");
    let cup = assert_ok!(CupFile::from_path(path));

    assert_eq!(cup.waypoints.len(), 252);
    assert_eq!(cup.waypoints[0].name, "000_Huetten Hotz");

    // Find "121_La Tourne" waypoint
    let la_tourne = assert_some!(cup.waypoints.iter().find(|w| w.name == "121_La Tourne"));

    // The description should contain "Passhöhe" with proper umlaut
    assert_some_eq!(&la_tourne.desc, "Passhöhe");
}

#[test]
fn test_explicit_utf8() {
    let path = Path::new("tests/fixtures/EC25.cup");
    let cup = assert_ok!(CupFile::from_path_with_encoding(path, CupEncoding::Utf8));

    assert_eq!(cup.waypoints.len(), 221);
    assert_eq!(cup.waypoints[0].name, "001 Aachen AB Kreuz");
}

#[test]
fn test_explicit_windows1252() {
    let path = Path::new("tests/fixtures/2018_Hotzenwaldwettbewerb_V3.cup");
    let cup = CupFile::from_path_with_encoding(path, CupEncoding::Windows1252);
    let cup = assert_ok!(cup);

    assert_eq!(cup.waypoints.len(), 252);
    assert_eq!(cup.waypoints[0].name, "000_Huetten Hotz");
}

#[test]
fn test_all_fixtures_parse() {
    let fixtures_path = Path::new("tests/fixtures");
    for (fixture, encoding) in &FIXTURES {
        let path = fixtures_path.join(fixture);
        let cup = assert_ok!(CupFile::from_path_with_encoding(path, *encoding));
        assert!(!cup.waypoints.is_empty(), "No waypoints in {}", fixture);
    }
}

#[test]
fn test_all_fixtures_parse_auto_detect() {
    let fixtures_path = Path::new("tests/fixtures");
    for (fixture, _) in &FIXTURES {
        let path = fixtures_path.join(fixture);
        let cup = assert_ok!(CupFile::from_path(path));
        assert!(!cup.waypoints.is_empty(), "No waypoints in {}", fixture);
    }
}
