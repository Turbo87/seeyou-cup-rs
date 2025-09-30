use seeyou_cup::{CupFile, Elevation, RunwayDimension};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <file.cup>", args[0]);
        process::exit(1);
    }

    let file_path = &args[1];

    let cup_file = match CupFile::from_path(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error parsing file: {}", e);
            process::exit(1);
        }
    };

    println!("=== Waypoints ({}) ===\n", cup_file.waypoints.len());

    for (i, wp) in cup_file.waypoints.iter().enumerate() {
        println!("{}. {}", i + 1, wp.name);
        if !wp.code.is_empty() {
            println!("   Code: {}", wp.code);
        }
        if !wp.country.is_empty() {
            println!("   Country: {}", wp.country);
        }
        println!("   Position: {:.6}°, {:.6}°", wp.lat, wp.lon);
        println!("   Elevation: {}", format_elevation(&wp.elev));
        println!("   Style: {:?}", wp.style);

        if let Some(dir) = wp.runway_dir {
            println!("   Runway direction: {}°", dir);
        }
        if let Some(ref len) = wp.runway_len {
            println!("   Runway length: {}", format_runway_dimension(len));
        }
        if let Some(ref width) = wp.runway_width {
            println!("   Runway width: {}", format_runway_dimension(width));
        }
        if let Some(ref freq) = wp.freq {
            println!("   Frequency: {}", freq);
        }
        if let Some(ref desc) = wp.desc {
            println!("   Description: {}", desc);
        }

        println!();
    }

    if !cup_file.tasks.is_empty() {
        println!("\n=== Tasks ({}) ===\n", cup_file.tasks.len());

        for (i, task) in cup_file.tasks.iter().enumerate() {
            println!(
                "{}. {}",
                i + 1,
                task.description.as_deref().unwrap_or("<unnamed>")
            );
            println!("   Waypoints: {}", task.waypoint_names.join(" → "));
            println!("   Points: {} turnpoints", task.points.len());

            if let Some(ref options) = task.options {
                println!("   Options:");
                if let Some(ref no_start) = options.no_start {
                    println!("     No start: {}", no_start);
                }
                if let Some(ref task_time) = options.task_time {
                    println!("     Task time: {}", task_time);
                }
                if let Some(wp_dis) = options.wp_dis {
                    println!("     WP distance: {}", wp_dis);
                }
                if let Some(min_dis) = options.min_dis {
                    println!("     Min distance: {}", min_dis);
                }
                if let Some(random_order) = options.random_order {
                    println!("     Random order: {}", random_order);
                }
                if let Some(max_pts) = options.max_pts {
                    println!("     Max points: {}", max_pts);
                }
            }

            if !task.observation_zones.is_empty() {
                println!("   Observation zones: {}", task.observation_zones.len());
            }

            if !task.multiple_starts.is_empty() {
                println!("   Multiple starts: {}", task.multiple_starts.join(", "));
            }

            println!();
        }
    }
}

fn format_elevation(elev: &Elevation) -> String {
    match elev {
        Elevation::Meters(m) => format!("{:.1}m", m),
        Elevation::Feet(ft) => format!("{:.1}ft", ft),
    }
}

fn format_runway_dimension(dim: &RunwayDimension) -> String {
    match dim {
        RunwayDimension::Meters(m) => format!("{:.0}m", m),
        RunwayDimension::NauticalMiles(nm) => format!("{:.2}nm", nm),
        RunwayDimension::StatuteMiles(mi) => format!("{:.2}mi", mi),
    }
}
