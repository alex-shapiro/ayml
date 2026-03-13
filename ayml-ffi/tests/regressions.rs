use std::ffi::CString;

use ayml_ffi::*;

// ── Fix 1: ayml_node_string no longer leaks (strings freed with doc) ──

#[test]
fn node_string_on_non_string_returns_null() {
    unsafe {
        let input = CString::new("key: hello").unwrap();
        let doc = ayml_parse(input.as_ptr());
        assert!(!doc.is_null());

        let root = ayml_root(doc);
        assert!(!root.is_null());

        // Root is a mapping, not a string, so this should be null.
        let s = ayml_node_string(doc, root);
        assert!(s.is_null());

        ayml_node_free(root as *mut AymlNode);
        ayml_free(doc);
    }
}

#[test]
fn node_string_on_string_value_returns_valid_ptr() {
    unsafe {
        let input = CString::new("hello world").unwrap();
        let doc = ayml_parse(input.as_ptr());
        assert!(!doc.is_null());

        let root = ayml_root(doc);
        assert!(!root.is_null());

        let s = ayml_node_string(doc, root);
        assert!(!s.is_null());

        let result = std::ffi::CStr::from_ptr(s).to_str().unwrap();
        assert_eq!(result, "hello world");

        // Call again — both pointers should remain valid (stored in doc).
        let s2 = ayml_node_string(doc, root);
        assert!(!s2.is_null());
        let result2 = std::ffi::CStr::from_ptr(s2).to_str().unwrap();
        assert_eq!(result2, "hello world");

        // Original pointer is still valid too.
        let result_again = std::ffi::CStr::from_ptr(s).to_str().unwrap();
        assert_eq!(result_again, "hello world");

        ayml_node_free(root as *mut AymlNode);
        ayml_free(doc);
    }
}

#[test]
fn node_string_many_calls_no_crash() {
    unsafe {
        let input = CString::new("hello").unwrap();
        let doc = ayml_parse(input.as_ptr());
        assert!(!doc.is_null());

        let root = ayml_root(doc);
        // Call ayml_node_string many times — previously each call leaked a CString.
        // Now they're stored in the document and freed together.
        for _ in 0..100 {
            let s = ayml_node_string(doc, root);
            assert!(!s.is_null());
        }

        ayml_node_free(root as *mut AymlNode);
        ayml_free(doc);
    }
}

// ── Fix 2: Null safety on FFI functions ─────────────────────────

#[test]
fn null_doc_returns_null_root() {
    unsafe {
        let root = ayml_root(std::ptr::null());
        assert!(root.is_null());
    }
}

#[test]
fn null_node_returns_defaults() {
    unsafe {
        assert!(matches!(
            ayml_node_type(std::ptr::null()),
            AymlValueType::Null
        ));
        assert_eq!(ayml_node_bool(std::ptr::null()), -1);

        let mut ok: i32 = 99;
        assert_eq!(ayml_node_int(std::ptr::null(), &mut ok), 0);
        assert_eq!(ok, 0);

        ok = 99;
        assert_eq!(ayml_node_float(std::ptr::null(), &mut ok), 0.0);
        assert_eq!(ok, 0);

        assert_eq!(ayml_node_seq_len(std::ptr::null()), 0);
    }
}

#[test]
fn null_node_string_returns_null() {
    unsafe {
        let input = CString::new("hello").unwrap();
        let doc = ayml_parse(input.as_ptr());

        // null node
        let s = ayml_node_string(doc, std::ptr::null());
        assert!(s.is_null());

        // null doc
        let root = ayml_root(doc);
        let s = ayml_node_string(std::ptr::null_mut(), root);
        assert!(s.is_null());

        ayml_node_free(root as *mut AymlNode);
        ayml_free(doc);
    }
}

#[test]
fn free_null_is_safe() {
    unsafe {
        ayml_free(std::ptr::null_mut());
        ayml_node_free(std::ptr::null_mut());
    }
}

#[test]
fn parse_null_input_returns_null() {
    unsafe {
        let doc = ayml_parse(std::ptr::null());
        assert!(doc.is_null());
    }
}
