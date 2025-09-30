use crate::{Elevation, RunwayDimension};

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
