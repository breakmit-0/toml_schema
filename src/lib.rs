//! A schema parser for TOML files
//! 
//! This crates aims to provide something similar to [JSON schemas](https://json-schema.org/understanding-json-schema/about) for TOML
//! - Schemas are written in TOML
//! - All TOML types are currently supported
//! - References (and recursive schemas) are not yet supported
//! 
//! This crate is very much new and a lot of functionnalities are not fully tested
//! 
//! Since toml can get pretty complicated with deep nesting, you may also want to write the schema in JSON and convert it to TOML
//! 
//! ## Syntax
//!  
//! A schema is represented by a table with a `type` key, its value may be any of
//! - `string` : any string that matches the specified regex
//! - `int` : a 64 bit signed integer with optional bounds
//! - `float` : a 64 bit float with optional bounds
//! - `bool` : a boolean
//! - `date` : a date
//! - `array` : an array of values that all match a specific schema
//! - `table` : a TOML table with specific keys
//! - `alternative` : an OR operation on sub-patterns
//! 
//! If the parser expects a schema but finds not `type` key, it will assume the type is `table`, but if the type is none of
//! the above, parsing will fail
//! 
//! For each type of schema there, are other keys that are either required of optional to give more details
//! about the schema
//! 
//! A `default` key may also be provided when the schema is the value of a key in a `table` schema
//! to make that key optional, `default` will be ignored in other positions
//! 
//! Any extra keys will be ignored (except in `table`
//! 
//! ### string
//! - `regex` (optional, default = `/.*/`) : a regular expression that must be found in the string, if you want to match the whole string,
//! use '^' and '$'
//! 
//! ### int
//! - `min` (optional, default = [i64::MIN]) : the minmimum value allowed
//! - `max` (optional, default = [i64::MAX]) : the maximum value allowed
//! 
//! ### float
//! - `min` (optional, default = [f64::NEG_INFINITY]) : the minmimum value allowed
//! - `max` (optional, default = [f64::INFINITY]) : the maximum value allowed
//! - `nan_ok` (optional, default = `false`) : if this is true, [f64::NAN] is accepted
//! 
//! ### bool
//! 
//! ### date
//! 
//! ### array
//! - `child` (required) : a schema that all elements of this array must match
//! - `min` (optional, default = `0`) : the minimum number of elements
//! - `max` (optional, default = [usize::MAX]) : the maximum number of elements
//! 
//! ### table
//! - `extras` (optional, default = `[]`) : an array of tables with a `key` and `schema` key that defines regex-based key-value pairs
//! - `extras[n].key` (required) : a regular expression that must be found in the key 
//! - `extras[n].schema` (required) : a schema that must be matched by the value
//! - `min` (optional, default = `0`) : the minimum number of extra keys
//! - `max` (optional, default = `0`) : the maximum number of extra keys
//! 
//! All other keys must be schemas, they defined a table key (optional if `default` is provided in this schema) that must match
//! the schema, a '$' is stripped from the beginning of the key if it exists to allow escaping schema keywords, if you want a key
//! that starts with '$', start your key with "$$" etc...
//! 
//! All keys in the TOML table beeing matched are matched against entries before extra keys, this means that if a key matches an
//! entry and an extra, it will not count towards the number of extra keys, this means that you may want to make extra key 
//! regular expressions mutually excusive with the table entries
//! 
//! ### alternative
//! - `options` (required) : an array of schemas, a TOML value matches if any of them match
//! 
//! ## Example
//!  ```toml
//!  type = "table"
//!  "$extras" = {type = "int"} # prefix a key with '$' to escape schema keys
//!  some_key = {type = "int"} # '$' is not needed here since 'some key' is not a schema key
//!  other_key = {type = "bool", default=false} # optional key
//! 
//!  [[extras]] # extra keys parsed with regex
//!  key = "^[a-z_]+$"
//!  schema = {type = "int"}
//! 
//! ```
//! 
//! ## Planned additions
//! - `reference` : a link to another schema (or the schema itself)
//! - `anything` : a schema that matches anything
//! - `exact` : a schema that matches only one value

use std::collections::{HashMap, HashSet};
use toml::Value;
use regex::Regex;

mod constructor;
mod parse_toml;
mod schema_type;

/// An enum that represents the a kind of schema, used mostly in errors
pub use schema_type::SchemaType;


/// A component of a [TomlSchema], only useful to construct a schema by hand
#[derive(Debug, Clone)]
pub struct TableEntry {
    pub key: Regex,
    pub value: TomlSchema
}


/// The main type of the crate, it can be constructed from a [toml::Table] object or by hand, the main constructor
/// for this type is [TomlSchema::try_from]
#[derive(Debug, Clone)]
pub enum TomlSchema {
    Alternative(Vec<TomlSchema>),
    String{regex: Regex},
    Integer{min: i64, max: i64},
    Date,
    Bool,
    Float{min: f64, max: f64, nan_ok: bool},
    Table{extras: Vec<TableEntry>, min: usize, max: usize, entries: HashMap<String, (TomlSchema, Option<Value>)>},
    Array{cond: Box<TomlSchema>, min: usize, max: usize}
}


impl TryFrom<toml::Table> for TomlSchema {
    type Error = String;

    /// The main constructor for a TomlSchema, calls [TomlSchema::from_table] and discards the default value
    /// 
    /// note: default values are not checked against the schema
    fn try_from(table: toml::Table) -> Result<Self,String>
    {
        let (schema, dv) = TomlSchema::from_table(&table)?;

        if dv.is_some() {log::warn!("Tried to construct a TOML Schema with a default value in the root")}
        
        Ok(schema)
    }
}


/// The error type returned by [TomlSchema::check], it cannot outlive the [TomlSchema] or the [toml::Table] it comes from
#[derive(Clone, PartialEq)]
pub enum SchemaError<'a> {
    TypeMismatch{expected: SchemaType, got: SchemaType},
    RegexMiss(&'a str, &'a str),
    FloatMiss{val: f64, min: f64, max: f64, nan_ok: bool},
    IntMiss{val: i64, min: i64, max: i64},
    ArrayMiss{count: usize, min: usize, max: usize},
    NoMatch{value: &'a Value, errors: Vec<SchemaError<'a>>},
    AtIndex{val: &'a Value, error: Box<SchemaError<'a>>},
    AtKey{key: &'a String, error: Box<SchemaError<'a>>},
    TableCount{count: usize, min: usize, max: usize}
}



impl<'a> std::fmt::Debug for SchemaError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeMismatch{expected, got} => write!(f, "Expected {:?} but got {:?}", expected, got),
            Self::RegexMiss(key, regex) => write!(f, "Regex {:?} does not match {:?}", regex, key),
            Self::FloatMiss { val, min, max, nan_ok } => write!(f, "Float {:?} does not match [{:?},{:?}] (nan:{:?})", val,min,max,nan_ok),
            Self::IntMiss { val, min, max } => write!(f, "Int {:?} does not match [{:?},{:?}]",val,min,max),
            Self::ArrayMiss { count, min, max } => write!(f, "Array count {:?} does not match [{:?},{:?}]",count,min,max),
            Self::NoMatch { value, errors } => write!(f, "No match for value {:?}, error list: ({:?})", value, errors),
            Self::AtIndex { val, error } => write!(f, "At value {:?}(..), got ({:?})", val, error),
            Self::AtKey { key, error } => write!(f, "At key '{:?}', got ({:?})", key, error),
            Self::TableCount { count, min, max } => write!(f, "Table extra count {:?} does not match [{:?},{:?}]",count,min,max),
        }
    }
}






#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_test() {
        let schema_toml = std::fs::read_to_string("test_files/test_schema.toml").unwrap()
            .parse::<toml::Table>().unwrap();
        
        let schema = TomlSchema::try_from(schema_toml).unwrap();

        let test_file = std::fs::read_to_string("Cargo.toml").unwrap();
        schema.check(
            &test_file.parse().unwrap()
        ).unwrap();
    }

    #[test]
    fn parse_fail_test() {
        let schema_toml = std::fs::read_to_string("test_files/test_schema.toml").unwrap()
        .parse::<toml::Table>().unwrap();
    
    let schema = TomlSchema::try_from(schema_toml).unwrap();

    let test_file = std::fs::read_to_string("test_files/test_schema.toml").unwrap();
    schema.check(
        &test_file.parse().unwrap()
    ).unwrap_err();
    }
}