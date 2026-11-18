use std::{env, error::Error, io, net::SocketAddr};

use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use rmcp::{
    Json, ServerHandler,
    handler::server::wrapper::Parameters,
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};
use serde::Serialize;

use crate::{Account, AccountList, Application, ApplicationError, Capabilities, CreateAccount};

pub const MCP_ADDRESS: &str = "127.0.0.1:3210";
pub const MCP_TOKEN_ENV: &str = "MAMBOFINANCE_MCP_TOKEN";

#[derive(Serialize)]
struct ToolFailure<'a> {
    code: &'static str,
    field: Option<&'a str>,
    message: String,
}

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
            .map_err(storage_failure)
    }

    #[tool(
        description = "List active finance accounts in normalized-name order.",
        annotations(title = "List MamboFinance accounts", read_only_hint = true)
    )]
    fn list_accounts(&self) -> Result<Json<AccountList>, String> {
        self.application
            .list_accounts()
            .map(Json)
            .map_err(storage_failure)
    }

    #[tool(
        description = "Create one active finance account after validating its unique name, account kind, and three-letter currency code.",
        annotations(
            title = "Create MamboFinance account",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false
        )
    )]
    fn create_account(
        &self,
        Parameters(input): Parameters<CreateAccount>,
    ) -> Result<Json<Account>, String> {
        self.application
            .create_account(&input)
            .map(Json)
            .map_err(|error| tool_failure(&error))
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
    let token = env::var(MCP_TOKEN_ENV).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "set {MCP_TOKEN_ENV} to a private token of at least 32 letters, digits, '-' or '_'"
            ),
        )
    })?;
    let authorization = authorization_value(&token)?;
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true);
    let service = StreamableHttpService::new(
        move || Ok(FinanceTools::new(application.clone())),
        LocalSessionManager::default().into(),
        config,
    );
    let router = Router::new()
        .nest_service("/mcp", service)
        .layer(middleware::from_fn_with_state(authorization, authorize));
    let listener = tokio::net::TcpListener::bind(address).await?;
    eprintln!("MamboFinance MCP listening at http://{address}/mcp");
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

fn authorization_value(token: &str) -> io::Result<HeaderValue> {
    if token.len() < 32
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{MCP_TOKEN_ENV} must contain at least 32 letters, digits, '-' or '_'"),
        ));
    }
    HeaderValue::from_str(&format!("Bearer {token}"))
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
}

async fn authorize(State(expected): State<HeaderValue>, request: Request, next: Next) -> Response {
    if request.headers().get(header::AUTHORIZATION) == Some(&expected) {
        return next.run(request).await;
    }

    let mut response = StatusCode::UNAUTHORIZED.into_response();
    response
        .headers_mut()
        .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
    response
}

fn tool_failure(error: &ApplicationError) -> String {
    let message = match error {
        ApplicationError::Storage(_) => "storage operation failed".to_owned(),
        ApplicationError::Validation { .. } | ApplicationError::Conflict { .. } => {
            error.to_string()
        }
    };
    rmcp::serde_json::to_string(&ToolFailure {
        code: error.code(),
        field: error.field(),
        message,
    })
    .unwrap_or_else(|_| "{\"code\":\"serialization_failed\"}".to_owned())
}

fn storage_failure(_error: crate::StorageError) -> String {
    "{\"code\":\"storage_failed\",\"field\":null,\"message\":\"storage operation failed\"}"
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn capability_tool_uses_the_shared_application_query() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let tools = FinanceTools::new(
            Application::open(directory.path().join("finance.db")).expect("application"),
        );

        let Json(capabilities) = tools.get_capabilities().expect("capabilities");

        assert_eq!(
            capabilities.available_tools,
            ["get_capabilities", "list_accounts", "create_account"]
        );
        assert_eq!(capabilities.database_schema_version, 2);
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

    #[test]
    fn account_tools_share_application_state_and_annotations() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let tools = FinanceTools::new(
            Application::open(directory.path().join("finance.db")).expect("application"),
        );

        let Json(created) = tools
            .create_account(Parameters(CreateAccount {
                name: "Daily".to_owned(),
                kind: crate::AccountKind::Bank,
                currency: "SGD".to_owned(),
            }))
            .expect("created account");
        let Json(accounts) = tools.list_accounts().expect("accounts");

        assert_eq!(accounts.accounts, [created]);
        let create = FinanceTools::create_account_tool_attr();
        let annotations = create.annotations.expect("annotations");
        assert_eq!(annotations.read_only_hint, Some(false));
        assert_eq!(annotations.destructive_hint, Some(false));
        assert_eq!(annotations.idempotent_hint, Some(false));
        assert!(create.output_schema.is_some());
    }

    #[test]
    fn account_tool_returns_a_stable_validation_error() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let tools = FinanceTools::new(
            Application::open(directory.path().join("finance.db")).expect("application"),
        );

        let Err(error) = tools.create_account(Parameters(CreateAccount {
            name: "Cash".to_owned(),
            kind: crate::AccountKind::Cash,
            currency: "bad currency".to_owned(),
        })) else {
            panic!("invalid currency must fail");
        };
        let value: rmcp::serde_json::Value =
            rmcp::serde_json::from_str(&error).expect("JSON tool error");

        assert_eq!(value["code"], "validation_failed");
        assert_eq!(value["field"], "currency");
    }

    #[test]
    fn bearer_auth_accepts_only_the_exact_header() {
        let expected =
            authorization_value("0123456789abcdef0123456789abcdef").expect("authorization value");
        let mut headers = HeaderMap::new();
        assert_ne!(headers.get(header::AUTHORIZATION), Some(&expected));

        headers.insert(header::AUTHORIZATION, expected.clone());
        assert_eq!(headers.get(header::AUTHORIZATION), Some(&expected));
        assert!(authorization_value("too-short").is_err());
        assert!(authorization_value("0123456789abcdef0123456789abcde!").is_err());
    }
}
