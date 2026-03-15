use ayml_serde::Commentable;
use serde::{Deserialize, Serialize};

// ── Deserialization tests ───────────────────────────────────────────

#[test]
fn de_commentable_inline_comment() {
    #[derive(Deserialize, Debug)]
    struct Config {
        port: Commentable<u16>,
    }
    let input = "port: 8080 # the port\n";
    let c: Config = ayml_serde::from_str(input).unwrap();
    assert_eq!(c.port.value, 8080);
    assert_eq!(c.port.top_comment, None);
    assert_eq!(c.port.inline_comment.as_deref(), Some("the port"));
}

#[test]
fn de_commentable_top_comment() {
    #[derive(Deserialize, Debug)]
    struct Config {
        port: Commentable<u16>,
    }
    let input = "# listen port\nport: 8080\n";
    let c: Config = ayml_serde::from_str(input).unwrap();
    assert_eq!(c.port.value, 8080);
    assert_eq!(c.port.top_comment.as_deref(), Some("listen port"));
    assert_eq!(c.port.inline_comment, None);
}

#[test]
fn de_commentable_both_comments() {
    #[derive(Deserialize, Debug)]
    struct Config {
        port: Commentable<u16>,
    }
    let input = "# listen port\nport: 8080 # default\n";
    let c: Config = ayml_serde::from_str(input).unwrap();
    assert_eq!(c.port.value, 8080);
    assert_eq!(c.port.top_comment.as_deref(), Some("listen port"));
    assert_eq!(c.port.inline_comment.as_deref(), Some("default"));
}

#[test]
fn de_commentable_no_comments() {
    #[derive(Deserialize, Debug)]
    struct Config {
        port: Commentable<u16>,
    }
    let input = "port: 8080\n";
    let c: Config = ayml_serde::from_str(input).unwrap();
    assert_eq!(c.port.value, 8080);
    assert_eq!(c.port.top_comment, None);
    assert_eq!(c.port.inline_comment, None);
}

#[test]
fn de_commentable_multiline_top_comment() {
    #[derive(Deserialize, Debug)]
    struct Config {
        port: Commentable<u16>,
    }
    let input = "# line one\n# line two\nport: 8080\n";
    let c: Config = ayml_serde::from_str(input).unwrap();
    assert_eq!(c.port.value, 8080);
    assert_eq!(c.port.top_comment.as_deref(), Some("line one\nline two"));
}

#[test]
fn de_commentable_string_value() {
    #[derive(Deserialize, Debug)]
    struct Config {
        name: Commentable<String>,
    }
    let input = "# the name\nname: hello # greeting\n";
    let c: Config = ayml_serde::from_str(input).unwrap();
    assert_eq!(c.name.value, "hello");
    assert_eq!(c.name.top_comment.as_deref(), Some("the name"));
    assert_eq!(c.name.inline_comment.as_deref(), Some("greeting"));
}

#[test]
fn de_commentable_bool_value() {
    #[derive(Deserialize, Debug)]
    struct Config {
        debug: Commentable<bool>,
    }
    let input = "debug: true # enable debug\n";
    let c: Config = ayml_serde::from_str(input).unwrap();
    assert_eq!(c.debug.value, true);
    assert_eq!(c.debug.inline_comment.as_deref(), Some("enable debug"));
}

#[test]
fn de_commentable_multiple_fields() {
    #[derive(Deserialize, Debug)]
    struct Config {
        host: Commentable<String>,
        port: Commentable<u16>,
    }
    let input = "# hostname\nhost: localhost # server\n# port number\nport: 3000\n";
    let c: Config = ayml_serde::from_str(input).unwrap();
    assert_eq!(c.host.value, "localhost");
    assert_eq!(c.host.top_comment.as_deref(), Some("hostname"));
    assert_eq!(c.host.inline_comment.as_deref(), Some("server"));
    assert_eq!(c.port.value, 3000);
    assert_eq!(c.port.top_comment.as_deref(), Some("port number"));
    assert_eq!(c.port.inline_comment, None);
}

#[test]
fn de_commentable_mixed_with_plain() {
    #[derive(Deserialize, Debug)]
    struct Config {
        host: String,
        port: Commentable<u16>,
        debug: bool,
    }
    let input = "host: localhost\n# the port\nport: 8080 # default\ndebug: false\n";
    let c: Config = ayml_serde::from_str(input).unwrap();
    assert_eq!(c.host, "localhost");
    assert_eq!(c.port.value, 8080);
    assert_eq!(c.port.top_comment.as_deref(), Some("the port"));
    assert_eq!(c.port.inline_comment.as_deref(), Some("default"));
    assert_eq!(c.debug, false);
}

// ── Serialization tests ─────────────────────────────────────────────

#[test]
fn ser_commentable_inline_comment() {
    #[derive(Serialize)]
    struct Config {
        port: Commentable<u16>,
    }
    let c = Config {
        port: Commentable {
            top_comment: None,
            inline_comment: Some("the port".into()),
            value: 8080,
        },
    };
    let s = ayml_serde::to_string(&c).unwrap();
    assert_eq!(s, "port: 8080 # the port\n");
}

#[test]
fn ser_commentable_top_comment() {
    #[derive(Serialize)]
    struct Config {
        port: Commentable<u16>,
    }
    let c = Config {
        port: Commentable {
            top_comment: Some("listen port".into()),
            inline_comment: None,
            value: 8080,
        },
    };
    let s = ayml_serde::to_string(&c).unwrap();
    assert_eq!(s, "port:\n  # listen port\n  8080\n");
}

#[test]
fn ser_commentable_both_comments() {
    #[derive(Serialize)]
    struct Config {
        port: Commentable<u16>,
    }
    let c = Config {
        port: Commentable {
            top_comment: Some("listen port".into()),
            inline_comment: Some("default".into()),
            value: 8080,
        },
    };
    let s = ayml_serde::to_string(&c).unwrap();
    assert_eq!(s, "port:\n  # listen port\n  8080 # default\n");
}

#[test]
fn ser_commentable_no_comments() {
    #[derive(Serialize)]
    struct Config {
        port: Commentable<u16>,
    }
    let c = Config {
        port: Commentable {
            top_comment: None,
            inline_comment: None,
            value: 8080,
        },
    };
    let s = ayml_serde::to_string(&c).unwrap();
    assert_eq!(s, "port: 8080\n");
}

#[test]
fn ser_commentable_multiline_top_comment() {
    #[derive(Serialize)]
    struct Config {
        port: Commentable<u16>,
    }
    let c = Config {
        port: Commentable {
            top_comment: Some("line one\nline two".into()),
            inline_comment: None,
            value: 8080,
        },
    };
    let s = ayml_serde::to_string(&c).unwrap();
    assert_eq!(s, "port:\n  # line one\n  # line two\n  8080\n");
}

// ── Roundtrip tests ─────────────────────────────────────────────────

#[test]
fn roundtrip_commentable_inline() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Config {
        port: Commentable<u16>,
    }
    let input = "port: 8080 # the port\n";
    let c: Config = ayml_serde::from_str(input).unwrap();
    let output = ayml_serde::to_string(&c).unwrap();
    assert_eq!(output, input);
}

#[test]
fn roundtrip_commentable_no_comments() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Config {
        port: Commentable<u16>,
    }
    let input = "port: 8080\n";
    let c: Config = ayml_serde::from_str(input).unwrap();
    let output = ayml_serde::to_string(&c).unwrap();
    assert_eq!(output, input);
}

#[test]
fn roundtrip_commentable_both_comments() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Config {
        port: Commentable<u16>,
    }
    let input = "port:\n  # listen port\n  8080 # default\n";
    let c: Config = ayml_serde::from_str(input).unwrap();
    assert_eq!(c.port.value, 8080);
    assert_eq!(c.port.top_comment.as_deref(), Some("listen port"));
    assert_eq!(c.port.inline_comment.as_deref(), Some("default"));
    let output = ayml_serde::to_string(&c).unwrap();
    assert_eq!(output, input);
}

#[test]
fn roundtrip_commentable_multiple_fields() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Config {
        host: Commentable<String>,
        port: Commentable<u16>,
    }
    let c = Config {
        host: Commentable {
            top_comment: Some("hostname".into()),
            inline_comment: None,
            value: "localhost".into(),
        },
        port: Commentable {
            top_comment: None,
            inline_comment: Some("default".into()),
            value: 3000,
        },
    };
    let s = ayml_serde::to_string(&c).unwrap();
    let c2: Config = ayml_serde::from_str(&s).unwrap();
    assert_eq!(c, c2);
}
