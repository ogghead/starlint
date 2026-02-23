//! LSP server for starlint.
//!
//! Provides inline diagnostics and quick-fix code actions for JS/TS files
//! via the Language Server Protocol. Reuses `starlint_core::LintSession`
//! directly for zero-overhead linting.
//!
//! # Usage
//!
//! ```bash
//! starlint lsp
//! ```
//!
//! The server communicates over stdio using JSON-RPC (LSP protocol).

pub mod convert;
pub mod document;
pub mod server;
pub mod snippet;

use tower_lsp::{LspService, Server};

use server::Backend;

/// Start the LSP server, reading from stdin and writing to stdout.
pub async fn run_lsp() -> miette::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
