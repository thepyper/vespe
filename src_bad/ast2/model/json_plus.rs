use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JsonPlusEntity {
    Flag,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    SingleQuotedString(String),
    DoubleQuotedString(String),
    NudeString(String),
    Object(JsonPlusObject),
    Array(Vec<JsonPlusEntity>),
}

impl JsonPlusEntity {
    pub fn as_string_only(&self) -> Option<String> {
        match self {
            JsonPlusEntity::NudeString(x)
            | JsonPlusEntity::SingleQuotedString(x)
            | JsonPlusEntity::DoubleQuotedString(x) => Some(x.clone()),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonPlusEntity::Flag => Some(true),
            JsonPlusEntity::Boolean(x) => Some(*x),
            JsonPlusEntity::Integer(x) => Some(*x != 0),
            JsonPlusEntity::Float(x) => Some(*x != 0.0f64),
            _ => None,
        }
    }
    pub fn as_object(&self) -> Option<&JsonPlusObject> {
        match self {
            JsonPlusEntity::Object(x) => Some(x),
            _ => None,
        }
    }
    pub fn to_prompt(&self) -> String {
        match self {
            JsonPlusEntity::Flag => String::new(),
            JsonPlusEntity::Boolean(x) => {
                format!("{}", if *x { "true" } else { "false" })
            }
            JsonPlusEntity::Integer(x) => format!("{}", x),
            JsonPlusEntity::Float(x) => format!("{}", x),
            JsonPlusEntity::SingleQuotedString(x) => format!("{}", x),
            JsonPlusEntity::DoubleQuotedString(x) => format!("{}", x),
            JsonPlusEntity::NudeString(x) => format!("{}", x),
            JsonPlusEntity::Object(x) => {
                format!("{}", Self::_object_to_string_0(x, ""))
            }
            JsonPlusEntity::Array(x) => {
                format!("{}", Self::_array_to_string_0(x, ""))
            }
        }
    }
    fn _to_string_0(&self, prefix: &str, pre_indent: &str) -> String {
        match self {
            JsonPlusEntity::Flag => String::new(),
            JsonPlusEntity::Boolean(x) => {
                format!("{}{}", prefix, if *x { "true" } else { "false" })
            }
            JsonPlusEntity::Integer(x) => format!("{}{}", prefix, x),
            JsonPlusEntity::Float(x) => format!("{}{}", prefix, x),
            JsonPlusEntity::SingleQuotedString(x) => format!("{}\'{}\'", prefix, x),
            JsonPlusEntity::DoubleQuotedString(x) => format!("{}\"{}\"", prefix, x),
            JsonPlusEntity::NudeString(x) => format!("{}{}", prefix, x),
            JsonPlusEntity::Object(x) => {
                format!("{}{}", prefix, Self::_object_to_string_0(x, pre_indent))
            }
            JsonPlusEntity::Array(x) => {
                format!("{}{}", prefix, Self::_array_to_string_0(x, pre_indent))
            }
        }
    }
    fn _array_to_string_0(array: &Vec<JsonPlusEntity>, pre_indent: &str) -> String {
        let mut s = format!("[");
        let n = array.len();
        let (separator, pre_indent, indent) = match n {
            0 | 1 => (" ", "", "".into()),
            _ => ("\n", pre_indent, format!("\t{}", pre_indent)),
        };
        let mut first = true;
        for value in array {
            if !first {
                s.push_str(",");
            }
            first = false;
            s.push_str(&separator);
            s.push_str(&indent);
            s.push_str(&value._to_string_0("", &indent));
        }
        if !first {
            s.push_str(&separator);
            s.push_str(pre_indent);
        }
        s.push_str("]");
        s
    }
    fn _object_to_string_0(object: &JsonPlusObject, pre_indent: &str) -> String {
        let mut s = format!("{{");
        let n = object.properties.len();
        let (separator, pre_indent, indent) = match n {
            0 | 1 => (" ", "", "".into()),
            _ => ("\n", pre_indent, format!("\t{}", pre_indent)),
        };
        let mut first = true;
        for (key, value) in &object.properties {
            if !first {
                s.push_str(",");
            }
            first = false;
            s.push_str(&separator);
            s.push_str(&indent);
            s.push_str(&key);
            s.push_str(&value._to_string_0(": ", &indent));
        }
        if !first {
            s.push_str(&separator);
            s.push_str(pre_indent);
        }
        s.push_str("}}");
        s
    }
}

impl ToString for JsonPlusEntity {
    fn to_string(&self) -> String {
        JsonPlusEntity::_to_string_0(&self, "", "")
    }
}

impl From<&JsonPlusEntity> for serde_json::Value {
    fn from(jpe: &JsonPlusEntity) -> Self {
        match jpe {
            JsonPlusEntity::Flag => true.into(),
            JsonPlusEntity::Boolean(x) => (*x).into(),
            JsonPlusEntity::Integer(x) => (*x).into(),
            JsonPlusEntity::Float(x) => (*x).into(),
            JsonPlusEntity::SingleQuotedString(x) => x.clone().into(),
            JsonPlusEntity::DoubleQuotedString(x) => x.clone().into(),
            JsonPlusEntity::NudeString(x) => x.clone().into(),
            JsonPlusEntity::Array(x) => {
                Value::Array(x.iter().map(|x| x.into()).collect::<Vec<Value>>())
            }
            JsonPlusEntity::Object(x) => x.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonPlusObject {
    pub properties: BTreeMap<String, JsonPlusEntity>,
}

impl From<&JsonPlusObject> for serde_json::Value {
    fn from(jpo: &JsonPlusObject) -> Self {
        Value::Object(
            jpo.properties
                .iter()
                .map(|(x, y)| (x.clone(), y.into()))
                .collect::<Map<String, Value>>(),
        )
    }
}

impl JsonPlusObject {
    pub fn new() -> Self {
        JsonPlusObject {
            properties: BTreeMap::new(),
        }
    }
    pub fn from_map(properties: BTreeMap<String, JsonPlusEntity>) -> Self {
        JsonPlusObject { properties }
    }
    pub fn insert(&mut self, key: String, value: JsonPlusEntity) {
        self.properties.insert(key, value);
    }
}

impl ToString for JsonPlusObject {
    fn to_string(&self) -> String {
        JsonPlusEntity::_object_to_string_0(&self, "")
    }
}
