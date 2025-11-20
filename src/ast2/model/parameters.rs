use serde::{Deserialize, Serialize};

use super::json_plus::{JsonPlusEntity, JsonPlusObject};
use super::range::Range;

/// A collection of key-value parameters associated with a `Tag` or `Anchor`.
///
/// Parameters are parsed from a `[key=value, ...]` syntax.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    /// The map of parameter keys to their JSON values.
    pub(crate) parameters: JsonPlusObject,
    /// The location of the parameter block in the source document.
    pub range: Range,
}

impl Parameters {
    /// Creates a new, empty set of parameters.
    pub fn new() -> Self {
        Parameters {
            parameters: JsonPlusObject::new(),
            range: Range::null(),
        }
    }
    pub fn from_json_object_range(parameters: JsonPlusObject, range: Range) -> Self {
        Parameters { parameters, range }
    }
    pub fn get(&self, key: &str) -> Option<&JsonPlusEntity> {
        self.parameters.properties.get(key)
    }
    pub fn get_as_string_only(&self, key: &str) -> Option<String> {
        match self.get(key) {
            None => None,
            Some(x) => x.as_string_only(),
        }
    }
    pub fn get_as_bool(&self, key: &str) -> bool {
        match self.get(key) {
            None => false,
            Some(x) => x.as_bool().unwrap_or(false),
        }
    }
    pub fn insert(&mut self, key: String, value: JsonPlusEntity) {
        self.parameters.properties.insert(key, value);
    }
    pub fn remove(&mut self, key: &str) {
        self.parameters.properties.remove(key);
    }
    pub fn update(mut self, other: &Parameters) -> Self {
        for parameter in other.parameters.properties.iter() {
            self.parameters
                .properties
                .insert(parameter.0.clone(), parameter.1.clone());
        }
        self
    }
    pub fn integrate(mut self, other: &Parameters) -> Self {
        for parameter in other.parameters.properties.iter() {
            if let Some(_) = self.parameters.properties.get(parameter.0) {
                continue;
            }
            self.parameters
                .properties
                .insert(parameter.0.clone(), parameter.1.clone());
        }
        self
    }
}

impl ToString for Parameters {
    fn to_string(&self) -> String {
        self.parameters.to_string()
    }
}
