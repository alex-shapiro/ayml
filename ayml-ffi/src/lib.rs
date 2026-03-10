use std::ffi::{CStr, CString, c_char};
use std::ptr;

use ayml_core::{Node, Value};

/// Opaque handle to a parsed AYML document.
pub struct AymlDocument {
    root: Node,
}

/// Opaque handle to an AYML node (borrowed from a document).
pub struct AymlNode {
    node: *const Node,
}

/// The type tag for a node's value.
#[repr(C)]
pub enum AymlValueType {
    Null = 0,
    Bool = 1,
    Int = 2,
    Float = 3,
    String = 4,
    Sequence = 5,
    Mapping = 6,
}

/// Parse an AYML string. Returns null on error.
///
/// # Safety
/// `input` must be a valid null-terminated UTF-8 C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ayml_parse(input: *const c_char) -> *mut AymlDocument {
    if input.is_null() {
        return ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(input) };
    let s = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    match ayml_core::parse(s) {
        Ok(root) => Box::into_raw(Box::new(AymlDocument { root })),
        Err(_) => ptr::null_mut(),
    }
}

/// Free a parsed document.
///
/// # Safety
/// `doc` must be a pointer returned by `ayml_parse`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ayml_free(doc: *mut AymlDocument) {
    if !doc.is_null() {
        drop(unsafe { Box::from_raw(doc) });
    }
}

/// Get the root node of a document.
///
/// # Safety
/// `doc` must be a valid pointer returned by `ayml_parse`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ayml_root(doc: *const AymlDocument) -> *const AymlNode {
    if doc.is_null() {
        return ptr::null();
    }
    let doc = unsafe { &*doc };
    Box::into_raw(Box::new(AymlNode {
        node: &doc.root as *const Node,
    }))
}

/// Get the value type of a node.
///
/// # Safety
/// `node` must be a valid pointer returned by `ayml_root` or node accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ayml_node_type(node: *const AymlNode) -> AymlValueType {
    if node.is_null() {
        return AymlValueType::Null;
    }
    let node = unsafe { &*(*node).node };
    match &node.value {
        Value::Null => AymlValueType::Null,
        Value::Bool(_) => AymlValueType::Bool,
        Value::Int(_) => AymlValueType::Int,
        Value::Float(_) => AymlValueType::Float,
        Value::Str(_) => AymlValueType::String,
        Value::Seq(_) => AymlValueType::Sequence,
        Value::Map(_) => AymlValueType::Mapping,
    }
}

/// Get a boolean value. Returns 0 for false, 1 for true, -1 if not a bool.
///
/// # Safety
/// `node` must be a valid node pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ayml_node_bool(node: *const AymlNode) -> i32 {
    if node.is_null() {
        return -1;
    }
    let node = unsafe { &*(*node).node };
    match &node.value {
        Value::Bool(b) => *b as i32,
        _ => -1,
    }
}

/// Get an integer value. Returns 0 and sets `ok` to 0 if not an int.
///
/// # Safety
/// `node` must be a valid node pointer. `ok` may be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ayml_node_int(node: *const AymlNode, ok: *mut i32) -> i64 {
    if node.is_null() {
        if !ok.is_null() {
            unsafe { *ok = 0 };
        }
        return 0;
    }
    let node = unsafe { &*(*node).node };
    match &node.value {
        Value::Int(i) => {
            if !ok.is_null() {
                unsafe { *ok = 1 };
            }
            *i
        }
        _ => {
            if !ok.is_null() {
                unsafe { *ok = 0 };
            }
            0
        }
    }
}

/// Get a float value.
///
/// # Safety
/// `node` must be a valid node pointer. `ok` may be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ayml_node_float(node: *const AymlNode, ok: *mut i32) -> f64 {
    if node.is_null() {
        if !ok.is_null() {
            unsafe { *ok = 0 };
        }
        return 0.0;
    }
    let node = unsafe { &*(*node).node };
    match &node.value {
        Value::Float(f) => {
            if !ok.is_null() {
                unsafe { *ok = 1 };
            }
            *f
        }
        _ => {
            if !ok.is_null() {
                unsafe { *ok = 0 };
            }
            0.0
        }
    }
}

/// Get a string value. Returns null if not a string. The returned pointer
/// is valid until the document is freed.
///
/// # Safety
/// `node` must be a valid node pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ayml_node_string(node: *const AymlNode) -> *const c_char {
    if node.is_null() {
        return ptr::null();
    }
    let node = unsafe { &*(*node).node };
    match &node.value {
        Value::Str(s) => {
            // Leak a CString — caller must not free this; it lives as long as the doc.
            // TODO: better lifetime management. For now, this leaks.
            match CString::new(s.as_str()) {
                Ok(cs) => cs.into_raw() as *const c_char,
                Err(_) => ptr::null(),
            }
        }
        _ => ptr::null(),
    }
}

/// Get the length of a sequence. Returns 0 if not a sequence.
///
/// # Safety
/// `node` must be a valid node pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ayml_node_seq_len(node: *const AymlNode) -> usize {
    if node.is_null() {
        return 0;
    }
    let node = unsafe { &*(*node).node };
    match &node.value {
        Value::Seq(seq) => seq.len(),
        _ => 0,
    }
}

/// Free a node handle.
///
/// # Safety
/// `node` must be a pointer returned by `ayml_root` or node accessor functions, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ayml_node_free(node: *mut AymlNode) {
    if !node.is_null() {
        drop(unsafe { Box::from_raw(node) });
    }
}
