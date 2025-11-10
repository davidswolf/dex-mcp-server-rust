//! MCP server implementation for Dex Personal CRM.
//!
//! This module provides the MCP protocol server that exposes Dex CRM
//! functionality to AI assistants through the Model Context Protocol.

pub mod handlers;

pub use handlers::DexMcpServer;

use anyhow::Result;
use rmcp::transport::io::stdio;
use rmcp::ServiceExt;

/// Run the Dex MCP server with stdio transport.
///
/// This function starts the MCP server and runs it until completion.
/// It communicates via stdin/stdout using the MCP protocol.
///
/// # Arguments
/// * `server` - The configured DexMcpServer instance
///
/// # Returns
/// An error if the server fails to start or encounters a fatal error
pub async fn run_server(server: DexMcpServer) -> Result<()> {
    // Serve the server with stdio transport
    let service = server.serve(stdio()).await?;

    // Wait for completion
    service.waiting().await?;

    Ok(())
}
