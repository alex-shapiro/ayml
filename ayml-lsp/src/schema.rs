use serde_json::Value as Json;

/// Walk a JSON Schema to find the sub-schema at the given path segments.
/// Follows `properties` for object schemas and `items` for array schemas.
/// Resolves local `$ref` pointers within the root schema.
pub fn resolve_sub_schema<'a>(root: &'a Json, path: &[&str]) -> Option<&'a Json> {
    let mut schema = root;

    for segment in path {
        schema = resolve_refs(root, schema);

        // Try `properties/<key>`
        if let Some(sub) = schema.get("properties").and_then(|p| p.get(*segment)) {
            schema = sub;
            continue;
        }

        // Try `items` (array index)
        if segment.parse::<usize>().is_ok()
            && let Some(items) = schema.get("items")
        {
            schema = items;
            continue;
        }

        // Try `additionalProperties` as object schema
        if let Some(additional) = schema.get("additionalProperties")
            && additional.is_object()
        {
            schema = additional;
            continue;
        }

        return None;
    }

    Some(resolve_refs(root, schema))
}

/// Resolve `$ref` pointers (local JSON pointer refs only).
fn resolve_refs<'a>(root: &'a Json, schema: &'a Json) -> &'a Json {
    if let Some(ref_str) = schema.get("$ref").and_then(|r| r.as_str())
        && let Some(pointer) = ref_str.strip_prefix('#')
        && let Some(resolved) = pointer_lookup(root, pointer)
    {
        return resolved;
    }
    schema
}

/// Look up a JSON pointer (e.g. "/definitions/Foo") in a JSON value.
fn pointer_lookup<'a>(root: &'a Json, pointer: &str) -> Option<&'a Json> {
    if pointer.is_empty() || pointer == "/" {
        return Some(root);
    }
    let segments = pointer.strip_prefix('/')?.split('/');
    let mut current = root;
    for seg in segments {
        // Unescape JSON pointer encoding (~0 = ~, ~1 = /)
        let unescaped = seg.replace("~1", "/").replace("~0", "~");
        current = current.get(&unescaped)?;
    }
    Some(current)
}

/// Build a markdown hover string from a JSON sub-schema.
pub fn hover_content(schema: &Json) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(desc) = schema.get("description").and_then(|d| d.as_str()) {
        parts.push(desc.to_string());
    }

    if let Some(ty) = schema_type_string(schema) {
        parts.push(format!("**Type:** `{ty}`"));
    }

    if let Some(default) = schema.get("default") {
        parts.push(format!("**Default:** `{default}`"));
    }

    if let Some(enum_vals) = schema.get("enum").and_then(|e| e.as_array()) {
        let vals: Vec<String> = enum_vals.iter().map(|v| format!("`{v}`")).collect();
        parts.push(format!("**Allowed values:** {}", vals.join(", ")));
    }

    if let Some(pattern) = schema.get("pattern").and_then(|p| p.as_str()) {
        parts.push(format!("**Pattern:** `{pattern}`"));
    }

    if let Some(min) = schema.get("minimum") {
        parts.push(format!("**Minimum:** `{min}`"));
    }

    if let Some(max) = schema.get("maximum") {
        parts.push(format!("**Maximum:** `{max}`"));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n\n"))
    }
}

/// Extract a human-readable type string from a schema node.
fn schema_type_string(schema: &Json) -> Option<String> {
    // Explicit "type" field
    if let Some(ty) = schema.get("type") {
        if let Some(s) = ty.as_str() {
            return Some(s.to_string());
        }
        if let Some(arr) = ty.as_array() {
            let types: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
            return Some(types.join(" | "));
        }
    }

    // oneOf / anyOf
    for keyword in &["oneOf", "anyOf"] {
        if let Some(variants) = schema.get(*keyword).and_then(|v| v.as_array()) {
            let types: Vec<String> = variants
                .iter()
                .filter_map(|v| {
                    v.get("type")
                        .and_then(|t| t.as_str())
                        .map(String::from)
                        .or_else(|| {
                            v.get("$ref").and_then(|r| r.as_str()).map(|r| {
                                // Show just the definition name from "#/definitions/Foo"
                                r.rsplit('/').next().unwrap_or(r).to_string()
                            })
                        })
                })
                .collect();
            if !types.is_empty() {
                return Some(types.join(" | "));
            }
        }
    }

    None
}
