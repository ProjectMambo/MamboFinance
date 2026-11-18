use std::{error::Error, net::SocketAddr};

use axum::Router;
use rmcp::{
    Json, ServerHandler, tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};

use crate::{Application, Capabilities};

pub const MCP_ADDRESS: &str = "127.0.0.1:3210";

#[derive(Clone)]
pub struct FinanceTools {
    application: Application,
}

#[tool_router]
impl FinanceTools {
    #[must_use]
    pub fn new(application: Application) -> Self {
        Self { application }
    }

    #[tool(
        description = "Return the implemented MamboFinance interfaces, tools, unavailable domains, and database schema version. This tool is read-only.",
        annotations(title = "Get MamboFinance capabilities", read_only_hint = true)
    )]
    fn get_capabilities(&self) -> Result<Json<Capabilities>, String> {
        self.application
            .capabilities()
            .map(Json)
            .map_err(|error| error.to_string())
    }
}

#[tool_handler(
    name = "mambofinance",
    version = "0.1.0",
    instructions = "Read the supported MamboFinance capability contract before attempting ledger actions."
)]
impl ServerHandler for FinanceTools {}

/// Serves the read-only foundation tools on the loopback MCP endpoint.
///
/// # Errors
///
/// Returns an error when the address cannot be parsed or bound, or the server fails.
pub async fn run(application: Application) -> Result<(), Box<dyn Error>> {
    let address: SocketAddr = MCP_ADDRESS.parse()?;
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true);
    let service = StreamableHttpService::new(
        move || Ok(FinanceTools::new(application.clone())),
        LocalSessionManager::default().into(),
        config,
    );
    let router = Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind(address).await?;
    eprintln!("MamboFinance MCP listening at http://{address}/mcp");
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_tool_uses_the_shared_application_query() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let tools = FinanceTools::new(
            Application::open(directory.path().join("finance.db")).expect("application"),
        );

        let Json(capabilities) = tools.get_capabilities().expect("capabilities");

        assert_eq!(capabilities.available_tools, ["get_capabilities"]);
        assert_eq!(capabilities.database_schema_version, 1);
    }

    #[test]
    fn capability_tool_declares_its_contract() {
        let tool = FinanceTools::get_capabilities_tool_attr();

        assert_eq!(
            tool.annotations.and_then(|value| value.read_only_hint),
            Some(true)
        );
        let schema = tool.output_schema.expect("output schema");
        assert!(schema.contains_key("properties"));
    }
}
