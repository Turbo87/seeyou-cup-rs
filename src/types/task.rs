use crate::types::waypoint::Waypoint;
use crate::{Distance, Elevation};

/// Task definition from a CUP file
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, Default, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
