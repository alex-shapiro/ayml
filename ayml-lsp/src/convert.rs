use ayml_core::{MapKey, Node, Value};
use serde_json::json;

/// Convert an AYML [`Node`] into a [`serde_json::Value`], discarding comments.
pub fn node_to_json(node: &Node) -> serde_json::Value {
    value_to_json(&node.value)
}

fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => json!(b),
        Value::Int(i) => json!(i),
        Value::Float(f) => json!(f),
        Value::Str(s) => json!(s),
        Value::Seq(items) => {
            serde_json::Value::Array(items.iter().map(node_to_json).collect())
        }
        Value::Map(map) => {
            let obj = map
                .iter()
                .map(|(k, v)| (map_key_to_string(k), node_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
    }
}

fn map_key_to_string(key: &MapKey) -> String {
    match key {
        MapKey::Bool(b) => b.to_string(),
        MapKey::Int(i) => i.to_string(),
        MapKey::String(s) => s.clone(),
    }
}
