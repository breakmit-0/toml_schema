use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemaType {
    Alternative, String, Integer,
    Date, Bool, Float, Table, Array
}

impl From<&TomlSchema> for SchemaType {
    fn from(value: &TomlSchema) -> Self {
        match value {
            TomlSchema::Alternative(_) => SchemaType::Alternative,
            TomlSchema::String{..} => SchemaType::String,
            TomlSchema::Integer{..} => SchemaType::Integer,
            TomlSchema::Date{..} => SchemaType::Date,
            TomlSchema::Bool => SchemaType::Bool,
            TomlSchema::Float{..} => SchemaType::Float,
            TomlSchema::Table{..} => SchemaType::Table,
            TomlSchema::Array{..} => SchemaType::Array,
        }
    }
}
impl From<&toml::Value> for SchemaType {
    fn from(value: &toml::Value) -> Self {
        match value {
            Value::String(_) => SchemaType::String,
            Value::Integer(_) => SchemaType::Integer,
            Value::Float(_) => SchemaType::Float,
            Value::Boolean(_) => SchemaType::Bool,
            Value::Datetime(_) => SchemaType::Date,
            Value::Array(_) => SchemaType::Array,
            Value::Table(_) => SchemaType::Table,
        }
    }
}

impl TryFrom<&str> for SchemaType {
    type Error = String;
    fn try_from(value: &str) -> Result<Self,String> {
        match value {
            "string" => Ok(SchemaType::String),
            "int" => Ok(SchemaType::Integer),
            "float" => Ok(SchemaType::Float),
            "bool" => Ok(SchemaType::Bool),
            "date" => Ok(SchemaType::Date),
            "array" => Ok(SchemaType::Array),
            "table" => Ok(SchemaType::Table),
            "alternative" => Ok(SchemaType::Alternative),
            _ => Err(format!("Invalid schema type {}", value))
        }
    }
}