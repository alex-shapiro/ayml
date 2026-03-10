use crate::value::{MapKey, Node, Value};

/// Emit an AYML document to a string.
pub fn emit(node: &Node) -> String {
    let mut out = String::new();
    emit_node(&mut out, node, 0, true);
    // Ensure trailing newline
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn emit_node(out: &mut String, node: &Node, indent: usize, top_level: bool) {
    // Top comment
    if let Some(ref comment) = node.comment {
        for line in comment.lines() {
            emit_indent(out, indent);
            out.push_str("# ");
            out.push_str(line);
            out.push('\n');
        }
    }

    match &node.value {
        Value::Null => {
            if !top_level {
                out.push_str("null");
            } else {
                emit_indent(out, indent);
                out.push_str("null");
            }
        }
        Value::Bool(b) => {
            if top_level {
                emit_indent(out, indent);
            }
            out.push_str(if *b { "true" } else { "false" });
        }
        Value::Int(i) => {
            if top_level {
                emit_indent(out, indent);
            }
            out.push_str(&i.to_string());
        }
        Value::Float(f) => {
            if top_level {
                emit_indent(out, indent);
            }
            if f.is_nan() {
                out.push_str("nan");
            } else if f.is_infinite() {
                if f.is_sign_negative() {
                    out.push_str("-inf");
                } else {
                    out.push_str("inf");
                }
            } else {
                let s = format!("{f}");
                out.push_str(&s);
                // Ensure there's a dot so it doesn't look like an int
                if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                    out.push_str(".0");
                }
            }
        }
        Value::Str(s) => {
            if top_level {
                emit_indent(out, indent);
            }
            emit_string(out, s, indent);
        }
        Value::Seq(entries) => {
            for (i, entry) in entries.iter().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                // Entry comment
                if let Some(ref comment) = entry.comment {
                    for line in comment.lines() {
                        emit_indent(out, indent);
                        out.push_str("# ");
                        out.push_str(line);
                        out.push('\n');
                    }
                }
                emit_indent(out, indent);
                out.push_str("- ");
                emit_seq_entry_value(out, entry, indent + 2);
                // Inline comment
                if let Some(ref ic) = entry.inline_comment {
                    out.push_str(" # ");
                    out.push_str(ic);
                }
                out.push('\n');
            }
        }
        Value::Map(map) => {
            let mut first = true;
            for (key, value_node) in map {
                if !first {
                    out.push('\n');
                }
                first = false;

                // Entry comment
                if let Some(ref comment) = value_node.comment {
                    for line in comment.lines() {
                        emit_indent(out, indent);
                        out.push_str("# ");
                        out.push_str(line);
                        out.push('\n');
                    }
                }

                emit_indent(out, indent);
                emit_map_key(out, key);
                out.push(':');

                if value_node.value.is_collection() {
                    out.push('\n');
                    emit_node(out, &Node::new(value_node.value.clone()), indent, true);
                } else {
                    out.push(' ');
                    emit_node(out, &Node::new(value_node.value.clone()), indent, false);
                }

                // Inline comment
                if let Some(ref ic) = value_node.inline_comment {
                    out.push_str(" # ");
                    out.push_str(ic);
                }

                if !value_node.value.is_collection() {
                    out.push('\n');
                }
            }
        }
    }
}

fn emit_seq_entry_value(out: &mut String, node: &Node, indent: usize) {
    match &node.value {
        Value::Map(_) => {
            // Compact mapping notation
            emit_compact_mapping(out, node, indent);
        }
        Value::Seq(_) => {
            // Nested sequence — use flow style for simplicity
            emit_flow_value(out, &node.value);
        }
        _ => {
            emit_node(out, &Node::new(node.value.clone()), indent, false);
        }
    }
}

fn emit_compact_mapping(out: &mut String, node: &Node, indent: usize) {
    if let Value::Map(ref map) = node.value {
        let mut first = true;
        for (key, value_node) in map {
            if !first {
                out.push('\n');
                emit_indent(out, indent);
            }
            first = false;

            emit_map_key(out, key);
            out.push(':');

            if value_node.value.is_collection() {
                out.push('\n');
                emit_node(out, &Node::new(value_node.value.clone()), indent, true);
            } else {
                out.push(' ');
                emit_node(out, &Node::new(value_node.value.clone()), indent, false);
            }
        }
    }
}

fn emit_flow_value(out: &mut String, value: &Value) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(i) => out.push_str(&i.to_string()),
        Value::Float(f) => {
            if f.is_nan() {
                out.push_str("nan");
            } else if f.is_infinite() {
                out.push_str(if f.is_sign_negative() { "-inf" } else { "inf" });
            } else {
                out.push_str(&format!("{f}"));
            }
        }
        Value::Str(s) => {
            out.push('"');
            for ch in s.chars() {
                match ch {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    ch if ch.is_control() => {
                        out.push_str(&format!("\\u{:04X}", ch as u32));
                    }
                    _ => out.push(ch),
                }
            }
            out.push('"');
        }
        Value::Seq(entries) => {
            out.push('[');
            for (i, entry) in entries.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                emit_flow_value(out, &entry.value);
            }
            out.push(']');
        }
        Value::Map(map) => {
            out.push('{');
            let mut first = true;
            for (key, value_node) in map {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                emit_map_key(out, key);
                out.push_str(": ");
                emit_flow_value(out, &value_node.value);
            }
            out.push('}');
        }
    }
}

fn emit_map_key(out: &mut String, key: &MapKey) {
    match key {
        MapKey::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        MapKey::Int(i) => out.push_str(&i.to_string()),
        MapKey::String(s) => {
            if needs_quoting(s) {
                out.push('"');
                for ch in s.chars() {
                    match ch {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        _ => out.push(ch),
                    }
                }
                out.push('"');
            } else {
                out.push_str(s);
            }
        }
    }
}

fn emit_string(out: &mut String, s: &str, indent: usize) {
    if s.contains('\n') {
        // Triple-quoted
        out.push_str("\"\"\"\n");
        for line in s.lines() {
            emit_indent(out, indent + 2);
            out.push_str(line);
            out.push('\n');
        }
        // Handle trailing newline in the original string
        if s.ends_with('\n') {
            emit_indent(out, indent + 2);
            out.push('\n');
        }
        emit_indent(out, indent + 2);
        out.push_str("\"\"\"");
    } else if needs_quoting(s) {
        out.push('"');
        for ch in s.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\t' => out.push_str("\\t"),
                '\r' => out.push_str("\\r"),
                ch if ch.is_control() => {
                    out.push_str(&format!("\\u{:04X}", ch as u32));
                }
                _ => out.push(ch),
            }
        }
        out.push('"');
    } else {
        out.push_str(s);
    }
}

fn needs_quoting(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    // Reserved words
    if matches!(
        s,
        "null" | "true" | "false" | "inf" | "+inf" | "-inf" | "nan"
    ) {
        return true;
    }
    // If it looks like a number, quote it
    if s.parse::<i64>().is_ok() || s.parse::<f64>().is_ok() {
        return true;
    }
    // Contains characters that would cause issues
    s.contains(": ")
        || s.contains(" #")
        || s.starts_with('#')
        || s.starts_with('-')
        || s.starts_with('[')
        || s.starts_with('{')
        || s.starts_with('"')
        || s.contains('\\')
}

fn emit_indent(out: &mut String, n: usize) {
    for _ in 0..n {
        out.push(' ');
    }
}
