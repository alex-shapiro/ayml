//! Roundtrip tests: serialize a value to AYML, then deserialize it back
//! and verify equality. Covers all serde data model types.

use ayml::{from_str, to_string};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Debug;

fn roundtrip<T: Serialize + for<'de> Deserialize<'de> + Debug + PartialEq>(value: &T) {
    let s = to_string(value).unwrap_or_else(|e| panic!("serialize failed for {value:?}: {e}"));
    let back: T =
        from_str(&s).unwrap_or_else(|e| panic!("deserialize failed for {value:?}: {e}\nayml: {s}"));
    assert_eq!(&back, value, "roundtrip mismatch for ayml:\n{s}");
}

// ── Null / Unit ─────────────────────────────────────────────────

#[test]
fn roundtrip_null() {
    roundtrip(&());
}

#[test]
fn roundtrip_unit_struct() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Marker;
    roundtrip(&Marker);
}

// ── Booleans ────────────────────────────────────────────────────

#[test]
fn roundtrip_bool() {
    roundtrip(&true);
    roundtrip(&false);
}

// ── Integers ────────────────────────────────────────────────────

#[test]
fn roundtrip_i8() {
    roundtrip(&0i8);
    roundtrip(&i8::MIN);
    roundtrip(&i8::MAX);
}

#[test]
fn roundtrip_i16() {
    roundtrip(&0i16);
    roundtrip(&i16::MIN);
    roundtrip(&i16::MAX);
}

#[test]
fn roundtrip_i32() {
    roundtrip(&0i32);
    roundtrip(&i32::MIN);
    roundtrip(&i32::MAX);
    roundtrip(&42i32);
    roundtrip(&-17i32);
}

#[test]
fn roundtrip_i64() {
    roundtrip(&0i64);
    roundtrip(&i64::MIN);
    roundtrip(&i64::MAX);
}

#[test]
fn roundtrip_u8() {
    roundtrip(&0u8);
    roundtrip(&u8::MAX);
}

#[test]
fn roundtrip_u16() {
    roundtrip(&0u16);
    roundtrip(&u16::MAX);
}

#[test]
fn roundtrip_u32() {
    roundtrip(&0u32);
    roundtrip(&u32::MAX);
}

#[test]
fn roundtrip_u64() {
    roundtrip(&0u64);
    roundtrip(&u64::MAX);
}

// ── Floats ──────────────────────────────────────────────────────

#[test]
fn roundtrip_f64() {
    roundtrip(&0.0f64);
    roundtrip(&3.25f64);
    roundtrip(&-0.5f64);
    roundtrip(&1.0f64);
    roundtrip(&f64::MIN);
    roundtrip(&f64::MAX);
    roundtrip(&f64::EPSILON);
    roundtrip(&1e10f64);
    roundtrip(&1.23015e3f64);
}

#[test]
fn roundtrip_f64_inf() {
    roundtrip(&f64::INFINITY);
    roundtrip(&f64::NEG_INFINITY);
}

#[test]
fn roundtrip_f64_nan() {
    let s = to_string(&f64::NAN).unwrap();
    let back: f64 = from_str(&s).unwrap();
    assert!(back.is_nan());
}

#[test]
fn roundtrip_f64_precision() {
    // Values known to be tricky for float roundtripping
    roundtrip(&51.24817837550540_4f64);
    roundtrip(&-93.3113703768803_3f64);
    roundtrip(&2.0030397744267762e-253f64);
    roundtrip(&7.101215824554616e260f64);
}

#[test]
fn roundtrip_f32() {
    roundtrip(&0.0f32);
    roundtrip(&3.25f32);
    roundtrip(&f32::INFINITY);
    roundtrip(&f32::NEG_INFINITY);
}

#[test]
fn roundtrip_f32_nan() {
    let s = to_string(&f32::NAN).unwrap();
    let back: f32 = from_str(&s).unwrap();
    assert!(back.is_nan());
}

// ── Strings ─────────────────────────────────────────────────────

#[test]
fn roundtrip_string_simple() {
    roundtrip(&String::from("hello"));
    roundtrip(&String::from("hello world"));
    roundtrip(&String::from("https://example.com/path"));
}

#[test]
fn roundtrip_string_empty() {
    roundtrip(&String::new());
}

#[test]
fn roundtrip_string_reserved_words() {
    // These would be parsed as non-string types without quoting
    roundtrip(&String::from("null"));
    roundtrip(&String::from("true"));
    roundtrip(&String::from("false"));
    roundtrip(&String::from("inf"));
    roundtrip(&String::from("+inf"));
    roundtrip(&String::from("-inf"));
    roundtrip(&String::from("nan"));
}

#[test]
fn roundtrip_string_numeric_looking() {
    roundtrip(&String::from("42"));
    roundtrip(&String::from("-17"));
    roundtrip(&String::from("3.25"));
    roundtrip(&String::from("1e10"));
    roundtrip(&String::from("0xFF"));
    roundtrip(&String::from("0b1010"));
    roundtrip(&String::from("0o77"));
}

#[test]
fn roundtrip_string_escapes() {
    roundtrip(&String::from("line\nbreak"));
    roundtrip(&String::from("tab\there"));
    roundtrip(&String::from("carriage\rreturn"));
    roundtrip(&String::from("null\0byte"));
    roundtrip(&String::from("bell\x07ring"));
    roundtrip(&String::from("backspace\x08char"));
    roundtrip(&String::from("escape\x1Bseq"));
    roundtrip(&String::from("quote\"inside"));
    roundtrip(&String::from("back\\slash"));
}

#[test]
fn roundtrip_string_unicode() {
    roundtrip(&String::from("Σ")); // U+03A3
    roundtrip(&String::from("☺")); // U+263A
    roundtrip(&String::from("🎉")); // U+1F389
    roundtrip(&String::from("日本語"));
    roundtrip(&String::from("Ñoño"));
}

#[test]
fn roundtrip_string_indicators() {
    roundtrip(&String::from("- dash start"));
    roundtrip(&String::from(": colon start"));
    roundtrip(&String::from("[bracket"));
    roundtrip(&String::from("{brace"));
    roundtrip(&String::from("#hash"));
    roundtrip(&String::from("key: value"));
    roundtrip(&String::from("value #comment"));
    roundtrip(&String::from(" leading space"));
    roundtrip(&String::from("trailing space "));
}

// ── Chars ───────────────────────────────────────────────────────

#[test]
fn roundtrip_char() {
    roundtrip(&'a');
    roundtrip(&'Z');
    roundtrip(&'0');
    roundtrip(&'Σ');
    roundtrip(&'☺');
}

// ── Options ─────────────────────────────────────────────────────

#[test]
fn roundtrip_option_none() {
    roundtrip(&None::<i32>);
    roundtrip(&None::<String>);
}

#[test]
fn roundtrip_option_some() {
    roundtrip(&Some(42i32));
    roundtrip(&Some(String::from("hello")));
    roundtrip(&Some(true));
    roundtrip(&Some(3.25f64));
}

// ── Sequences ───────────────────────────────────────────────────

#[test]
fn roundtrip_vec_empty() {
    roundtrip(&Vec::<i32>::new());
}

#[test]
fn roundtrip_vec_integers() {
    roundtrip(&vec![1, 2, 3]);
    roundtrip(&vec![-1, 0, 1]);
    roundtrip(&vec![i64::MIN, 0, i64::MAX]);
}

#[test]
fn roundtrip_vec_strings() {
    roundtrip(&vec![String::from("hello"), String::from("world")]);
}

#[test]
fn roundtrip_vec_bools() {
    roundtrip(&vec![true, false, true]);
}

#[test]
fn roundtrip_vec_nested() {
    roundtrip(&vec![vec![1, 2], vec![3, 4]]);
    roundtrip(&vec![vec![vec![1]]]);
}

// ── Tuples ──────────────────────────────────────────────────────

#[test]
fn roundtrip_tuple2() {
    roundtrip(&(42i32, String::from("hello")));
}

#[test]
fn roundtrip_tuple3() {
    roundtrip(&(1i32, 2.0f64, true));
}

// ── Maps ────────────────────────────────────────────────────────

#[test]
fn roundtrip_map_empty() {
    roundtrip(&BTreeMap::<String, i32>::new());
}

#[test]
fn roundtrip_map_string_keys() {
    let mut m = BTreeMap::new();
    m.insert(String::from("a"), 1);
    m.insert(String::from("b"), 2);
    m.insert(String::from("c"), 3);
    roundtrip(&m);
}

#[test]
fn roundtrip_map_nested() {
    let mut inner = BTreeMap::new();
    inner.insert(String::from("x"), 10);
    let mut outer = BTreeMap::new();
    outer.insert(String::from("inner"), inner);
    roundtrip(&outer);
}

// ── Structs ─────────────────────────────────────────────────────

#[test]
fn roundtrip_struct_simple() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Config {
        name: String,
        port: u16,
        debug: bool,
    }
    roundtrip(&Config {
        name: "myapp".into(),
        port: 8080,
        debug: false,
    });
}

#[test]
fn roundtrip_struct_nested() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Outer {
        inner: Inner,
    }
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Inner {
        value: i32,
    }
    roundtrip(&Outer {
        inner: Inner { value: 42 },
    });
}

#[test]
fn roundtrip_struct_deeply_nested() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct A {
        b: B,
    }
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct B {
        c: C,
    }
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct C {
        value: i32,
    }
    roundtrip(&A {
        b: B { c: C { value: 99 } },
    });
}

#[test]
fn roundtrip_struct_with_option() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Config {
        name: String,
        label: Option<String>,
    }
    roundtrip(&Config {
        name: "app".into(),
        label: None,
    });
    roundtrip(&Config {
        name: "app".into(),
        label: Some("prod".into()),
    });
}

#[test]
fn roundtrip_struct_with_vec() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Config {
        items: Vec<String>,
    }
    roundtrip(&Config {
        items: vec!["a".into(), "b".into()],
    });
    roundtrip(&Config { items: vec![] });
}

#[test]
fn roundtrip_struct_with_map() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Config {
        tags: BTreeMap<String, String>,
    }
    let mut tags = BTreeMap::new();
    tags.insert("env".into(), "prod".into());
    tags.insert("region".into(), "us".into());
    roundtrip(&Config { tags });
    roundtrip(&Config {
        tags: BTreeMap::new(),
    });
}

#[test]
fn roundtrip_vec_of_structs() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Item {
        name: String,
        qty: u32,
    }
    roundtrip(&vec![
        Item {
            name: "Widget".into(),
            qty: 5,
        },
        Item {
            name: "Gadget".into(),
            qty: 3,
        },
    ]);
}

#[test]
fn roundtrip_newtype_struct() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Meters(f64);
    roundtrip(&Meters(3.5));
    roundtrip(&Meters(0.0));
    roundtrip(&Meters(-1.0));
}

// ── Enums ───────────────────────────────────────────────────────

#[test]
fn roundtrip_enum_unit() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Color {
        Red,
        Green,
        Blue,
    }
    roundtrip(&Color::Red);
    roundtrip(&Color::Green);
    roundtrip(&Color::Blue);
}

#[test]
fn roundtrip_enum_newtype() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Value {
        Int(i64),
        Float(f64),
        Str(String),
    }
    roundtrip(&Value::Int(42));
    roundtrip(&Value::Float(3.25));
    roundtrip(&Value::Str("hello".into()));
}

#[test]
fn roundtrip_enum_tuple() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Cmd {
        Move(i32, i32),
        Scale(f64, f64, f64),
    }
    roundtrip(&Cmd::Move(10, 20));
    roundtrip(&Cmd::Scale(1.0, 2.0, 3.0));
}

#[test]
fn roundtrip_enum_struct() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Shape {
        Circle { radius: f64 },
        Rect { w: u32, h: u32 },
    }
    roundtrip(&Shape::Circle { radius: 5.0 });
    roundtrip(&Shape::Rect { w: 10, h: 20 });
}

#[test]
fn roundtrip_enum_in_vec() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Color {
        Red,
        Green,
        Blue,
    }
    roundtrip(&vec![Color::Red, Color::Green, Color::Blue]);
}

#[test]
fn roundtrip_enum_in_struct() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[allow(dead_code)]
    enum Status {
        Active,
        Inactive,
    }
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct User {
        name: String,
        status: Status,
    }
    roundtrip(&User {
        name: "Alice".into(),
        status: Status::Active,
    });
}

// ── Complex / spec-inspired examples ────────────────────────────

#[test]
fn roundtrip_spec_invoice() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Invoice {
        invoice: u32,
        date: String,
        #[serde(rename = "bill-to")]
        bill_to: BillTo,
        #[serde(rename = "ship-to")]
        ship_to: Option<String>,
        product: Vec<Product>,
        tax: f64,
        total: f64,
        comments: String,
    }
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct BillTo {
        given: String,
        family: String,
    }
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Product {
        sku: String,
        quantity: u32,
        description: String,
        price: f64,
    }
    roundtrip(&Invoice {
        invoice: 34843,
        date: "2001-01-23".into(),
        bill_to: BillTo {
            given: "Chris".into(),
            family: "Dumars".into(),
        },
        ship_to: None,
        product: vec![
            Product {
                sku: "BL394D".into(),
                quantity: 4,
                description: "Basketball".into(),
                price: 450.0,
            },
            Product {
                sku: "BL4438H".into(),
                quantity: 1,
                description: "Super Hoop".into(),
                price: 2392.0,
            },
        ],
        tax: 251.42,
        total: 4443.42,
        comments: "Late afternoon is best.".into(),
    });
}

#[test]
fn roundtrip_spec_log() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[allow(non_snake_case)]
    struct Log {
        Date: String,
        User: String,
        Fatal: String,
        Stack: Vec<Frame>,
    }
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Frame {
        file: String,
        line: u32,
        code: String,
    }
    roundtrip(&Log {
        Date: "2001-11-23T15:03:17-5:00".into(),
        User: "ed".into(),
        Fatal: "Unknown variable \"bar\"".into(),
        Stack: vec![
            Frame {
                file: "TopClass.py".into(),
                line: 23,
                code: "x = MoreObject(\"345\\n\")".into(),
            },
            Frame {
                file: "MoreClass.py".into(),
                line: 58,
                code: "foo = bar".into(),
            },
        ],
    });
}

#[test]
fn roundtrip_mixed_nesting() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Config {
        name: String,
        debug: bool,
        server: Server,
        ports: Vec<u16>,
        tags: BTreeMap<String, String>,
    }
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Server {
        host: String,
        port: u16,
    }
    let mut tags = BTreeMap::new();
    tags.insert("env".into(), "prod".into());
    tags.insert("version".into(), "1.0".into());
    roundtrip(&Config {
        name: "myapp".into(),
        debug: false,
        server: Server {
            host: "localhost".into(),
            port: 8080,
        },
        ports: vec![80, 443, 8080],
        tags,
    });
}
