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
                    publish_diagnostics(connection, uri, &text)?;
                }
                lsp_types::notification::DidChangeTextDocument::METHOD => {
                    let params: DidChangeTextDocumentParams = serde_json::from_value(not.params)?;
                    let uri = params.text_document.uri.clone();
                    if let Some(change) = params.content_changes.into_iter().next() {
                        documents.insert(uri.clone(), change.text.clone());
                        publish_diagnostics(connection, uri, &change.text)?;
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
) -> Result<(), Box<dyn std::error::Error>> {
    let diagnostics = match ayml_core::parse(text) {
        Ok(_) => vec![],
        Err(e) => {
            let line = e.line.saturating_sub(1) as u32;
            let col = e.column.saturating_sub(1) as u32;
            vec![Diagnostic {
                range: Range::new(Position::new(line, col), Position::new(line, col)),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("ayml".to_string()),
                message: format!("{}", e),
                ..Default::default()
            }]
        }
    };

    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };
    send_notification::<lsp_types::notification::PublishDiagnostics>(connection, params)?;
    Ok(())
}

fn send_notification<N: lsp_types::notification::Notification>(
    connection: &Connection,
    params: N::Params,
) -> Result<(), Box<dyn std::error::Error>> {
    let not = Notification::new(N::METHOD.to_string(), params);
    connection.sender.send(Message::Notification(not))?;
    Ok(())
}
