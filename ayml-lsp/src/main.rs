mod convert;

use lsp_server::{Connection, Message, Notification, Response};
use lsp_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, InitializeParams, Position, PublishDiagnosticsParams, Range,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
    notification::Notification as _,
};
use std::collections::HashMap;

fn main() {
    if let Err(e) = run() {
        eprintln!("ayml-lsp error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (connection, io_threads) = Connection::stdio();

    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        ..Default::default()
    };

    let init_params = connection.initialize(serde_json::to_value(&capabilities)?)?;
    let _init_params: InitializeParams = serde_json::from_value(init_params)?;

    main_loop(&connection)?;

    io_threads.join()?;
    Ok(())
}

fn main_loop(connection: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let mut documents: HashMap<Uri, String> = HashMap::new();
    let mut schema_cache: HashMap<String, serde_json::Value> = HashMap::new();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                let resp = Response::new_err(
                    req.id,
                    lsp_server::ErrorCode::MethodNotFound as i32,
                    format!("unhandled method: {}", req.method),
                );
                connection.sender.send(Message::Response(resp))?;
            }
            Message::Notification(not) => match not.method.as_str() {
                lsp_types::notification::DidOpenTextDocument::METHOD => {
                    let params: DidOpenTextDocumentParams = serde_json::from_value(not.params)?;
                    let uri = params.text_document.uri.clone();
                    let text = params.text_document.text.clone();
                    documents.insert(uri.clone(), text.clone());
                    publish_diagnostics(connection, uri, &text, &mut schema_cache)?;
                }
                lsp_types::notification::DidChangeTextDocument::METHOD => {
                    let params: DidChangeTextDocumentParams = serde_json::from_value(not.params)?;
                    let uri = params.text_document.uri.clone();
                    if let Some(change) = params.content_changes.into_iter().next() {
                        documents.insert(uri.clone(), change.text.clone());
                        publish_diagnostics(connection, uri, &change.text, &mut schema_cache)?;
                    }
                }
                lsp_types::notification::DidCloseTextDocument::METHOD => {
                    let params: DidCloseTextDocumentParams = serde_json::from_value(not.params)?;
                    documents.remove(&params.text_document.uri);
                    let params = PublishDiagnosticsParams {
                        uri: params.text_document.uri,
                        diagnostics: vec![],
                        version: None,
                    };
                    send_notification::<lsp_types::notification::PublishDiagnostics>(
                        connection, params,
                    )?;
                }
                _ => {}
            },
            Message::Response(_) => {}
        }
    }

    Ok(())
}

fn publish_diagnostics(
    connection: &Connection,
    uri: Uri,
    text: &str,
    schema_cache: &mut HashMap<String, serde_json::Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut diagnostics = Vec::new();

    match ayml_core::parse(text) {
        Err(e) => {
            let line = e.line.saturating_sub(1) as u32;
            let col = e.column.saturating_sub(1) as u32;
            diagnostics.push(Diagnostic {
                range: Range::new(Position::new(line, col), Position::new(line, col)),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("ayml".to_string()),
                message: format!("{e}"),
                ..Default::default()
            });
        }
        Ok(node) => {
            // Check for schema directive in the document comment.
            if let Some(comment) = &node.comment
                && let Some(schema_url) = ayml_core::schema_uri(comment) {
                    let schema_diagnostics = validate_with_schema(&node, schema_url, schema_cache);
                    diagnostics.extend(schema_diagnostics);
                }
        }
    }

    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };
    send_notification::<lsp_types::notification::PublishDiagnostics>(connection, params)?;
    Ok(())
}

fn validate_with_schema(
    node: &ayml_core::Node,
    schema_url: &str,
    cache: &mut HashMap<String, serde_json::Value>,
) -> Vec<Diagnostic> {
    let schema_value = match cache.get(schema_url) {
        Some(v) => v.clone(),
        None => match fetch_schema(schema_url) {
            Ok(v) => {
                cache.insert(schema_url.to_string(), v.clone());
                v
            }
            Err(e) => {
                return vec![Diagnostic {
                    range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    severity: Some(DiagnosticSeverity::WARNING),
                    source: Some("ayml".to_string()),
                    message: format!("failed to fetch schema: {e}"),
                    ..Default::default()
                }];
            }
        },
    };

    let validator = match jsonschema::validator_for(&schema_value) {
        Ok(v) => v,
        Err(e) => {
            return vec![Diagnostic {
                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                severity: Some(DiagnosticSeverity::WARNING),
                source: Some("ayml".to_string()),
                message: format!("invalid schema: {e}"),
                ..Default::default()
            }];
        }
    };

    let json_value = convert::node_to_json(node);

    validator
        .iter_errors(&json_value)
        .map(|error| {
            let path = error.instance_path().to_string();
            let message = if path.is_empty() {
                format!("{error}")
            } else {
                format!("{path}: {error}")
            };
            Diagnostic {
                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("ayml-schema".to_string()),
                message,
                ..Default::default()
            }
        })
        .collect()
}

fn fetch_schema(url: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if url.starts_with("file://") {
        let path = url.strip_prefix("file://").unwrap();
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        let body = ureq::get(url).call()?.body_mut().read_to_string()?;
        Ok(serde_json::from_str(&body)?)
    }
}

fn send_notification<N: lsp_types::notification::Notification>(
    connection: &Connection,
    params: N::Params,
) -> Result<(), Box<dyn std::error::Error>> {
    let not = Notification::new(N::METHOD.to_string(), params);
    connection.sender.send(Message::Notification(not))?;
    Ok(())
}
