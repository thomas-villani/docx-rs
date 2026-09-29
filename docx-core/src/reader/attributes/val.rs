use std::str::FromStr;

use crate::reader::OwnedAttribute;

use super::super::errors::*;

pub fn read_val(attrs: &[OwnedAttribute]) -> Option<String> {
    for a in attrs {
        let local_name = &a.name.local_name;
        if local_name == "val" {
            return Some(a.value.to_owned());
        }
    }
    None
}

// Points per unit of each ECMA-376 universal measure (ST_UniversalMeasure).
const UNIVERSAL_MEASURE_UNITS: [(&str, f64); 6] = [
    ("mm", 72.0 / 25.4),
    ("cm", 72.0 / 2.54),
    ("in", 72.0),
    ("pt", 1.0),
    ("pc", 12.0),
    ("pi", 12.0),
];

/// Reads a measurement that is either a plain number, already in the
/// attribute's own unit, or a universal measure such as `12pt` or `1.5in`,
/// which is converted using `units_per_point`.
fn value_to_units(v: &str, units_per_point: f64) -> Result<i32, ReaderError> {
    for (unit, points) in UNIVERSAL_MEASURE_UNITS {
        if let Some(number) = v.strip_suffix(unit) {
            return Ok((f64::from_str(number)? * points * units_per_point).round() as i32);
        }
    }
    Ok(f64::from_str(v)? as i32)
}

/// Reads a twips measurement (ST_TwipsMeasure, ST_SignedTwipsMeasure).
pub fn value_to_dax(v: &str) -> Result<i32, ReaderError> {
    value_to_units(v, 20.0)
}

/// Reads a half-point measurement (ST_HpsMeasure), e.g. a font size.
pub fn value_to_half_points(v: &str) -> Result<i32, ReaderError> {
    value_to_units(v, 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_numbers_are_already_in_the_target_unit() {
        assert_eq!(value_to_dax("1440").unwrap(), 1440);
        assert_eq!(value_to_dax("-120").unwrap(), -120);
        assert_eq!(value_to_half_points("24").unwrap(), 24);
    }

    #[test]
    fn universal_measures_are_converted() {
        assert_eq!(value_to_dax("122.40pt").unwrap(), 2448);
        assert_eq!(value_to_dax("1in").unwrap(), 1440);
        assert_eq!(value_to_dax("2.54cm").unwrap(), 1440);
        assert_eq!(value_to_dax("25.4mm").unwrap(), 1440);
        assert_eq!(value_to_dax("1pc").unwrap(), 240);
        assert_eq!(value_to_dax("1pi").unwrap(), 240);
        assert_eq!(value_to_dax("-0.5in").unwrap(), -720);
        assert_eq!(value_to_half_points("20pt").unwrap(), 40);
    }

    #[test]
    fn invalid_values_are_errors() {
        assert!(value_to_dax("").is_err());
        assert!(value_to_dax("pt").is_err());
        assert!(value_to_dax("12px").is_err());
    }
}
