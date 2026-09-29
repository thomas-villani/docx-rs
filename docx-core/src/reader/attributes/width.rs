use std::str::FromStr;

use crate::reader::OwnedAttribute;

use crate::types::*;

use super::super::errors::*;

pub fn read_width(attrs: &[OwnedAttribute]) -> Result<(isize, WidthType), ReaderError> {
    let mut w = 0;
    let mut width_type = WidthType::Auto;
    for a in attrs {
        let local_name = &a.name.local_name;
        if local_name == "type" {
            width_type = WidthType::from_str(&a.value)?;
        } else if local_name == "w" {
            w = match a.value.strip_suffix('%') {
                Some(percent) => f64::from_str(percent)? as isize,
                None => super::value_to_dax(&a.value)? as isize,
            };
        }
    }
    Ok((w, width_type))
}
