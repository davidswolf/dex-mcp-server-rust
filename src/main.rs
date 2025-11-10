//! Dex MCP Server - Main entry point
//!
//! This is the main executable for the Dex MCP Server, which provides a Model Context
//! Protocol (MCP) interface to the Dex Personal CRM system.

use anyhow::Result;
use dex_mcp_server::client::{AsyncDexClient, AsyncDexClientImpl};
use dex_mcp_server::repositories::{
    ContactRepository, DexContactRepository, DexNoteRepository, DexReminderRepository,
    NoteRepository, ReminderRepository,
};
use dex_mcp_server::{Config, DexClient, DexMcpServer};
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Initialize logging (stderr only to avoid polluting stdout/MCP communication)
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("error"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    // Load configuration
    let config = match Config::from_env() {
        Ok(cfg) => {
            info!("Configuration loaded successfully");
            cfg
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };

    info!(
        "Starting Dex MCP Server with API URL: {}",
        config.dex_api_url
    );

    // Initialize Dex client
    let sync_client = DexClient::new(&config);
    let client = Arc::new(AsyncDexClientImpl::new(sync_client)) as Arc<dyn AsyncDexClient>;

    // Initialize repositories
    let contact_repo =
        Arc::new(DexContactRepository::new(client.clone())) as Arc<dyn ContactRepository>;
    let note_repo = Arc::new(DexNoteRepository::new(client.clone())) as Arc<dyn NoteRepository>;
    let reminder_repo =
        Arc::new(DexReminderRepository::new(client.clone())) as Arc<dyn ReminderRepository>;

    // Cache TTL configuration
    let cache_ttl_secs = config.cache_ttl_minutes * 60; // Convert minutes to seconds

    // Create the MCP server (tools are constructed internally)
    let server = DexMcpServer::new(
        contact_repo,
        note_repo,
        reminder_repo,
        client,
        cache_ttl_secs, // discovery cache TTL
        cache_ttl_secs, // search cache TTL
    );

    info!("Dex MCP Server initialized");
    info!(
        "Cache TTL: {} minutes ({} seconds)",
        config.cache_ttl_minutes, cache_ttl_secs
    );

    // Run the server (this will block until the server exits)
    info!("Starting MCP server with stdio transport");
    dex_mcp_server::server::run_server(server).await?;

    info!("Dex MCP Server shutdown complete");
    Ok(())
}
