use std::fmt::{Display, Formatter};
use std::str::FromStr;

macro_rules! dimension_enum {
    (
        $(#[$meta:meta])*
        $name:ident,
        $display_name:literal,
        [
            $( $variant:ident = $suffix:literal ),* $(,)?
        ]
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq)]
        pub enum $name {
            $( $variant(f64) ),*
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match self {
                    $( $name::$variant(value) => write!(f, "{value}{}", $suffix) ),*
                }
            }
        }

        impl FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let s = s.trim();

                $(
                    if let Some(value_str) = s.strip_suffix($suffix) {
                        let value: f64 = value_str
                            .parse()
                            .map_err(|_| format!("Invalid {}: {s}", $display_name))?;
                        return Ok($name::$variant(value));
                    }
                )*

                if let Some(unit_start) = s.chars().position(|c| c.is_alphabetic()) {
                    let unit = &s[unit_start..];
                    return Err(format!("Invalid {} unit: {unit}", $display_name));
                }

                let value: f64 = s
                    .parse()
                    .map_err(|_| format!("Invalid {}: {s}", $display_name))?;
                Ok($name::Meters(value))
            }
        }
    };
}

/// Waypoint information from a CUP file
#[derive(Debug)]
pub struct Waypoint {
    /// Waypoint name
    pub name: String,
    /// Short waypoint identifier code
    pub code: String,
    /// Country code (IANA Top level domain standard)
    pub country: String,
    /// Latitude in decimal degrees (WGS-1984)
    pub latitude: f64,
    /// Longitude in decimal degrees (WGS-1984)
    pub longitude: f64,
    /// Elevation above sea level
    pub elevation: Elevation,
    /// Waypoint style/type
    pub style: WaypointStyle,
    /// Runway direction in degrees (0-359)
    pub runway_direction: Option<u16>,
    /// Runway length
    pub runway_length: Option<RunwayDimension>,
    /// Runway width
    pub runway_width: Option<RunwayDimension>,
    /// Radio frequency
    pub frequency: String,
    /// Waypoint description
    pub description: String,
    /// User-defined data
    pub userdata: String,
    /// Picture filenames (stored in pics/ folder of pics.zip)
    pub pictures: Vec<String>,
}

dimension_enum!(
    /// Elevation measurement with unit
    Elevation,
    "elevation",
    [Feet = "ft", Meters = "m"]
);

impl Elevation {
    pub fn to_meters(&self) -> f64 {
        match self {
            Elevation::Meters(m) => *m,
            Elevation::Feet(ft) => ft * 0.3048,
        }
    }

    pub fn to_feet(&self) -> f64 {
        match self {
            Elevation::Meters(m) => m / 0.3048,
            Elevation::Feet(ft) => *ft,
        }
    }
}

dimension_enum!(
    /// Runway dimension measurement with unit
    RunwayDimension,
    "runway dimension",
    [NauticalMiles = "nm", StatuteMiles = "ml", Meters = "m"]
);

impl RunwayDimension {
    pub fn to_meters(&self) -> f64 {
        match self {
            RunwayDimension::Meters(m) => *m,
            RunwayDimension::NauticalMiles(nm) => nm * 1852.0,
            RunwayDimension::StatuteMiles(mi) => mi * 1609.344,
        }
    }
}

dimension_enum!(
    /// Distance measurement with unit
    Distance,
    "distance",
    [
        Kilometers = "km",
        NauticalMiles = "nm",
        StatuteMiles = "ml",
        Meters = "m",
    ]
);

impl Distance {
    pub fn to_meters(&self) -> f64 {
        match self {
            Distance::Meters(m) => *m,
            Distance::Kilometers(km) => km * 1000.0,
            Distance::NauticalMiles(nm) => nm * 1852.0,
            Distance::StatuteMiles(mi) => mi * 1609.344,
        }
    }
}

/// Waypoint style/type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaypointStyle {
    Unknown = 0,
    Waypoint = 1,
    GrassAirfield = 2,
    Outlanding = 3,
    GlidingAirfield = 4,
    SolidAirfield = 5,
    MountainPass = 6,
    MountainTop = 7,
    TransmitterMast = 8,
    Vor = 9,
    Ndb = 10,
    CoolingTower = 11,
    Dam = 12,
    Tunnel = 13,
    Bridge = 14,
    PowerPlant = 15,
    Castle = 16,
    Intersection = 17,
    Marker = 18,
    ControlPoint = 19,
    PgTakeOff = 20,
    PgLandingZone = 21,
}

impl WaypointStyle {
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => WaypointStyle::Waypoint,
            2 => WaypointStyle::GrassAirfield,
            3 => WaypointStyle::Outlanding,
            4 => WaypointStyle::GlidingAirfield,
            5 => WaypointStyle::SolidAirfield,
            6 => WaypointStyle::MountainPass,
            7 => WaypointStyle::MountainTop,
            8 => WaypointStyle::TransmitterMast,
            9 => WaypointStyle::Vor,
            10 => WaypointStyle::Ndb,
            11 => WaypointStyle::CoolingTower,
            12 => WaypointStyle::Dam,
            13 => WaypointStyle::Tunnel,
            14 => WaypointStyle::Bridge,
            15 => WaypointStyle::PowerPlant,
            16 => WaypointStyle::Castle,
            17 => WaypointStyle::Intersection,
            18 => WaypointStyle::Marker,
            19 => WaypointStyle::ControlPoint,
            20 => WaypointStyle::PgTakeOff,
            21 => WaypointStyle::PgLandingZone,
            _ => WaypointStyle::Unknown,
        }
    }
}

/// Task definition from a CUP file
#[derive(Debug)]
pub struct Task {
    /// Task description
    pub description: Option<String>,
    /// Names of waypoints in task order
    pub waypoint_names: Vec<String>,
    /// Task options
    pub options: Option<TaskOptions>,
    /// Observation zones for task points
    pub observation_zones: Vec<ObservationZone>,
    /// Additional task points with index and waypoint data
    pub points: Vec<(u32, Waypoint)>,
    /// Alternative start points
    pub multiple_starts: Vec<String>,
}

/// Task options and constraints
#[derive(Debug)]
pub struct TaskOptions {
    /// Opening of start line
    pub no_start: Option<String>,
    /// Designated time for the task
    pub task_time: Option<String>,
    /// Task distance calculation (false = use fixes, true = use waypoints)
    pub wp_dis: Option<bool>,
    /// Distance tolerance
    pub near_dis: Option<Distance>,
    /// Altitude tolerance
    pub near_alt: Option<Elevation>,
    /// Uncompleted leg (false = calculate maximum distance from last observation zone)
    pub min_dis: Option<bool>,
    /// If true, random order of waypoints is checked
    pub random_order: Option<bool>,
    /// Maximum number of points
    pub max_pts: Option<u32>,
    /// Number of mandatory waypoints at the beginning
    pub before_pts: Option<u32>,
    /// Number of mandatory waypoints at the end
    pub after_pts: Option<u32>,
    /// Bonus for crossing the finish line
    pub bonus: Option<f64>,
}

/// Observation zone definition for task points
#[derive(Debug)]
pub struct ObservationZone {
    /// Consecutive number of a waypoint (0 = Start)
    pub index: u32,
    /// Observation zone direction
    pub style: ObsZoneStyle,
    /// Radius 1
    pub r1: Option<Distance>,
    /// Angle 1 in degrees
    pub a1: Option<f64>,
    /// Radius 2
    pub r2: Option<Distance>,
    /// Angle 2 in degrees
    pub a2: Option<f64>,
    /// Angle 12
    pub a12: Option<f64>,
    /// Whether zone is a line
    pub line: Option<bool>,
}

/// Observation zone direction style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObsZoneStyle {
    Fixed = 0,
    Symmetrical = 1,
    ToNextPoint = 2,
    ToPreviousPoint = 3,
    ToStartPoint = 4,
}

impl ObsZoneStyle {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ObsZoneStyle::Fixed),
            1 => Some(ObsZoneStyle::Symmetrical),
            2 => Some(ObsZoneStyle::ToNextPoint),
            3 => Some(ObsZoneStyle::ToPreviousPoint),
            4 => Some(ObsZoneStyle::ToStartPoint),
            _ => None,
        }
    }
}
