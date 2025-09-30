use std::fmt::{Display, Formatter};
use std::str::FromStr;

macro_rules! dimension_enum {
    (
        $name:ident,
        $display_name:literal,
        [
            $( $variant:ident = $suffix:literal ),* $(,)?
        ]
    ) => {
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

#[derive(Debug)]
pub struct Waypoint {
    pub name: String,
    pub code: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: Elevation,
    pub style: WaypointStyle,
    pub runway_direction: Option<u16>,
    pub runway_length: Option<RunwayDimension>,
    pub runway_width: Option<RunwayDimension>,
    pub frequency: String,
    pub description: String,
    pub userdata: String,
    pub pictures: Vec<String>,
}

dimension_enum!(Elevation, "elevation", [Feet = "ft", Meters = "m"]);

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

#[derive(Debug)]
pub struct Task {
    pub description: Option<String>,
    pub waypoint_names: Vec<String>,
    pub options: Option<TaskOptions>,
    pub observation_zones: Vec<ObservationZone>,
    pub points: Vec<(u32, Waypoint)>,
    pub multiple_starts: Vec<String>,
}

#[derive(Debug)]
pub struct TaskOptions {
    pub no_start: Option<String>,
    pub task_time: Option<String>,
    pub wp_dis: Option<bool>,
    pub near_dis: Option<Distance>,
    pub near_alt: Option<Elevation>,
    pub min_dis: Option<bool>,
    pub random_order: Option<bool>,
    pub max_pts: Option<u32>,
    pub before_pts: Option<u32>,
    pub after_pts: Option<u32>,
    pub bonus: Option<f64>,
}

#[derive(Debug)]
pub struct ObservationZone {
    pub index: u32,
    pub style: ObsZoneStyle,
    pub r1: Option<Distance>,
    pub a1: Option<f64>,
    pub r2: Option<Distance>,
    pub a2: Option<f64>,
    pub a12: Option<f64>,
    pub line: Option<bool>,
}

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
