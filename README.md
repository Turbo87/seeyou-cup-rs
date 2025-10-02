# seeyou-cup

A Rust library for parsing and writing [SeeYou CUP files](https://downloads.naviter.com/docs/SeeYou_CUP_file_format.pdf), commonly used in aviation and gliding for waypoint and task data.

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

```rust,no_run
use seeyou_cup::CupFile;
use std::str::FromStr;

// From a file path
let (cup_file, warnings) = CupFile::from_path("waypoints.cup").unwrap();

// From a string
let cup_content = r#"name,code,country,lat,lon,elev,style,rwdir,rwlen,rwwidth,freq,desc,userdata,pics
"Home Field","HOME",US,4015.500N,07355.250W,150.0m,5,90,800.0m,,123.500,"Home airfield"
"#;
let (cup_file, warnings) = CupFile::from_str(cup_content).unwrap();

// Access waypoints
for waypoint in &cup_file.waypoints {
    println!("Waypoint: {} ({})", waypoint.name, waypoint.code);
    println!("  Location: {:.3}°N, {:.3}°W", waypoint.latitude, waypoint.longitude);
    println!("  Elevation: {:.0}m", waypoint.elevation.to_meters());
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

```rust,no_run
use seeyou_cup::{CupFile, Waypoint, Elevation, WaypointStyle};

let mut cup_file = CupFile::default();

// Create a waypoint
let waypoint = Waypoint {
    name: "Test Field".to_string(),
    code: "TEST".to_string(),
    country: "US".to_string(),
    latitude: 40.25833,  // 40°15.500'N
    longitude: -73.92083, // 73°55.250'W  
    elevation: Elevation::Meters(150.0),
    style: WaypointStyle::SolidAirfield,
    runway_direction: Some(90),
    runway_length: Some(seeyou_cup::RunwayDimension::Meters(800.0)),
    runway_width: None,
    frequency: "123.500".to_string(),
    description: "Test airfield".to_string(),
    userdata: "".to_string(),
    pictures: Vec::new(),
};

cup_file.waypoints.push(waypoint);

// Write to file
cup_file.to_path("output.cup").unwrap();

// Or get as string
let cup_string = cup_file.to_string().unwrap();
```

### Working with different encodings

```rust,no_run
use seeyou_cup::{CupFile, CupEncoding};

// Read with specific encoding
let (cup_file, warnings) = CupFile::from_path_with_encoding("waypoints.cup", CupEncoding::Windows1252).unwrap();

// Write with specific encoding
cup_file.to_path_with_encoding("output.cup", CupEncoding::Utf8).unwrap();
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
