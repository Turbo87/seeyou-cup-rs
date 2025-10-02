use claims::assert_ok;
use insta::assert_snapshot;
use seeyou_cup::CupEncoding::{self, Utf8, Windows1252};
use seeyou_cup::CupFile;
use std::path::{Path, PathBuf};

const FIXTURES: [(&str, CupEncoding); 4] = [
    ("2018_schwarzwald_landefelder.cup", Utf8),
    ("2018_Hotzenwaldwettbewerb_V3.cup", Windows1252),
    ("709-km-Dreieck-DMSt-Aachen-Stolberg-TV.cup", Utf8),
    ("EC25.cup", Utf8),
];

fn hotzenwald() -> PathBuf {
    PathBuf::from("tests/fixtures/2018_Hotzenwaldwettbewerb_V3.cup")
}

fn schwarzwald() -> PathBuf {
    PathBuf::from("tests/fixtures/2018_schwarzwald_landefelder.cup")
}

#[test]
fn test_encoding_auto_detect_utf8() {
    let cup = assert_ok!(CupFile::from_path(schwarzwald()));
    assert_eq!(cup.waypoints.len(), 64);
    assert_snapshot!(cup.waypoints[2].description, @r"Landerichtung 24 wegen Geländeanstieg vorzuziehen.\nAnfang der Wiese ist sumpfig!");
}

#[test]
fn test_encoding_auto_detect_windows1252() {
    let cup = assert_ok!(CupFile::from_path(hotzenwald()));
    assert_eq!(cup.waypoints.len(), 252);
    assert_snapshot!(cup.waypoints[121].description, @"Passhöhe");
}

#[test]
fn test_explicit_utf8() {
    let cup = assert_ok!(CupFile::from_path_with_encoding(schwarzwald(), Utf8));
    assert_eq!(cup.waypoints.len(), 64);
    assert_snapshot!(cup.waypoints[2].description, @r"Landerichtung 24 wegen Geländeanstieg vorzuziehen.\nAnfang der Wiese ist sumpfig!");

    let cup = assert_ok!(CupFile::from_path_with_encoding(hotzenwald(), Utf8));
    assert_eq!(cup.waypoints.len(), 252);
    assert_snapshot!(cup.waypoints[121].description, @"Passh�he");
}

#[test]
fn test_explicit_windows1252() {
    let cup = assert_ok!(CupFile::from_path_with_encoding(schwarzwald(), Windows1252));
    assert_eq!(cup.waypoints.len(), 64);
    assert_snapshot!(cup.waypoints[2].description, @r"Landerichtung 24 wegen GelÃ¤ndeanstieg vorzuziehen.\nAnfang der Wiese ist sumpfig!");

    let cup = assert_ok!(CupFile::from_path_with_encoding(hotzenwald(), Windows1252));
    assert_eq!(cup.waypoints.len(), 252);
    assert_snapshot!(cup.waypoints[121].description, @"Passhöhe");
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
