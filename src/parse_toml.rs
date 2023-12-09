use toml::Table;

use crate::*;


impl TomlSchema {

    /// An internal function for matching table entries
    fn find_extra_match<'s,'v>(extras: &'s Vec<TableEntry>, key: &'v String, value: &'v Value) -> Result<(), Vec<SchemaError<'s,'v>>>
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


    fn check_string<'s,'v>(regex: &'s Regex, s: &'v String) -> Result<(), SchemaError<'s,'v>> {
        if regex.is_match(s) {Ok(())} 
        else {Err(SchemaError::RegexMiss{string: s, re: regex.as_str()})}
    }

    fn check_int(i : i64, min: i64, max: i64) -> Result<(), SchemaError<'static, 'static>> {
        if i >= min && i <= max {Ok(())}
        else {Err(SchemaError::IntMiss { val: i, min: min, max: max })}
    }

    fn check_float(f : f64, min: f64, max: f64, nan_ok: bool) -> Result<(), SchemaError<'static, 'static>> {
        if (nan_ok && f.is_nan()) || (f >= min && f <= max) {Ok(())}
        else {Err(SchemaError::FloatMiss { val: f, min: min, max: max, nan_ok: nan_ok })}
    }

    fn check_alt<'s,'v>(options: &'s Vec<TomlSchema>, val: &'v Value) -> Result<(), SchemaError<'s,'v>> {
        let mut errors = Vec::with_capacity(options.len());
        for schema in options {
            match schema.check(val) {
                Ok(()) => {return Ok(());},
                Err(e) => {errors.push(e);}
            }
        }
        Err(SchemaError::AlternativeMiss {val, errors})
    }

    const OK: Result<(), SchemaError<'static,'static>> = Ok(());

    fn check_array<'s,'v>(child: &'s TomlSchema, min: usize, max: usize, arr: &'v Vec<Value>) -> Result<(), SchemaError<'s,'v>> {
        if arr.len() < min || arr.len() > max {
            return Err(SchemaError::ArrayCount { count: arr.len(), min: min, max: max })
        }
        for val in arr.iter() {
            if let Err(e) = child.check(val) {
                return Err(SchemaError::ArrayMiss{ value: val , error: Box::new(e) })
            }
        }
        Ok(())
    }

    fn check_table<'s,'v>(
        entries: &'s HashMap<String, (TomlSchema, Option<Value>)>, 
        extras: &'s Vec<TableEntry>, min: usize, max: usize, 
        table: &'v Table
    ) -> Result<(), SchemaError<'s,'v>> {

        let mut found_extras = 0;
        let mut req_entries = HashSet::with_capacity(entries.len());
        
        //consider all entries required
        for (k,_) in entries.iter() {req_entries.insert(k);}

        for (key,value) in table {
            match entries.get(key) {
                // first try to match an explicit entry
                Some((schema, _)) => {
                    if let Err(e) = schema.check(value) {
                        return Err(SchemaError::AtKey { key, error: Box::new(e)})
                    }
                    req_entries.remove(key);
                },
                // then one of the regex-based extras
                None => {
                    match Self::find_extra_match(extras, key, value) {
                        Ok(()) => {found_extras += 1;}
                        Err(errs) => {
                            return Err(SchemaError::TableMiss { key, value, errors: errs })
                        }
                    }
                }
            }
        }
        if found_extras >= min && found_extras <= max {Ok(())} 
        else {Err(SchemaError::TableCount { count: found_extras, min, max })}
    }

    /// This checks that a toml value matches a schema, without modifying/copying, the returned error
    /// cannot outlive the passed toml value or self since it contains references to them
    pub fn check<'s,'v>(&'s self, data: &'v toml::Value) -> Result<(), SchemaError<'s,'v>> {
        match (self, data) {
            (TomlSchema::String {regex}, Value::String(s)) =>            {Self::check_string(regex, s)},
            (TomlSchema::Integer { min, max }, Value::Integer(i)) =>     {Self::check_int(*i, *min, *max)}
            (TomlSchema::Float { min, max, nan_ok }, Value::Float(f)) => {Self::check_float(*f, *min, *max, *nan_ok)}
            (TomlSchema::Date , Value::Datetime(_)) =>                   {Self::OK},
            (TomlSchema::Bool, Value::Boolean(_)) =>                     {Self::OK},
            (TomlSchema::Alternative(opts), any) =>                      {Self::check_alt(opts, any)},
            (TomlSchema::Array { cond, min, max }, Value::Array(arr)) => {Self::check_array(cond, *min, *max, arr)},
            (TomlSchema::Anything, _) =>                                 {Self::OK},
            
            (TomlSchema::Table { entries, extras, min, max }, Value::Table(table)) => {
                Self::check_table(entries, extras, *min, *max, table)
            }
            
            (s,v) => Err(SchemaError::TypeMismatch{expected: s.into(), got: v.into()})
        }
    }



    /// Check that the data matches the schema and fills in the default values as needed
    /// 
    /// important note: default values are checked against the schema and validation will fail if
    /// a default value does not match the schema, this means that [TomlSchema::check_and_complete] can fail
    /// even when [TomlSchema::check] passes
    pub fn check_and_complete<'s, 'v>(&'s self, data: &'v mut toml::Value) -> Result<(),SchemaError<'s,'v>>
    {
        match (self, data) {
            (TomlSchema::String {regex}, Value::String(s)) =>            {Self::check_string(regex, s)},
            (TomlSchema::Integer { min, max }, Value::Integer(i)) =>     {Self::check_int(*i, *min, *max)},
            (TomlSchema::Float { min, max, nan_ok }, Value::Float(f)) => {Self::check_float(*f, *min, *max, *nan_ok)},
            (TomlSchema::Date , Value::Datetime(_)) =>                   {Self::OK},
            (TomlSchema::Bool, Value::Boolean(_)) =>                     {Self::OK},
            (TomlSchema::Alternative(opts), any) =>                      {Self::check_alt(opts, any)},
            (TomlSchema::Array { cond, min, max }, Value::Array(arr)) => {Self::check_array(cond, *min, *max, arr)},
            (TomlSchema::Anything, _) =>                                 {Self::OK},

            (TomlSchema::Table { entries, extras, min, max }, Value::Table(table)) => {

                //add default values as needed
                for (key, (_,def_val)) in entries.iter() {
                    if let (Some(dv), None) = (def_val, table.get(key)) {
                        table.insert(key.clone(), dv.clone());
                    }
                }
                Self::check_table(entries, extras, *min, *max, table)
            }
            
            (s,v) => Err(SchemaError::TypeMismatch{expected: s.into(), got: (&*v).into()})
        }
    }
}