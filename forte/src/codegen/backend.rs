use crate::config::ForteConfig;
use crate::watcher::RouteInfo;
use anyhow::{Context, Result};
use std::path::Path;

/// Generate all backend code files (.generated/backend/*)
pub fn generate_backend_code(project_root: &Path, routes: &[RouteInfo], config: &ForteConfig) -> Result<()> {
    let gen_dir = project_root.join(".generated/backend");
    std::fs::create_dir_all(&gen_dir).context("Failed to create .generated/backend")?;

    // Generate error.rs
    let error_rs = generate_error_module();
    std::fs::write(gen_dir.join("error.rs"), error_rs)?;

    // Generate router.rs
    let router_rs = generate_router_module(routes)?;
    std::fs::write(gen_dir.join("router.rs"), router_rs)?;

    // Generate main.rs
    let main_rs = generate_main_module();
    std::fs::write(gen_dir.join("main.rs"), main_rs)?;

    // Generate env.rs
    let env_rs = generate_env_module(config);
    std::fs::write(gen_dir.join("env.rs"), env_rs)?;

    // Update backend/src/routes/mod.rs
    let routes_mod = generate_routes_mod(routes)?;
    let routes_mod_path = project_root.join("backend/src/routes/mod.rs");
    std::fs::write(routes_mod_path, routes_mod)?;

    // Update backend/src/lib.rs to export ActionResult if needed
    let has_actions = routes.iter().any(|r| r.has_action_input);
    let lib_rs = generate_lib_module(has_actions);
    let lib_rs_path = project_root.join("backend/src/lib.rs");
    std::fs::write(lib_rs_path, lib_rs)?;

    println!("  ✓ Generated backend code in .generated/backend/");

    Ok(())
}

fn generate_lib_module(has_actions: bool) -> String {
    let mut output = String::new();
    output.push_str("// [Generated] Library crate for backend routes\n");
    output.push_str("// This file is auto-managed by Forte CLI\n\n");
    output.push_str("pub mod routes;\n");

    if has_actions {
        output.push_str("\n// Re-export ActionResult for use in route handlers\n");
        output.push_str("// This allows routes to use crate::ActionResult<T>\n");
        output.push_str("#[path = \"../../.generated/backend/error.rs\"]\n");
        output.push_str("mod _generated_error;\n");
        output.push_str("pub use _generated_error::ActionResult;\n");
    }

    output
}

fn generate_error_module() -> String {
    r#"// [Generated] Do not edit manually

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum AppError {
    NotFound(ErrorBody),
    BadRequest(ErrorBody),
    Validation(ValidationErrorBody),
    Unauthorized(ErrorBody),
    Forbidden(ErrorBody),
    Internal(ErrorBody),
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    pub error: String,
    pub status: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationErrorBody {
    pub error: String,
    pub status: u16,
    pub fields: HashMap<String, Vec<String>>,
}

// ActionResult for post_action functions
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ActionResult<T> {
    Redirect { url: String },
    Render { props: T },
}

impl AppError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(ErrorBody {
            error: msg.into(),
            status: 404,
        })
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(ErrorBody {
            error: msg.into(),
            status: 400,
        })
    }

    pub fn validation(fields: HashMap<String, Vec<String>>) -> Self {
        Self::Validation(ValidationErrorBody {
            error: "Validation failed".to_string(),
            status: 400,
            fields,
        })
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(ErrorBody {
            error: msg.into(),
            status: 401,
        })
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(ErrorBody {
            error: msg.into(),
            status: 403,
        })
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(ErrorBody {
            error: msg.into(),
            status: 500,
        })
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) | Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        (status, axum::Json(&self)).into_response()
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(e) => write!(f, "{}", e.error),
            Self::BadRequest(e) => write!(f, "{}", e.error),
            Self::Validation(e) => write!(f, "{}", e.error),
            Self::Unauthorized(e) => write!(f, "{}", e.error),
            Self::Forbidden(e) => write!(f, "{}", e.error),
            Self::Internal(e) => write!(f, "{}", e.error),
        }
    }
}

impl std::error::Error for AppError {}

// ValidatedJson extractor for automatic validation
// This wraps Axum's Json extractor and adds validation support
use axum::{
    async_trait,
    extract::{FromRequest, Request},
    http::StatusCode as HttpStatusCode,
};

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: serde::de::DeserializeOwned + validator::Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // First, extract JSON
        let axum::Json(value) = axum::Json::<T>::from_request(req, state)
            .await
            .map_err(|_| AppError::bad_request("Invalid JSON body"))?;

        // Then validate
        value.validate().map_err(|e| {
            let mut field_errors = HashMap::new();

            for (field, errors) in e.field_errors() {
                let messages: Vec<String> = errors
                    .iter()
                    .filter_map(|e| e.message.as_ref().map(|m| m.to_string()))
                    .collect();
                if !messages.is_empty() {
                    field_errors.insert(field.to_string(), messages);
                }
            }

            AppError::validation(field_errors)
        })?;

        Ok(ValidatedJson(value))
    }
}
"#
    .to_string()
}

fn generate_router_module(routes: &[RouteInfo]) -> Result<String> {
    let mut output = String::new();
    let has_actions = routes.iter().any(|r| r.has_action_input);
    let has_path_params = routes.iter().any(|r| {
        let route_path = extract_route_path(&r.props_path).unwrap_or_default();
        !extract_route_params(&route_path).is_empty()
    });

    output.push_str("// [Generated] Do not edit manually\n\n");

    // Build conditional imports
    let mut routing_imports = vec!["get"];
    if has_actions {
        routing_imports.push("post");
    }

    let mut extract_imports = vec!["Json"];
    if has_path_params {
        extract_imports.push("Path");
    }

    output.push_str(&format!(
        "use axum::{{Router, routing::{{{}}}, extract::{{{}}}}};\n",
        routing_imports.join(", "),
        extract_imports.join(", ")
    ));

    if has_actions {
        output.push_str("use serde::Deserialize;\n");
    }

    output.push_str("\nuse backend::routes;\n");

    // Only import ActionResult if there are routes with actions
    if has_actions {
        output.push_str("use backend::ActionResult;\n");
    }

    output.push_str("use super::error::AppError;\n");

    // Import ValidatedJson if there are actions
    if has_actions {
        output.push_str("use super::error::ValidatedJson;\n");
    }

    output.push_str("\n");

    // Generate route handlers
    for route in routes {
        let route_path = extract_route_path(&route.props_path)?;
        let handler_name = route_path_to_handler_name(&route_path);
        let module_path = route_path_to_module_path(&route_path);
        let params = extract_route_params(&route_path);

        if params.is_empty() {
            // No parameters - simple handler
            output.push_str(&format!("// Handler for: {}\n", route_path));
            output.push_str(&format!(
                "async fn {}() -> Result<Json<routes::{}::PageProps>, AppError> {{\n",
                handler_name, module_path
            ));
            output.push_str(&format!(
                "    let props = routes::{}::get_props().await;\n",
                module_path
            ));
            output.push_str("    Ok(Json(props))\n");
            output.push_str("}\n\n");
        } else {
            // Has parameters - need Path extractor
            // Use the actual *Path struct from the route module
            let path_struct_name = get_path_struct_name(&route_path);

            // Generate handler with Path extractor
            output.push_str(&format!("// Handler for: {}\n", route_path));
            output.push_str(&format!(
                "async fn {}(Path(path): Path<routes::{}::{}>) -> Result<Json<routes::{}::PageProps>, AppError> {{\n",
                handler_name, module_path, path_struct_name, module_path
            ));

            // Call get_props with path struct
            output.push_str(&format!(
                "    let props = routes::{}::get_props(path).await;\n",
                module_path
            ));
            output.push_str("    Ok(Json(props))\n");
            output.push_str("}\n\n");
        }

        // Generate POST handler if route has ActionInput
        if route.has_action_input {
            let action_handler_name = format!("{}_action", handler_name);
            let params = extract_route_params(&route_path);

            if params.is_empty() {
                // No path parameters
                output.push_str(&format!("// POST handler for: {}\n", route_path));
                output.push_str(&format!(
                    "async fn {}(ValidatedJson(input): ValidatedJson<routes::{}::ActionInput>) -> Result<Json<ActionResult<routes::{}::PageProps>>, AppError> {{\n",
                    action_handler_name, module_path, module_path
                ));
                output.push_str(&format!(
                    "    let result = routes::{}::post_action(input).await;\n",
                    module_path
                ));
                output.push_str("    Ok(Json(result))\n");
                output.push_str("}\n\n");
            } else {
                // Has path parameters
                let path_struct_name = get_path_struct_name(&route_path);
                output.push_str(&format!("// POST handler for: {}\n", route_path));
                output.push_str(&format!(
                    "async fn {}(Path(path): Path<routes::{}::{}>, ValidatedJson(input): ValidatedJson<routes::{}::ActionInput>) -> Result<Json<ActionResult<routes::{}::PageProps>>, AppError> {{\n",
                    action_handler_name, module_path, path_struct_name, module_path, module_path
                ));
                output.push_str(&format!(
                    "    let result = routes::{}::post_action(path, input).await;\n",
                    module_path
                ));
                output.push_str("    Ok(Json(result))\n");
                output.push_str("}\n\n");
            }
        }
    }

    // Generate create_router function
    output.push_str("pub fn create_router() -> Router {\n");
    output.push_str("    Router::new()\n");

    for route in routes {
        let route_path = extract_route_path(&route.props_path)?;
        let url_path = convert_to_url_path(&route_path);
        let handler_name = route_path_to_handler_name(&route_path);

        if route.has_action_input {
            // Route with both GET and POST
            let action_handler_name = format!("{}_action", handler_name);
            output.push_str(&format!(
                "        .route(\"{}\", get({}).post({}))\n",
                url_path, handler_name, action_handler_name
            ));
        } else {
            // GET only route
            output.push_str(&format!(
                "        .route(\"{}\", get({}))\n",
                url_path, handler_name
            ));
        }
    }

    output.push_str("}\n");

    Ok(output)
}

fn generate_main_module() -> String {
    r#"// [Generated] Do not edit manually

mod error;
mod router;

#[tokio::main]
async fn main() {
    let app = router::create_router();

    let port = std::env::var("RUST_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("127.0.0.1:{}", port);

    println!("Backend server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to port");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
"#
    .to_string()
}

fn generate_routes_mod(routes: &[RouteInfo]) -> Result<String> {
    let mut output = String::new();

    output.push_str("// [Generated] Do not edit manually\n");
    output.push_str("// This file is managed by the Forte CLI\n\n");

    // Collect all top-level modules
    let mut modules = std::collections::HashSet::new();
    for route in routes {
        let route_path = extract_route_path(&route.props_path)?;
        let first_segment = route_path.split('/').next().unwrap_or("index");
        modules.insert(first_segment.to_string());
    }

    // Generate pub mod declarations
    let mut sorted_modules: Vec<_> = modules.into_iter().collect();
    sorted_modules.sort();

    for module in sorted_modules {
        output.push_str(&format!("pub mod {};\n", module));
    }

    Ok(output)
}

/// Extract route path from props.rs full path
/// Example: /path/to/backend/src/routes/product/_id_/props.rs -> product/_id_
fn extract_route_path(props_path: &Path) -> Result<String> {
    let path_str = props_path.to_str().context("Invalid UTF-8 in path")?;

    // Find "routes/" and extract everything after it until "/props.rs"
    if let Some(routes_idx) = path_str.find("routes/") {
        let after_routes = &path_str[routes_idx + 7..]; // Skip "routes/"
        if let Some(props_idx) = after_routes.find("/props.rs") {
            return Ok(after_routes[..props_idx].to_string());
        }
    }

    anyhow::bail!("Could not extract route path from: {}", path_str)
}

/// Convert route path to handler function name
/// Example: product/_id_ -> handler_product_id
fn route_path_to_handler_name(route_path: &str) -> String {
    let normalized = route_path
        .replace('/', "_")
        .replace("_id_", "id")
        .replace("_userId_", "user_id")
        .replace("_postId_", "post_id");

    format!("handler_{}", normalized)
}

/// Convert route path to module path
/// Example: product/_id_ -> product::_id_
fn route_path_to_module_path(route_path: &str) -> String {
    route_path.replace('/', "::")
}

/// Extract route parameters from route path
/// Example: product/_id_/review/_reviewId_ -> vec!["id", "reviewId"]
fn extract_route_params(route_path: &str) -> Vec<String> {
    let mut params = Vec::new();

    for segment in route_path.split('/') {
        if segment.starts_with('_') && segment.ends_with('_') && segment.len() > 2 {
            let param_name = &segment[1..segment.len() - 1];
            params.push(param_name.to_string());
        }
    }

    params
}

/// Convert string to PascalCase
/// Example: handler_product_id -> HandlerProductId
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Get the expected *Path struct name for a route
/// Example: product/_id_ -> ProductIdPath
fn get_path_struct_name(route_path: &str) -> String {
    let normalized = route_path
        .replace('/', "_")
        .replace("_id_", "Id")
        .replace("_userId_", "UserId")
        .replace("_postId_", "PostId");

    format!("{}Path", to_pascal_case(&normalized))
}

/// Convert route path to URL path
/// Example: product/_id_ -> /product/:id
fn convert_to_url_path(route_path: &str) -> String {
    let mut url_path = String::from("/");

    for segment in route_path.split('/') {
        if segment.is_empty() {
            continue;
        }

        if segment == "index" {
            continue; // index becomes root path
        }

        // Convert _paramName_ to :paramName
        if segment.starts_with('_') && segment.ends_with('_') && segment.len() > 2 {
            let param_name = &segment[1..segment.len() - 1];
            url_path.push(':');
            url_path.push_str(param_name);
        } else {
            url_path.push_str(segment);
        }

        url_path.push('/');
    }

    // Remove trailing slash unless it's the root path
    if url_path.len() > 1 && url_path.ends_with('/') {
        url_path.pop();
    }

    url_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_conversion() {
        assert_eq!(convert_to_url_path("index"), "/");
        assert_eq!(convert_to_url_path("about"), "/about");
        assert_eq!(convert_to_url_path("product/_id_"), "/product/:id");
        assert_eq!(
            convert_to_url_path("user/_userId_/post/_postId_"),
            "/user/:userId/post/:postId"
        );
    }

    #[test]
    fn test_handler_name() {
        assert_eq!(route_path_to_handler_name("index"), "handler_index");
        assert_eq!(
            route_path_to_handler_name("product/_id_"),
            "handler_product_id"
        );
    }
}

fn generate_env_module(config: &ForteConfig) -> String {
    let mut output = String::new();

    output.push_str("// [Generated] Do not edit manually\n");
    output.push_str("// Environment variables with type safety\n\n");
    output.push_str("use std::sync::OnceLock;\n\n");

    // Generate Env struct
    output.push_str("pub struct Env {\n");

    // Add required fields
    for var in &config.env.required {
        let field_name = var.to_lowercase();
        output.push_str(&format!("    pub {}: String,\n", field_name));
    }

    // Add optional fields
    for var in &config.env.optional {
        let field_name = var.to_lowercase();
        output.push_str(&format!("    pub {}: Option<String>,\n", field_name));
    }

    output.push_str("}\n\n");

    // Generate load function
    output.push_str("impl Env {\n");
    output.push_str("    fn load() -> Self {\n");
    output.push_str("        Self {\n");

    // Load required vars
    for var in &config.env.required {
        let field_name = var.to_lowercase();
        if let Some(default) = config.env.defaults.get(var) {
            output.push_str(&format!(
                "            {}: std::env::var(\"{}\").unwrap_or_else(|_| \"{}\".to_string()),\n",
                field_name, var, default
            ));
        } else {
            output.push_str(&format!(
                "            {}: std::env::var(\"{}\").expect(\"Missing required env var: {}\"),\n",
                field_name, var, var
            ));
        }
    }

    // Load optional vars
    for var in &config.env.optional {
        let field_name = var.to_lowercase();
        if let Some(default) = config.env.defaults.get(var) {
            output.push_str(&format!(
                "            {}: Some(std::env::var(\"{}\").unwrap_or_else(|_| \"{}\".to_string())),\n",
                field_name, var, default
            ));
        } else {
            output.push_str(&format!(
                "            {}: std::env::var(\"{}\").ok(),\n",
                field_name, var
            ));
        }
    }

    output.push_str("        }\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Global static
    output.push_str("static ENV: OnceLock<Env> = OnceLock::new();\n\n");
    output.push_str("pub fn env() -> &'static Env {\n");
    output.push_str("    ENV.get_or_init(|| Env::load())\n");
    output.push_str("}\n");

    output
}
