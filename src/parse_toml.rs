use crate::*;


impl TomlSchema {

    /// An internal function for matching table entries
    fn find_extra_match<'a>(extras: &'a Vec<TableEntry>, key: &String, value: &'a Value) -> Result<(), Vec<SchemaError<'a>>>
    {
        let mut errors = Vec::new();
        
        for TableEntry { key: ex_key, value: ex_schema } in extras.iter() {
            if ex_key.is_match(key) {
                match ex_schema.check(value) {
                    Ok(()) => {
                        return Ok(())
                    },
                    Err(e) => errors.push(e)
                }
            }
        }
        return Err(errors)
    }

    
    /// This checks that a toml value matches a schema, without modifying/copying, the returned error
    /// cannot outlive the passed toml value or self since it contains references to them
    pub fn check<'a,'b: 'a>(&'a self, data: &'b toml::Value) -> Result<(), SchemaError<'a>> {
        match (self, data) {
            (TomlSchema::String {regex}, Value::String(s)) => {
                if regex.is_match(s) {Ok(())} 
                else {Err(SchemaError::RegexMiss(s, regex.as_str()))}
            },
            (TomlSchema::Integer { min, max }, Value::Integer(i)) => {
                if i >= min && i <= max {Ok(())}
                else {Err(SchemaError::IntMiss { val: *i, min: *min, max: *max })}
            }
            (TomlSchema::Float { min, max, nan_ok }, Value::Float(f)) => {
                if (*nan_ok && f.is_nan()) || (f >= min && f <= max) {Ok(())}
                else {Err(SchemaError::FloatMiss { val: *f, min: *min, max: *max, nan_ok: *nan_ok })}
            }
            (TomlSchema::Date , Value::Datetime(_)) => {
                Ok(())
            },
            (TomlSchema::Bool, Value::Boolean(_)) => {
                Ok(())
            },
            (TomlSchema::Alternative(opts), any) => {
                let mut errors = Vec::with_capacity(opts.len());
                for schema in opts {
                    match schema.check(any) {
                        Ok(()) => {return Ok(());},
                        Err(e) => {errors.push(e);}
                    }
                }
                Err(SchemaError::NoMatch {value: any, errors})
            },
            (TomlSchema::Array { cond, min, max }, Value::Array(arr)) => {
                if arr.len() < *min || arr.len() > *max {
                    return Err(SchemaError::ArrayMiss { count: arr.len(), min: *min, max: *max })
                }
                for val in arr.iter() {
                    if let Err(e) = cond.check(val) {
                        return Err(SchemaError::AtIndex { val: val , error: Box::new(e) })
                    }
                }
                Ok(())
            },
            (TomlSchema::Table { entries, extras, min, max }, Value::Table(table)) => {
                let mut found_extras = 0;
                let mut req_entries = HashSet::with_capacity(entries.len());
                
                for (k,(_,x)) in entries.iter() {
                    if x.is_none() {req_entries.insert(k);}
                }

                for (key,value) in table {
                    match entries.get(key) {
                        // case of an explicit kv pair
                        Some((schema, _)) => {
                            if let Err(e) = schema.check(value) {
                                return Err(SchemaError::AtKey { key: key, error: Box::new(e)})
                            }
                            req_entries.remove(key);
                        },
                        // case of a regex-based kv pair
                        None => {
                            match Self::find_extra_match(extras, key, value) {
                                Ok(()) => {found_extras += 1;}
                                Err(errs) => {
                                    return Err(SchemaError::NoMatch { value, errors: errs })
                                }
                            }
                        }
                    }
                }
                if found_extras >= *min && found_extras <= *max {
                    Ok(())
                } else {
                    Err(SchemaError::TableCount { count: found_extras, min: *min, max: *max })
                }
            }
            
            (s,v) => Err(SchemaError::TypeMismatch{expected: s.into(), got: v.into()})
        }
    }



    /// Check that the data matches the schema and fills in the default values as needed
    /// 
    /// important note: default values are checked against the schema and validation will fail if
    /// a default value does not match the schema, this means that [TomlSchema::check_and_complete] can fail
    /// even when [TomlSchema::check] passes
    pub fn check_and_complete<'a, 'b: 'a>(&'a self, data: &'b mut toml::Value) -> Result<(),SchemaError<'a>>
    {
        // The code for this function is pretty much exactly the same as check, except in Table
        // where default fields are set

        match (self, data) {
            (TomlSchema::String {regex}, Value::String(s)) => {
                if regex.is_match(s) {Ok(())} 
                else {Err(SchemaError::RegexMiss(s, regex.as_str()))}
            },
            (TomlSchema::Integer { min, max }, Value::Integer(i)) => {
                if *i >= *min && *i <= *max {Ok(())}
                else {Err(SchemaError::IntMiss { val: *i, min: *min, max: *max })}
            }
            (TomlSchema::Float { min, max, nan_ok }, Value::Float(f)) => {
                if (*nan_ok && f.is_nan()) || (*f >= *min && *f <= *max) {Ok(())}
                else {Err(SchemaError::FloatMiss { val: *f, min: *min, max: *max, nan_ok: *nan_ok })}
            }
            (TomlSchema::Date, Value::Datetime(_)) => {
                Ok(())
            },
            (TomlSchema::Bool, Value::Boolean(_)) => {
                Ok(())
            },
            (TomlSchema::Alternative(opts), any) => {
                let mut errors = Vec::with_capacity(opts.len());
                for schema in opts {
                    match schema.check(any) {
                        Ok(()) => {return Ok(());},
                        Err(e) => {errors.push(e);}
                    }
                }
                Err(SchemaError::NoMatch {value: any, errors})
            },
            (TomlSchema::Array { cond, min, max }, Value::Array(arr)) => {
                if arr.len() < *min || arr.len() > *max {
                    return Err(SchemaError::ArrayMiss { count: arr.len(), min: *min, max: *max })
                }
                for val in arr.iter() {
                    if let Err(e) = cond.check(val) {
                        return Err(SchemaError::AtIndex { val: val , error: Box::new(e) })
                    }
                }
                Ok(())
            },
            (TomlSchema::Table { entries, extras, min, max }, Value::Table(table)) => {
                let mut found_extras = 0;
                let mut req_entries = HashSet::with_capacity(entries.len());
                
                for (k,(_,def)) in entries.iter() {
                    // DIFFERENT: handle defaults
                    match def {
                        None => {req_entries.insert(k);}
                        Some(d) => {
                            if table.get(k).is_none() {
                                table.insert(k.clone(), d.clone()).unwrap();
                            }
                        }
                    }
                }

                

                for (key,value) in table {
                    match entries.get(key) {
                        // case of an explicit kv pair
                        Some((schema, _)) => {
                            if let Err(e) = schema.check(value) {
                                return Err(SchemaError::AtKey { key: key, error: Box::new(e)})
                            }
                            req_entries.remove(key);
                        },
                        // case of a regex-based kv pair
                        None => {
                            match Self::find_extra_match(extras, key, value) {
                                Ok(()) => {found_extras += 1;}
                                Err(errs) => {
                                    return Err(SchemaError::NoMatch { value, errors: errs })
                                }
                            }
                        }
                    }
                }
                if found_extras >= *min && found_extras <= *max {
                    Ok(())
                } else {
                    Err(SchemaError::TableCount { count: found_extras, min: *min, max: *max })
                }
            }
            
            (s,v) => Err(SchemaError::TypeMismatch{expected: s.into(), got: (&*v).into()})
        }
    }
}