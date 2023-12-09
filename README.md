# Toml Schema

[![Build](https://github.com/breakmit-0/toml_schema/actions/workflows/rust.yml/badge.svg)](https://github.com/breakmit-0/toml_schema/actions/workflows/rust.yml)

 This crates aims to provide something similar to [JSON schemas](https://json-schema.org/understanding-json-schema/about) for TOML
 - Schemas are written in TOML
 - All TOML types are currently supported
 - References (and recursive schemas) are not yet supported
 
 This crate is very much new and a lot of functionnalities are not fully tested
 
 
 ## Syntax
  
 A schema is represented by a table with a `type` key, its value may be any of
 - `string` : any string that matches the specified regex
 - `int` : a 64 bit signed integer with optional bounds
 - `float` : a 64 bit float with optional bounds
 - `bool` : a boolean
 - `date` : a date
 - `array` : an array of values that all match a specific schema
 - `table` : a TOML table with specific keys
 - `alternative` : an OR operation on sub-patterns
 
 If the parser expects a schema but finds not `type` key, it will assume the type is `table`, but if the type is none of
 the above, parsing will fail
 
 For each type of schema there, are other keys that are either required of optional to give more details
 about the schema
 
 A `default` key may also be provided when the schema is the value of a key in a `table` schema
 to make that key optional, `default` will be ignored in other positions
 
 Any extra keys will be ignored (except in `table`
 
 ### string
 - `regex` (optional, default = `/.*/`) : a regular expression that must be found in the string, if you want to match the whole string,
 use '^' and '$'
 
 ### int
 - `min` (optional, default = [i64::MIN]) : the minmimum value allowed
 - `max` (optional, default = [i64::MAX]) : the maximum value allowed
 
 ### float
 - `min` (optional, default = [f64::NEG_INFINITY]) : the minmimum value allowed
 - `max` (optional, default = [f64::INFINITY]) : the maximum value allowed
 - `nan_ok` (optional, default = `false`) : if this is true, [f64::NAN] is accepted
 
 ### bool
 
 ### date
 
 ### array
 - `child` (required) : a schema that all elements of this array must match
 - `min` (optional, default = `0`) : the minimum number of elements
 - `max` (optional, default = [usize::MAX]) : the maximum number of elements
 
 ### table
 - `extras` (optional, default = `[]`) : an array of tables with a `key` and `schema` key that defines regex-based key-value pairs
 - `extras[n].key` (required) : a regular expression that must be found in the key 
 - `extras[n].schema` (required) : a schema that must be matched by the value
 - `min` (optional, default = `0`) : the minimum number of extra keys
 - `max` (optional, default = `0`) : the maximum number of extra keys
 
 All other keys must be schemas, they defined a table key (optional if `default` is provided in this schema) that must match
 the schema, a `$` is stripped from the beginning of the key if it exists to allow escaping schema keywords, if you want a key
 that starts with `$`, start your key with `$$` etc...
 
 All keys in the TOML table beeing matched are matched against entries before extra keys, this means that if a key matches an
 entry and an extra, it will not count towards the number of extra keys, this means that you may want to make extra key 
 regular expressions mutually excusive with the table entries
 
 ### alternative
 - `options` (required) : an array of schemas, a TOML value matches if any of them match
 
 ## Examples
 
 - To match any table
 
 ```toml
 type = "table"
 extras = [{key = ".*", schema = {type = "anything"}}]
 ```
 
 - to match an array of strings
 ```toml
 type = "array"
 child = {type = "string"}
 ```
 - you may find a basic schema for Cargo.toml files on github at "test_files/test_schema.toml"
 
 
 ## Planned additions
 - `reference` : a link to another schema (or the schema itself)