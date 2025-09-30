# seeyou-cup

A Rust library for parsing and writing [SeeYou CUP files](docs/SeeYou_CUP_file_format.pdf), commonly used in aviation and gliding for waypoint and task data.

## Overview

The SeeYou CUP format is widely used by flight planning software and GPS devices in the aviation community to store waypoint information, tasks, and related flight data. This library provides a safe, efficient way to read, write, and manipulate CUP files in Rust.

## Features

- **Parse CUP files** from strings, files, or any `Read` implementation
- **Write CUP files** to strings, files, or any `Write` implementation  
- **Multiple encoding support** (UTF-8 and Windows-1252)
- **Full waypoint support** including coordinates, elevations, runway information, and descriptions
- **Task parsing** with observation zones and task options

## Quick Start

### Reading a CUP file

```rust
use seeyou_cup::CupFile;

// From a file path
let cup_file = CupFile::from_path("waypoints.cup")?;

// From a string
let cup_content = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Home Field","HOME",US,4015.500N,07355.250W,150.0m,5,90,800.0m,,123.500,"Home airfield"
"#;
let cup_file = CupFile::from_str(cup_content)?;

// Access waypoints
for waypoint in &cup_file.waypoints {
    println!("Waypoint: {} ({})", waypoint.name, waypoint.code);
    println!("  Location: {:.3}°N, {:.3}°W", waypoint.lat, waypoint.lon);
    println!("  Elevation: {:.0}m", waypoint.elev.to_meters());
}

// Access tasks
for task in &cup_file.tasks {
    if let Some(description) = &task.description {
        println!("Task: {}", description);
    }
    println!("  Waypoints: {:?}", task.waypoint_names);
}
```

### Writing a CUP file

```rust
use seeyou_cup::{CupFile, Waypoint, Elevation, WaypointStyle};

let mut cup_file = CupFile::default();

// Create a waypoint
let waypoint = Waypoint {
    name: "Test Field".to_string(),
    code: "TEST".to_string(),
    country: "US".to_string(),
    lat: 40.25833,  // 40°15.500'N
    lon: -73.92083, // 73°55.250'W  
    elev: Elevation::Meters(150.0),
    style: WaypointStyle::SolidAirfield,
    runway_dir: Some(90),
    runway_len: Some(seeyou_cup::RunwayDimension::Meters(800.0)),
    runway_width: None,
    freq: Some("123.500".to_string()),
    desc: Some("Test airfield".to_string()),
    userdata: None,
    pics: Vec::new(),
};

cup_file.waypoints.push(waypoint);

// Write to file
cup_file.to_path("output.cup")?;

// Or get as string
let cup_string = cup_file.to_string()?;
```

### Working with different encodings

```rust
use seeyou_cup::{CupFile, CupEncoding};

// Read with specific encoding
let cup_file = CupFile::from_path_with_encoding("waypoints.cup", CupEncoding::Windows1252)?;

// Write with specific encoding
cup_file.to_path_with_encoding("output.cup", CupEncoding::Utf8)?;
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
