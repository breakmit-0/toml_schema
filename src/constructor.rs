use crate::*;


impl TomlSchema {
    /// The internal constructor for a schema, from toml data, the returned values are
    /// - `Ok(..)` => the schema and it's default value
    /// - `Err(..)` => Some kind of indication on where parsing the schema falied
    /// 
    /// note: no checking is done on the default values, that means the schema may contain default values that dont match it
    /// and fail on valid data using check_and_complete(..)
    pub fn from_table(table: &toml::Table) -> Result<(TomlSchema, Option<Value>),String>
    {
        // get the type of the table if possible
        let type_str = match table.get("type") {
            Some(Value::String(s)) => &s,
            None => "table",
            _ => return Err(format!("Invalid schema format: type should be a string"))
        };

        match SchemaType::try_from(type_str)?
        {
            SchemaType::String => parse_string(table),

            SchemaType::Integer => parse_int(table),

            SchemaType::Date => parse_date(table),

            SchemaType::Bool => parse_bool(table),

            SchemaType::Float => parse_float(table),

            // todo! : implement an Anything schema, that acs as default and allows child-less array
            SchemaType::Array => parse_array(table),

            SchemaType::Table => parse_table(table),

            SchemaType::Alternative => parse_alternative(table),

            SchemaType::Anything => parse_anything(table),

            SchemaType::Exact => parse_exact(table)
        }
    }
}



//=======================================================================================================================
//=======================================================================================================================
//=======================================================================================================================
// parse methods


fn parse_string(table: &toml::Table) -> Result<(TomlSchema, Option<Value>),String>
{
    let mut res = Regex::new(".*").unwrap();
    let mut dv = None;

    for k in table.keys() {
        match k.as_str()
        {
            "type" => (),
            
            "default" => {dv = Some((&table[k]).clone())},
            
            "regex" => {
                if let Value::String(re) = &table[k] {
                    res = Regex::new(re).map_err(|e| e.to_string())?
                } else {
                    return Err(format!("regex must be a string but got {:?}", &table[k]))
                }
            },
            
            other_key => log::warn!("Schema parser got unexpected (ignored) key '{}' in string config", other_key)
        }
    }

    Ok((TomlSchema::String { regex: res }, dv))
}

/* ------------------------------- */

fn parse_int(table: &toml::Table) -> Result<(TomlSchema, Option<Value>),String> 
{
    let mut min = i64::MIN;
    let mut max = i64::MAX;
    let mut dv = None;

    for k in table.keys() {
        match k.as_str()
        {
            "type" => (),
            
            "default" => {dv = Some((&table[k]).clone())},
            
            "min" => {
                if let Value::Integer(i) = &table[k] {min = *i;} 
                else {return Err(format!("Int min must be an int but got {:?}", &table[k]))}
            },
            
            "max" => {
                if let Value::Integer(i) = &table[k] {max = *i;}
                else {return Err(format!("Int max must be an int but got {:?}", &table[k]))} 
            },
            
            other_key => log::warn!("Schema parser got unexpected (ignored) key '{}' in int config", other_key)
        }
    }

    Ok((TomlSchema::Integer { min, max }, dv))
}

/* ------------------------------- */

fn parse_float(table: &toml::Table) -> Result<(TomlSchema, Option<Value>),String>
{
    let mut min = f64::NEG_INFINITY;
    let mut max = f64::INFINITY;
    let mut nan_ok = false;
    let mut dv = None;

    for k in table.keys() {
        match k.as_str()
        {
            "type" => (),
            
            "default" => {dv = Some((&table[k]).clone())},
            
            "min" => {
                if let Value::Float(x) = &table[k] {min = *x;} 
                else {return Err(format!("Float min must be a float but got {:?}", &table[k]))}
            },
            
            "max" => {
                if let Value::Float(x) = &table[k] {max = *x;}
                else {return Err(format!("Float max must be a float but got {:?}", &table[k]))} 
            },
            
            "nan_ok" => {
                if let Value::Boolean(b) = &table[k] {nan_ok = *b} 
                else {return Err(format!("Float nan_ok must be a boolean but got {:?}", &table[k]))}
            }
            
            other_key => log::warn!("Schema parser got unexpected (ignored) key '{}' in float config", other_key)
        }
    }

    Ok((TomlSchema::Float { min, max, nan_ok }, dv))
}

/* ------------------------------- */

fn parse_bool(table: &toml::Table) -> Result<(TomlSchema, Option<Value>),String> 
{
    let mut dv = None;

    for k in table.keys() {
        match k.as_str()
        {
            "type" => (),
            
            "default" => {dv = Some((&table[k]).clone())},
            
            other_key => log::warn!("Schema parser got unexpected (ignored) key '{}' in float config", other_key)
        }
    }

    Ok((TomlSchema::Bool, dv))
}

/* ------------------------------- */

fn parse_date(table: &toml::Table) -> Result<(TomlSchema, Option<Value>),String> 
{
    let mut dv = None;

    for k in table.keys() {
        match k.as_str()
        {
            "type" => (),
            
            "default" => {dv = Some((&table[k]).clone())},
            
            other_key => log::warn!("Schema parser got unexpected (ignored) key '{}' in date config", other_key)
        }
    }

    Ok((TomlSchema::Date, dv))
}

/* ------------------------------- */

fn parse_anything(table: &toml::Table) -> Result<(TomlSchema, Option<Value>),String> 
{
    let mut dv = None;

    for k in table.keys() {
        match k.as_str()
        {
            "type" => (),
            
            "default" => {dv = Some((&table[k]).clone())},
            
            other_key => log::warn!("Schema parser got unexpected (ignored) key '{}' in date config", other_key)
        }
    }

    Ok((TomlSchema::Anything, dv))
}

/* ------------------------------- */

fn parse_exact(table: &toml::Table) -> Result<(TomlSchema, Option<Value>),String> 
{
    let mut dv = None;
    let mut value = None;

    for k in table.keys() {
        match k.as_str()
        {
            "type" => (),
            
            "default" => {dv = Some((&table[k]).clone())},

            "value" => {value = Some(&table[k])}
            
            other_key => log::warn!("Schema parser got unexpected (ignored) key '{}' in date config", other_key)
        }
    }

    match value {
        Some(v) => Ok((TomlSchema::Exact(v.clone()), dv)),
        None => Err("Exact without a value is not allowed".to_string())
    }
    
}

/* ------------------------------- */

fn parse_array(table: &toml::Table) -> Result<(TomlSchema, Option<Value>),String>
{
    let mut min = 0;
    let mut max = usize::MAX;
    let mut cond = None;
    let mut dv = None;

    for k in table.keys() {
        match k.as_str() 
        {
            "type" => (),
            
            "default" => {dv = Some((&table[k]).clone())},
            
            "min" => { 
                match &table[k] {
                    Value::Integer(i) if *i >= 0 => {min = *i as usize;} 
                    _ => {return Err(format!("Array min size must be a positive int but got {:?}", &table[k]))}
                }
            },
            
            "max" => { 
                match &table[k] {
                    Value::Integer(i) if *i >= 0 => {max = *i as usize;} 
                    _ => {return Err(format!("Array max size must be a positive int but got {:?}", &table[k]))}
                }
            },
            
            "child" => {
                if let Value::Table(t) = &table[k] {
                    match TomlSchema::from_table(t) {
                        Err(e) => return Err(format!("Invalid array condition: {:?}", e)),
                        Ok((schema, dv)) => {
                            cond = Some(schema);
                            if let Some(d) = dv {
                                log::warn!("Schema parser got unexpected (ignored) default {:?} for array element", d)
                            }
                        }
                    }
                }
            }
            other_key => log::warn!("Schema parser got unexpected (ignored) key '{}' in array config", other_key)
        }
    }

    match cond {
        Some(cond) => Ok((TomlSchema::Array { cond: Box::new(cond), min, max }, dv)),
        None => return Err("Array without a 'child' key is not allowed".to_string())
    }
}

/* ------------------------------- */

fn parse_table(table: &toml::Table) -> Result<(TomlSchema, Option<Value>),String>
{
    let mut min = 0;
    let mut max = usize::MAX;
    let mut entries = HashMap::new();
    let mut extras = Vec::new();
    let mut dv = None;

    for k in table.keys() {
        match k.as_str() {
            "type" => (),

            "default" =>{ dv = Some((&table[k]).clone())},

            "min" => { 
                match &table[k] {
                    Value::Integer(i) if *i >= 0 => {min = *i as usize;} 
                    _ => {return Err(format!("Array min size must be a positive int but got {:?}", &table[k]))}
                }
            },
            
            "max" => { 
                match &table[k] {
                    Value::Integer(i) if *i >= 0 => {max = *i as usize;} 
                    _ => {return Err(format!("Array max size must be a positive int but got {:?}", &table[k]))}
                }
            },

            "extras" => { match &table[k] {
                Value::Array(arr) =>
                {
                    for extra in arr { match extra {
                        Value::Table(extra_table) =>
                        {
                            // declare these variables here for scoping
                            let extra_key;
                            let extra_schema;

                            // get key regex
                            match extra_table.get("key") {
                                Some(Value::String(s)) =>
                                {
                                    match Regex::new(s) {
                                        Ok(re) => {extra_key = re;},
                                        Err(e) => {return Err(format!("Regex error : {:?}", e))}
                                    }
                                },
                                _ => {return Err(format!("Extra entry keys must be strings but got {:?}", extra_table.get("key")))}
                            }

                            // parse sub-schema
                            match extra_table.get("schema") {
                                Some(Value::Table(t)) =>
                                {
                                    match TomlSchema::from_table(t) {
                                        Ok((sch, dv)) => {
                                            if let Some(d) = dv {
                                                log::warn!("Schema parser got unexpected (ignored) default {:?} in table extras", d);
                                            }
                                            extra_schema = sch;
                                        },
                                        Err(e) => {return Err(format!("In table extra with key {:?} \n{}", extra_key, e))}
                                    }
                                },
                                _ => {return Err(format!("Extra entry schemas must be tables but got {:?}", extra_table.get("schema")))}
                            }

                            //throw a vague warning if there are other keys
                            if extra_table.len() > 2 {
                                log::warn!("Table extra contains unused keys (got {} but expected 2)", extra_table.len())
                            }

                            // and add to registered
                            extras.push(TableEntry { key: extra_key, value: extra_schema });
                        }
                        _ => {return Err(format!("Extra entries must be tables but got {:?}", extra))}
                    }}
                },
                _ => {return Err(format!("Table entries must be an array but got {:?}", &table[k]))}
            }}

            // extra params do not cause errors table only
            _ => {
                let custom_key = 
                    if k.chars().next() == Some('$') {k[1..].to_string()} 
                    else {k.to_string()};

                match &table[k] {
                    Value::Table(t) => {
                        match TomlSchema::from_table(t)
                        {
                            Ok((schema, dv)) => {
                                entries.insert(custom_key, (schema, dv));
                            },
                            Err(e) => {return Err(format!("In schema for key {}\n{}", custom_key, e));}
                        }
                    },
                    _ => {return Err(format!("Schema for key {} should be a table but got {:?}", custom_key, &table[k]))}
                }
            }
        }
    }

    Ok((TomlSchema::Table { extras, min, max, entries }, dv))
}

/* ------------------------------- */

fn parse_alternative(table: &toml::Table) -> Result<(TomlSchema, Option<Value>),String>
{
    let mut options = Vec::new();
    let mut dv = None;

    for k in table.keys() {
        match k.as_str()
        {
            "type" => (),

            "default" =>{ dv = Some((&table[k]).clone())},

            "options" => { match &table[k] {
                Value::Array(arr) =>
                {
                    for opt in arr { match opt {
                        Value::Table(opt_table) =>
                        {
                            match TomlSchema::from_table(opt_table) {
                                Ok((schema, dv)) => {
                                    if let Some(d) = dv {
                                        log::warn!("Schema parser got unexpected (ignored) default {:?} in alternative options", d)
                                    }
                                    options.push(schema)
                                },
                                Err(e) => return Err(format!("In alternative option \n {}", e))
                            }
                        },
                        _ => return Err(format!("Option in alternative must be a table but got {:?}", opt))
                    }}
                },
                _ => return Err(format!("Alternative options must be an array but got {}", &table[k]))
            }}
            other_key => log::warn!("Schema parser got unexpected (ignored) key '{}' in array config", other_key) 
        }
    }

    Ok((TomlSchema::Alternative(options), dv))
}




#[cfg(test)]
mod tests {
    use super::*;

    fn test_parser_string(table: &toml::Table) {
        if let (schema, Some(def)) = TomlSchema::from_table(table).unwrap() {
            match schema {
                TomlSchema::String { regex } => {
                    match def {
                        Value::String(s) => {assert!(regex.is_match(&s))},
                        _ => panic!("Default value is {:?}", def)
                    }
                },
                _ => panic!("Schema is {:?}", schema)
            }
        }
        else {
            panic!("Did not get a default value")
        }
    }

    fn test_parser_int(table: &toml::Table) {
        let (schema, maybe_def) = TomlSchema::from_table(table).unwrap();
        let def = maybe_def.unwrap();

        match (schema, def) {
            (TomlSchema::Integer { min, max }, Value::Integer(i)) => {
                assert!(i >= min &&  i <= max)
            },
            (u,v) => panic!("Incorrect return value {:?} {:?}", u, v)
        }
    } 
    fn test_parser_float(table: &toml::Table) {
        let (schema, maybe_def) = TomlSchema::from_table(table).unwrap();
        let def = maybe_def.unwrap();

        match (schema, def) {
            (TomlSchema::Float { min, max, nan_ok }, Value::Float(i)) => {
                assert!( (nan_ok && i.is_nan()) || (i >= min &&  i <= max))
            },
            (u,v) => panic!("Incorrect return value {:?} {:?}", u, v)
        }
    } 

    #[test]
    fn parser_string() {
        let table = "
            type='string'
            regex='^[a-zA-Z_]+$'
            extra_key='allowed'
            default='lul_LUL'
        ".parse().unwrap();
        test_parser_string(&table);
    }

    #[test]
    #[should_panic]
    fn parser_string_fail() {
        let table = "
            type='string'
            regex=5
            default='lul_LUL'
        ".parse().unwrap();
        test_parser_string(&table)
    }

    #[test]
    fn parser_int() {
        let table = "
            type='int'
            min=0
            max=100
            default=5
        ".parse().unwrap();
        test_parser_int(&table);
    }

    #[test]
    fn parser_float() {
        let table = "
            type='float'
            min=0.0
            max=100.0
            nan_ok=false
            default=5.0
        ".parse().unwrap();
        test_parser_float(&table);
    }

    #[test]
    fn parser_bool() {
        let table = "
            type='bool'
            default=false
        ".parse().unwrap();
        
        let (schema, def) = TomlSchema::from_table(&table).unwrap();

        assert!(matches!(schema, TomlSchema::Bool), "schema is not a boolean, schema = {:?}", schema);
        assert!(matches!(def, Some(Value::Boolean(_))), "def is not a boolean, def = {:?}", def);
    }

    #[test]
    fn parser_date() {
        let table = "
            type='date'
            default=1970-01-01
        ".parse().unwrap();

        let (schema, def) = TomlSchema::from_table(&table).unwrap();

        assert!(matches!(schema, TomlSchema::Date), "schema is not a date, schema = {:?}", schema);
        assert!(matches!(def, Some(Value::Datetime(_))), "def is not a date, def = {:?}", def);
    }

    #[test]
    fn parser_table() {
        let table = "
            type='table'
            '$custom_key' = {type='bool'}
            min = 0
            max = 5
            [[extras]]
            key = '[a-zA-Z]+'
            schema = {type='int', default=1}

        ".parse().unwrap();

        let (schema,def) = TomlSchema::from_table(&table).unwrap();

        assert!(def.is_none());
        match schema {
            TomlSchema::Table { extras, entries, .. } => {
                assert!(entries.len() == 1);
                assert!(extras.len() == 1);

                assert!(entries.get("custom_key").is_some(), "entries invalid: {:?}", entries);
            },
            _ => panic!("schema is not a table but {:?}", schema)
        }
    }

    #[test]
    fn parser_alternative() {
        let table = "
            type='alternative'
            
            [[options]]
            type = 'int'

            [[options]]
            type = 'bool'
        ".parse().unwrap();

        let (schema, def) = TomlSchema::from_table(&table).unwrap();

        assert!(def.is_none());
        match schema {
            TomlSchema::Alternative(opts) => {
                assert!(opts.len() == 2)
            },
            _ => panic!("schema is not an alternative but {:?}", schema)
        }
    }

    #[test]
    fn parser_array() {
        let table = "
            type='array'
            child = {type = 'int'}
            min = 1
        ".parse().unwrap();

        let (schema, def) = TomlSchema::from_table(&table).unwrap();

        assert!(def.is_none());
        match schema {
            TomlSchema::Array { cond, .. } => {
                assert!(matches!(*cond, TomlSchema::Integer { .. }));
            },
            _ => panic!("schema is not an array but {:?}", schema)
        } 
    }

}