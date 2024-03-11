library;

// ANCHOR: strings_import
use std::primitive_conversions::str::*;
// ANCHOR_END: strings_import


pub fn convert_str_to_str_array() {
    // ANCHOR: str_to_str_array
    let fuel_str: str = "fuel";
    let fuel_str_array: str[4] = fuel_str.try_as_str_array().unwrap();
    // ANCHOR_END: str_to_str_array
}

pub fn convert_str_array_to_str() {
    // ANCHOR: str_array_to_str
    let fuel_str_array: str[4] = __to_str_array("fuel");
    let fuel_str: str = from_str_array(fuel_str_array);
    // ANCHOR_END: str_array_to_str
}
