use crate::FromStr;
use std::fmt::{Display, Formatter};

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
        #[derive(Debug, Clone, PartialEq, PartialOrd)]
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
                            .map_err(|_| format!("Invalid {}: '{s}'", $display_name))?;
                        return Ok($name::$variant(value));
                    }
                )*

                if let Some(unit_start) = s.chars().position(|c| c.is_alphabetic()) {
                    let unit = &s[unit_start..];
                    return Err(format!("Invalid {} unit: '{unit}'", $display_name));
                }

                let value: f64 = s
                    .parse()
                    .map_err(|_| format!("Invalid {}: '{s}'", $display_name))?;
                Ok($name::Meters(value))
            }
        }
    };
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
