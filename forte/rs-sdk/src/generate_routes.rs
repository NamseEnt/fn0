use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{env, fs, path::Path};

pub fn generate_routes() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let pages_dir = Path::new(&manifest_dir).join("src/pages");
    let api_dir = Path::new(&manifest_dir).join("src/api");
    let hooks_dir = Path::new(&manifest_dir).join("src/hooks");
    let actions_dir = Path::new(&manifest_dir).join("src/actions");
    let queue_task_dir = Path::new(&manifest_dir).join("src/queue_task");
    let output_path = Path::new(&manifest_dir).join("src/route_generated.rs");
    let fe_paths_output = Path::new(&manifest_dir).join("../fe/src/paths.generated.ts");

    println!("cargo:rerun-if-changed=src/pages");
    println!("cargo:rerun-if-changed=src/api");
    println!("cargo:rerun-if-changed=src/hooks");
    println!("cargo:rerun-if-changed=src/actions");
    println!("cargo:rerun-if-changed=src/queue_task");

    let mut pages = discover_pages(&pages_dir);
    pages.extend(discover_api(&api_dir));
    let hooks = discover_hooks(&hooks_dir);
    let actions = discover_actions(&actions_dir);
    let queue_tasks = discover_queue_tasks(&queue_task_dir);
    let tokens = generate_code(&pages, &hooks, &actions, &queue_tasks);

    let syntax_tree = syn::parse2::<syn::File>(tokens).expect("Failed to parse generated code");
    let formatted = prettyplease::unparse(&syntax_tree);

    // Only write if content changed to avoid triggering file watchers
    let current_content = fs::read_to_string(&output_path).unwrap_or_default();
    if current_content != formatted {
        fs::write(&output_path, formatted).unwrap();
    }

    let fe_paths_content = generate_fe_paths(&pages);
    let current_fe_paths = fs::read_to_string(&fe_paths_output).unwrap_or_default();
    if current_fe_paths != fe_paths_content {
        fs::write(&fe_paths_output, fe_paths_content).unwrap();
    }

    if !queue_tasks.is_empty() {
        let queue_task_mod_path = Path::new(&manifest_dir).join("src/queue_task/mod.rs");
        let queue_task_mod_content = generate_queue_task_mod(&queue_tasks);
        let current_qt_mod = fs::read_to_string(&queue_task_mod_path).unwrap_or_default();
        if current_qt_mod != queue_task_mod_content {
            fs::write(&queue_task_mod_path, queue_task_mod_content).unwrap();
        }
    }
}

#[derive(Debug)]
struct PageInfo {
    module_name: String,
    module_path: String,
    route_path: String,
    route_segments: Vec<RouteSegment>,
    path_params: Option<Vec<PathParamField>>,
    search_params: Option<Vec<SearchParamField>>,
    is_redirect_only: bool,
    is_api: bool,
}

#[derive(Debug, Clone)]
enum RouteSegment {
    Static(String),
    Dynamic(String), // [id] -> "id"
}

#[derive(Debug)]
struct SearchParamField {
    name: String,
    is_optional: bool,
    inner_type: String,
}

#[derive(Debug)]
struct PathParamField {
    name: String,
    inner_type: String,
}

#[derive(Debug)]
struct HookInfo {
    name: String,
    module_name: String,
    module_path: String,
}

#[derive(Debug)]
struct ActionInfo {
    name: String,
    module_name: String,
    module_path: String,
}

#[derive(Debug)]
struct QueueTaskInfo {
    name: String,
}

fn discover_hooks(hooks_dir: &Path) -> Vec<HookInfo> {
    let mut hooks = Vec::new();

    if !hooks_dir.exists() {
        return hooks;
    }

    let Ok(entries) = fs::read_dir(hooks_dir) else {
        return hooks;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() && path.extension().map(|e| e == "rs").unwrap_or(false) {
            let file_name = path.file_stem().unwrap().to_string_lossy().to_string();

            if file_name == "mod" {
                continue;
            }

            let Some(content) = fs::read_to_string(&path).ok() else {
                continue;
            };

            if has_hook_handler(&content) {
                let module_name = format!("hooks_{}", file_name);
                let module_path = format!("hooks/{}.rs", file_name);
                hooks.push(HookInfo {
                    name: file_name,
                    module_name,
                    module_path,
                });
            }
        }
    }

    hooks
}

fn has_hook_handler(content: &str) -> bool {
    let Ok(syntax_tree) = syn::parse_file(content) else {
        return false;
    };

    let mut has_input = false;
    let mut has_output = false;
    let mut has_handler = false;

    for item in syntax_tree.items {
        match item {
            syn::Item::Struct(item_struct) => {
                if item_struct.ident == "Input" {
                    has_input = true;
                } else if item_struct.ident == "Output" {
                    has_output = true;
                }
            }
            syn::Item::Enum(item_enum) => {
                if item_enum.ident == "Output" {
                    has_output = true;
                }
            }
            syn::Item::Fn(func) => {
                let is_pub = matches!(func.vis, syn::Visibility::Public(_));
                let is_handler_fn = func.sig.ident == "handler";

                if is_pub && is_handler_fn {
                    has_handler = true;
                }
            }
            _ => {}
        }
    }

    has_input && has_output && has_handler
}

fn discover_actions(actions_dir: &Path) -> Vec<ActionInfo> {
    let mut actions = Vec::new();

    if !actions_dir.exists() {
        return actions;
    }

    let Ok(entries) = fs::read_dir(actions_dir) else {
        return actions;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() && path.extension().map(|e| e == "rs").unwrap_or(false) {
            let file_name = path.file_stem().unwrap().to_string_lossy().to_string();

            if file_name == "mod" {
                continue;
            }

            let Some(content) = fs::read_to_string(&path).ok() else {
                continue;
            };

            if has_action_handler(&content) {
                let module_name = format!("actions_{}", file_name);
                let module_path = format!("actions/{}.rs", file_name);
                actions.push(ActionInfo {
                    name: file_name,
                    module_name,
                    module_path,
                });
            }
        }
    }

    actions
}

fn has_action_handler(content: &str) -> bool {
    let Ok(syntax_tree) = syn::parse_file(content) else {
        return false;
    };

    let mut has_input = false;
    let mut has_output = false;
    let mut has_handler = false;

    for item in syntax_tree.items {
        match item {
            syn::Item::Struct(item_struct) => {
                if item_struct.ident == "Input" {
                    has_input = true;
                } else if item_struct.ident == "Output" {
                    has_output = true;
                }
            }
            syn::Item::Enum(item_enum) => {
                if item_enum.ident == "Output" {
                    has_output = true;
                }
            }
            syn::Item::Fn(func) => {
                let is_pub = matches!(func.vis, syn::Visibility::Public(_));
                let is_async = func.sig.asyncness.is_some();
                let is_handler_fn = func.sig.ident == "handler";

                if is_pub && is_async && is_handler_fn {
                    has_handler = true;
                }
            }
            _ => {}
        }
    }

    has_input && has_output && has_handler
}

fn discover_queue_tasks(queue_task_dir: &Path) -> Vec<QueueTaskInfo> {
    let mut queue_tasks = Vec::new();

    if !queue_task_dir.exists() {
        return queue_tasks;
    }

    let Ok(entries) = fs::read_dir(queue_task_dir) else {
        return queue_tasks;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() && path.extension().map(|e| e == "rs").unwrap_or(false) {
            let file_name = path.file_stem().unwrap().to_string_lossy().to_string();

            if file_name == "mod" {
                continue;
            }

            let Some(content) = fs::read_to_string(&path).ok() else {
                continue;
            };

            if has_queue_task_handler(&content) {
                queue_tasks.push(QueueTaskInfo { name: file_name });
            }
        }
    }

    queue_tasks
}

fn has_queue_task_handler(content: &str) -> bool {
    let Ok(syntax_tree) = syn::parse_file(content) else {
        return false;
    };

    let mut has_input = false;
    let mut has_handle = false;

    for item in syntax_tree.items {
        match item {
            syn::Item::Struct(item_struct) => {
                if item_struct.ident == "Input" {
                    has_input = true;
                }
            }
            syn::Item::Fn(func) => {
                let is_pub = matches!(func.vis, syn::Visibility::Public(_));
                let is_async = func.sig.asyncness.is_some();
                let is_handle_fn = func.sig.ident == "handle";

                if is_pub && is_async && is_handle_fn {
                    has_handle = true;
                }
            }
            _ => {}
        }
    }

    has_input && has_handle
}

enum HandlerType {
    None,
    Props,
    Redirect,
}

fn get_handler_type(content: &str) -> HandlerType {
    let Ok(syntax_tree) = syn::parse_file(content) else {
        return HandlerType::None;
    };

    for item in syntax_tree.items {
        if let syn::Item::Fn(func) = item {
            // Check: pub async fn handler
            let is_pub = matches!(func.vis, syn::Visibility::Public(_));
            let is_async = func.sig.asyncness.is_some();
            let is_handler = func.sig.ident == "handler";

            if is_pub
                && is_async
                && is_handler
                && let syn::ReturnType::Type(_, ty) = &func.sig.output
            {
                let type_str = quote!(#ty).to_string();
                if type_str.contains("Result") && type_str.contains("Props") {
                    if is_props_redirect(content) {
                        return HandlerType::Redirect;
                    }
                    return HandlerType::Props;
                }
                if type_str.contains("Result") && type_str.contains("Redirect") {
                    return HandlerType::Redirect;
                }
            }
        }
    }

    HandlerType::None
}

fn is_props_redirect(content: &str) -> bool {
    let Ok(syntax_tree) = syn::parse_file(content) else {
        return false;
    };

    for item in syntax_tree.items {
        if let syn::Item::Type(type_alias) = item
            && type_alias.ident == "Props"
        {
            let type_str = quote!(#type_alias.ty).to_string();
            return type_str.contains("Redirect");
        }
    }

    false
}

fn parse_search_params(content: &str) -> Option<Vec<SearchParamField>> {
    let syntax_tree = syn::parse_file(content).ok()?;

    for item in syntax_tree.items {
        if let syn::Item::Struct(item_struct) = item
            && item_struct.ident == "SearchParams"
        {
            let mut fields = Vec::new();

            if let syn::Fields::Named(named_fields) = item_struct.fields {
                for field in named_fields.named {
                    let name = field.ident?.to_string();
                    let (is_optional, inner_type) = extract_type_info(&field.ty);
                    fields.push(SearchParamField {
                        name,
                        is_optional,
                        inner_type,
                    });
                }
            }

            return Some(fields);
        }
    }

    None
}

fn parse_path_params(content: &str) -> Option<Vec<PathParamField>> {
    let syntax_tree = syn::parse_file(content).ok()?;

    for item in syntax_tree.items {
        if let syn::Item::Struct(item_struct) = item
            && item_struct.ident == "PathParams"
        {
            let mut fields = Vec::new();

            if let syn::Fields::Named(named_fields) = item_struct.fields {
                for field in named_fields.named {
                    let name = field.ident?.to_string();
                    let (_is_optional, inner_type) = extract_type_info(&field.ty);
                    fields.push(PathParamField { name, inner_type });
                }
            }

            return Some(fields);
        }
    }

    None
}

fn extract_type_info(ty: &syn::Type) -> (bool, String) {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        if segment.ident == "Option"
            && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
            && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
        {
            return (true, quote!(#inner_ty).to_string());
        }
        return (false, quote!(#ty).to_string());
    }
    (false, quote!(#ty).to_string())
}

fn discover_pages(pages_dir: &Path) -> Vec<PageInfo> {
    let mut pages = Vec::new();
    if !pages_dir.exists() {
        return pages;
    }
    discover_endpoints_recursive(pages_dir, pages_dir, &mut pages, "pages", &[], false);
    pages
}

fn discover_api(api_dir: &Path) -> Vec<PageInfo> {
    let mut endpoints = Vec::new();
    if !api_dir.exists() {
        return endpoints;
    }
    discover_endpoints_recursive(
        api_dir,
        api_dir,
        &mut endpoints,
        "api",
        &["api".to_string()],
        true,
    );
    endpoints
}

fn discover_endpoints_recursive(
    base_dir: &Path,
    current_dir: &Path,
    pages: &mut Vec<PageInfo>,
    module_prefix: &str,
    route_prefix: &[String],
    is_api: bool,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            discover_endpoints_recursive(
                base_dir,
                &path,
                pages,
                module_prefix,
                route_prefix,
                is_api,
            );
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            let Some(content) = fs::read_to_string(&path).ok() else {
                continue;
            };

            let handler_type = get_handler_type(&content);
            let is_redirect_only = match handler_type {
                HandlerType::None => continue,
                HandlerType::Props => false,
                HandlerType::Redirect => true,
            };

            let relative_path = path.strip_prefix(base_dir).unwrap();
            let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
            let parent_segments: Vec<_> = relative_path
                .parent()
                .map(|p| {
                    p.components()
                        .map(|c| c.as_os_str().to_string_lossy().to_string())
                        .collect()
                })
                .unwrap_or_default();

            let mut route_segments: Vec<String> = if file_name == "index" || file_name == "mod" {
                parent_segments.clone()
            } else {
                let mut segments = parent_segments.clone();
                segments.push(file_name.clone());
                segments
            };

            if route_segments.last() == Some(&"index".to_string()) {
                route_segments.pop();
            }

            let mut full_route_segments = route_prefix.to_vec();
            full_route_segments.extend(route_segments);

            let module_name = if full_route_segments.is_empty() {
                format!("{}_index", module_prefix)
            } else {
                format!(
                    "{}_{}",
                    module_prefix,
                    full_route_segments
                        .iter()
                        .skip(route_prefix.len())
                        .map(|s| {
                            if s.starts_with('[') && s.ends_with(']') {
                                format!("_{}_", &s[1..s.len() - 1])
                            } else {
                                s.clone()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("_")
                )
            };

            let module_path = format!("{}/{}", module_prefix, relative_path.to_string_lossy());

            let route_path = if full_route_segments.is_empty() {
                "/".to_string()
            } else {
                format!("/{}", full_route_segments.join("/"))
            };

            let parsed_route_segments: Vec<RouteSegment> = full_route_segments
                .iter()
                .map(|s| {
                    if s.starts_with('[') && s.ends_with(']') {
                        RouteSegment::Dynamic(s[1..s.len() - 1].to_string())
                    } else {
                        RouteSegment::Static(s.clone())
                    }
                })
                .collect();

            let search_params = parse_search_params(&content);
            let path_params = parse_path_params(&content);

            pages.push(PageInfo {
                module_name,
                module_path,
                route_path,
                route_segments: parsed_route_segments,
                path_params,
                search_params,
                is_redirect_only,
                is_api,
            });
        }
    }
}

fn generate_code(
    pages: &[PageInfo],
    hooks: &[HookInfo],
    actions: &[ActionInfo],
    queue_tasks: &[QueueTaskInfo],
) -> TokenStream {
    let module_declarations = generate_module_declarations(pages);
    let hook_module_declarations = generate_hook_module_declarations(hooks);
    let action_module_declarations = generate_action_module_declarations(actions);
    let route_matches = generate_route_matches(pages);
    let redirect_enum = generate_redirect_enum(pages);
    let hook_handler = generate_hook_handler(hooks);
    let action_handler = generate_action_handler(actions);
    let enqueue_module = generate_enqueue_module(queue_tasks);
    let queue_task_execute_handler = generate_queue_task_execute_handler(queue_tasks);

    let route_chain = if route_matches.is_empty() {
        quote! {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap())
        }
    } else {
        let needs_path_segments = pages
            .iter()
            .any(|p| has_dynamic_segments(&p.route_segments));
        let path_segments_decl = if needs_path_segments {
            quote! {
                let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
            }
        } else {
            quote! {}
        };
        let first = &route_matches[0];
        let rest = &route_matches[1..];
        quote! {
            #path_segments_decl
            #first
            #(else #rest)*
            else {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap())
            }
        }
    };

    quote! {
        // Auto-generated by build.rs

        #(#module_declarations)*
        #(#hook_module_declarations)*
        #(#action_module_declarations)*

        use forte_sdk::anyhow::Result;
        use forte_sdk::http::{Error, Request, Response, StatusCode, body::Body, HeaderMap};
        use forte_sdk::http_header::{COOKIE, LOCATION, SET_COOKIE};
        use forte_sdk::*;

        #redirect_enum

        #enqueue_module

        #[forte_sdk::wstd::http_server]
        pub async fn main(request: Request<Body>) -> Result<Response<Body>, Error> {
            let (parts, mut body) = request.into_parts();
            let headers = parts.headers;
            let path = parts.uri.path();
            let method = parts.method;
            let body_bytes = body.contents().await?.to_vec();
            let mut cookie_jar = make_cookie_jar(&headers);
            let db = forte_sdk::forte_db::default_db();

            let Some(uri_authority) = parts.uri.authority() else {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from("Missing authority in request URI"))
                    .unwrap());
            };
            let uri_authority = uri_authority.as_str();

            if let Some(hook_name) = path.strip_prefix("/__forte_hook/") {
                return handle_hook(hook_name, uri_authority, &method, &headers, &mut cookie_jar, &body_bytes, &db).await;
            }

            if let Some(action_name) = path.strip_prefix("/__forte_action/") {
                return handle_action(action_name, uri_authority, &method, &headers, &mut cookie_jar, &body_bytes, &db).await;
            }

            if path == "/__forte_queue_task/execute" {
                return handle_queue_task_execute(&body_bytes).await;
            }

            #route_chain
        }

        #hook_handler

        #action_handler

        #queue_task_execute_handler

        fn make_cookie_jar(headers: &HeaderMap) -> cookie::CookieJar {
            let mut jar = cookie::CookieJar::new();
            let Some(cookie) = headers.get(COOKIE) else {
                return jar;
            };
            let Ok(cookie_str) = cookie.to_str() else {
                return jar;
            };

            for cookie in cookie::Cookie::split_parse(cookie_str) {
                let Ok(cookie) = cookie else { continue };
                jar.add_original(cookie.into_owned());
            }

            jar
        }

        fn build_response_with_cookies(mut response: Response<Body>, cookie_jar: &cookie::CookieJar) -> Response<Body> {
            for cookie in cookie_jar.delta() {
                if let Ok(value) = cookie.encoded().to_string().parse() {
                    response.headers_mut().append(SET_COOKIE, value);
                }
            }
            response
        }
    }
}

fn generate_module_declarations(pages: &[PageInfo]) -> Vec<TokenStream> {
    pages
        .iter()
        .map(|page| {
            let module_name = format_ident!("{}", page.module_name);
            let module_path = &page.module_path;

            let allow_attr = if has_dynamic_segments(&page.route_segments) {
                quote! { #[allow(non_snake_case)] }
            } else {
                quote! {}
            };

            quote! {
                #allow_attr
                #[path = #module_path]
                mod #module_name;
            }
        })
        .collect()
}

fn has_dynamic_segments(segments: &[RouteSegment]) -> bool {
    segments
        .iter()
        .any(|s| matches!(s, RouteSegment::Dynamic(_)))
}

fn generate_hook_module_declarations(hooks: &[HookInfo]) -> Vec<TokenStream> {
    hooks
        .iter()
        .map(|hook| {
            let module_name = format_ident!("{}", hook.module_name);
            let module_path = &hook.module_path;

            quote! {
                #[path = #module_path]
                mod #module_name;
            }
        })
        .collect()
}

fn generate_route_matches(pages: &[PageInfo]) -> Vec<TokenStream> {
    pages
        .iter()
        .map(|page| {
            let module_name = format_ident!("{}", page.module_name);

            // Generate route condition
            let route_condition = if has_dynamic_segments(&page.route_segments) {
                generate_dynamic_route_condition(&page.route_segments)
            } else {
                let route_path = &page.route_path;
                quote! { path == #route_path }
            };


            // Generate path params extraction (if dynamic)
            let path_params_extraction = if let Some(path_params) = &page.path_params {
                generate_path_params_extraction(&module_name, &page.route_segments, path_params)
            } else {
                quote! {}
            };

            let search_params_extraction = if let Some(fields) = &page.search_params {
                let field_parsers = generate_search_field_parsers(fields);
                let field_names: Vec<_> =
                    fields.iter().map(|f| format_ident!("{}", f.name)).collect();
                quote! {
                    use std::collections::HashMap;
                    let query = parts.uri.query().unwrap_or("");
                    let query_params: HashMap<String, String> = forte_sdk::form_urlencoded::parse(query.as_bytes())
                        .map(|(k, v)| (k.into_owned(), v.into_owned()))
                        .collect();
                    #(#field_parsers)*
                    let search_params = #module_name::SearchParams {
                        #(#field_names),*
                    };
                }
            } else {
                quote! {}
            };

            // Generate handler call based on what params exist
            let handler_call = match (&page.path_params, &page.search_params) {
                (Some(_), Some(_)) => quote! {
                    #module_name::handler(req, path_params, search_params).await
                },
                (Some(_), None) => quote! {
                    #module_name::handler(req, path_params).await
                },
                (None, Some(_)) => quote! {
                    #module_name::handler(req, search_params).await
                },
                (None, None) => quote! {
                    #module_name::handler(req).await
                },
            };

            // Different response handling for redirect-only pages vs props pages
            let response_handling = if page.is_redirect_only {
                quote! {
                    match #handler_call {
                        Ok(redirect) => {
                            Ok(build_response_with_cookies(
                                Response::builder()
                                    .status(StatusCode::FOUND)
                                    .header(LOCATION, redirect.to_path())
                                    .body(Body::empty())
                                    .unwrap(),
                                &cookie_jar,
                            ))
                        }
                        Err(e) => {
                            eprintln!("Error at {}: {:?}", path, e);
                            Ok(Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from("Internal Server Error"))
                                .unwrap())
                        }
                    }
                }
            } else {
                let ok_response = if page.is_api {
                    quote! {
                        Response::builder()
                            .status(StatusCode::OK)
                            .header("content-type", "application/json")
                            .body(Body::from_stream(stream))
                            .unwrap()
                    }
                } else {
                    quote! {
                        Response::new(Body::from_stream(stream))
                    }
                };
                quote! {
                    match #handler_call {
                        Ok(props) => {
                            let stream = forte_json::to_stream(&props);
                            Ok(build_response_with_cookies(#ok_response, &cookie_jar))
                        }
                        Err(e) => {
                            if let Some(redirect) = e.downcast_ref::<Redirect>() {
                                Ok(build_response_with_cookies(
                                    Response::builder()
                                        .status(StatusCode::FOUND)
                                        .header(LOCATION, redirect.to_path())
                                        .body(Body::empty())
                                        .unwrap(),
                                    &cookie_jar,
                                ))
                            } else {
                                eprintln!("Error at {}: {:?}", path, e);
                                Ok(Response::builder()
                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                    .body(Body::from("Internal Server Error"))
                                    .unwrap())
                            }
                        }
                    }
                }
            };

            quote! {
                if #route_condition {
                    #path_params_extraction
                    #search_params_extraction

                    let req = ForteRequest {
                        uri_authority,
                        method: &method,
                        headers: &headers,
                        jar: &mut cookie_jar,
                        raw_body: &body_bytes,
                        body: (),
                        db: &db,
                    };

                    #response_handling
                }
            }
        })
        .collect()
}

fn generate_dynamic_route_condition(segments: &[RouteSegment]) -> TokenStream {
    let segment_count = segments.len();
    let segment_checks: Vec<TokenStream> = segments
        .iter()
        .enumerate()
        .filter_map(|(i, seg)| {
            if let RouteSegment::Static(s) = seg {
                if i == 0 {
                    Some(quote! { path_segments.first() == Some(&#s) })
                } else {
                    Some(quote! { path_segments.get(#i) == Some(&#s) })
                }
            } else {
                None // Dynamic segments match anything
            }
        })
        .collect();

    if segment_checks.is_empty() {
        quote! {
            path_segments.len() == #segment_count
        }
    } else {
        quote! {
            path_segments.len() == #segment_count && #(#segment_checks)&&*
        }
    }
}

fn generate_path_params_extraction(
    module_name: &syn::Ident,
    route_segments: &[RouteSegment],
    path_params: &[PathParamField],
) -> TokenStream {
    let extractions: Vec<TokenStream> = route_segments
        .iter()
        .enumerate()
        .filter_map(|(i, seg)| {
            if let RouteSegment::Dynamic(param_name) = seg {
                // Find the corresponding PathParamField
                let field = path_params.iter().find(|f| &f.name == param_name)?;
                let field_ident = format_ident!("{}", field.name);
                let inner_type: TokenStream = field.inner_type.parse().unwrap();

                if field.inner_type == "String" {
                    Some(quote! {
                        let #field_ident: String = path_segments[#i].to_string();
                    })
                } else {
                    Some(quote! {
                        let #field_ident: #inner_type = match path_segments[#i].parse::<#inner_type>() {
                            Ok(v) => v,
                            Err(_) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!("Invalid path parameter: {}", stringify!(#field_ident))))
                                    .unwrap());
                            }
                        };
                    })
                }
            } else {
                None
            }
        })
        .collect();

    let field_names: Vec<_> = path_params
        .iter()
        .map(|f| format_ident!("{}", f.name))
        .collect();

    quote! {
        #(#extractions)*
        let path_params = #module_name::PathParams { #(#field_names),* };
    }
}

fn generate_search_field_parsers(fields: &[SearchParamField]) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let field_name = format_ident!("{}", field.name);
            let field_name_str = &field.name;
            let inner_type: TokenStream = field.inner_type.parse().unwrap();

            if field.is_optional {
                if field.inner_type == "String" {
                    quote! {
                        let #field_name: Option<String> = query_params.get(#field_name_str).cloned();
                    }
                } else {
                    quote! {
                        let #field_name: Option<#inner_type> = query_params
                            .get(#field_name_str)
                            .and_then(|v| v.parse::<#inner_type>().ok());
                    }
                }
            } else if field.inner_type == "String" {
                quote! {
                    let Some(#field_name) = query_params.get(#field_name_str).cloned() else {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!("Missing required query parameter: {}", #field_name_str)))
                            .unwrap());
                    };
                }
            } else {
                quote! {
                    let #field_name: #inner_type = match query_params.get(#field_name_str) {
                        Some(v) => match v.parse::<#inner_type>() {
                            Ok(parsed) => parsed,
                            Err(_) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!("Invalid value for query parameter: {}", #field_name_str)))
                                    .unwrap());
                            }
                        },
                        None => {
                            return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Missing required query parameter: {}", #field_name_str)))
                                .unwrap());
                        }
                    };
                }
            }
        })
        .collect()
}

fn generate_redirect_enum(pages: &[PageInfo]) -> TokenStream {
    let variants: Vec<TokenStream> = pages
        .iter()
        .map(|page| {
            let variant_name = page_to_variant_name(&page.route_segments);
            let variant_ident = format_ident!("{}", variant_name);

            if let Some(path_params) = &page.path_params {
                let fields: Vec<TokenStream> = path_params
                    .iter()
                    .map(|p| {
                        let name = format_ident!("{}", p.name);
                        let ty: TokenStream = p.inner_type.parse().unwrap();
                        quote! { #name: #ty }
                    })
                    .collect();
                quote! { #variant_ident { #(#fields),* } }
            } else {
                quote! { #variant_ident }
            }
        })
        .collect();

    let to_path_arms: Vec<TokenStream> = pages
        .iter()
        .map(|page| {
            let variant_name = page_to_variant_name(&page.route_segments);
            let variant_ident = format_ident!("{}", variant_name);

            if let Some(path_params) = &page.path_params {
                let field_names: Vec<_> = path_params
                    .iter()
                    .map(|p| format_ident!("{}", p.name))
                    .collect();

                // Build path with dynamic segments
                let path_parts: Vec<TokenStream> = page
                    .route_segments
                    .iter()
                    .map(|seg| match seg {
                        RouteSegment::Static(s) => quote! { #s.to_string() },
                        RouteSegment::Dynamic(name) => {
                            let name_ident = format_ident!("{}", name);
                            quote! { #name_ident.to_string() }
                        }
                    })
                    .collect();

                quote! {
                    Redirect::#variant_ident { #(#field_names),* } => {
                        format!("/{}", [#(#path_parts),*].join("/"))
                    }
                }
            } else {
                let route_path = &page.route_path;
                quote! {
                    Redirect::#variant_ident => #route_path.to_string()
                }
            }
        })
        .collect();

    quote! {
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        #[allow(non_camel_case_types)]
        pub enum Redirect {
            External { url: String },
            #(#variants),*
        }

        impl Redirect {
            pub fn to_path(&self) -> String {
                match self {
                    Redirect::External { url } => url.clone(),
                    #(#to_path_arms),*
                }
            }
        }

        impl std::fmt::Display for Redirect {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Redirect to {}", self.to_path())
            }
        }

        impl std::error::Error for Redirect {}
    }
}

fn page_to_variant_name(segments: &[RouteSegment]) -> String {
    // /post -> Post
    // /post/id (static) -> PostId
    // /post/[id] -> Post_id_
    // /post/[id]/comment -> Post_id_Comment
    if segments.is_empty() {
        return "Index".to_string();
    }

    segments
        .iter()
        .map(|seg| match seg {
            RouteSegment::Static(s) => {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            }
            RouteSegment::Dynamic(name) => {
                format!("_{}_", name)
            }
        })
        .collect()
}

fn generate_hook_handler(hooks: &[HookInfo]) -> TokenStream {
    if hooks.is_empty() {
        return quote! {
            async fn handle_hook(
                hook_name: &str,
                _uri_authority: &str,
                _method: &http::Method,
                _headers: &HeaderMap,
                _cookie_jar: &mut cookie::CookieJar,
                _body_bytes: &[u8],
                _db: &forte_sdk::forte_db::Database,
            ) -> Result<Response<Body>, Error> {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from(format!("Hook '{}' not found", hook_name)))
                    .unwrap())
            }
        };
    }

    let hook_matches: Vec<TokenStream> = hooks
        .iter()
        .map(|hook| {
            let name = &hook.name;
            let module_name = format_ident!("{}", hook.module_name);

            quote! {
                #name => {
                    let input: #module_name::Input = forte_json::from_slice(body_bytes)
                        .map_err(|e| Error::msg(e.to_string()))?;
                    let req = ForteRequest {
                        uri_authority,
                        method,
                        headers,
                        jar: cookie_jar,
                        raw_body: body_bytes,
                        body: input,
                        db,
                    };
                    let output = #module_name::handler(req);
                    let json = forte_json::to_vec(&output);
                    Ok(build_response_with_cookies(
                        Response::builder()
                            .status(StatusCode::OK)
                            .header("content-type", "application/json")
                            .body(Body::from(json))
                            .unwrap(),
                        cookie_jar,
                    ))
                }
            }
        })
        .collect();

    quote! {
        async fn handle_hook(
            hook_name: &str,
            uri_authority: &str,
            method: &http::Method,
            headers: &HeaderMap,
            cookie_jar: &mut cookie::CookieJar,
            body_bytes: &[u8],
            db: &forte_sdk::forte_db::Database,
        ) -> Result<Response<Body>, Error> {
            match hook_name {
                #(#hook_matches)*
                _ => Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from(format!("Hook '{}' not found", hook_name)))
                    .unwrap()),
            }
        }
    }
}

fn generate_action_module_declarations(actions: &[ActionInfo]) -> Vec<TokenStream> {
    actions
        .iter()
        .map(|action| {
            let module_name = format_ident!("{}", action.module_name);
            let module_path = &action.module_path;

            quote! {
                #[path = #module_path]
                mod #module_name;
            }
        })
        .collect()
}

fn generate_action_handler(actions: &[ActionInfo]) -> TokenStream {
    if actions.is_empty() {
        return quote! {
            async fn handle_action(
                action_name: &str,
                _uri_authority: &str,
                _method: &http::Method,
                _headers: &HeaderMap,
                _cookie_jar: &mut cookie::CookieJar,
                _body_bytes: &[u8],
                _db: &forte_sdk::forte_db::Database,
            ) -> Result<Response<Body>, Error> {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from(format!("Action '{}' not found", action_name)))
                    .unwrap())
            }
        };
    }

    let action_matches: Vec<TokenStream> = actions
        .iter()
        .map(|action| {
            let name = &action.name;
            let module_name = format_ident!("{}", action.module_name);

            quote! {
                #name => {
                    let input: #module_name::Input = forte_json::from_slice(body_bytes)
                        .map_err(|e| Error::msg(e.to_string()))?;
                    let req = ForteRequest {
                        uri_authority,
                        method,
                        headers,
                        jar: cookie_jar,
                        raw_body: body_bytes,
                        body: input,
                        db,
                    };
                    let output = #module_name::handler(req).await;
                    let json = forte_json::to_vec(&output);
                    Ok(build_response_with_cookies(
                        Response::builder()
                            .status(StatusCode::OK)
                            .header("content-type", "application/json")
                            .body(Body::from(json))
                            .unwrap(),
                        cookie_jar,
                    ))
                }
            }
        })
        .collect();

    quote! {
        async fn handle_action(
            action_name: &str,
            uri_authority: &str,
            method: &http::Method,
            headers: &HeaderMap,
            cookie_jar: &mut cookie::CookieJar,
            body_bytes: &[u8],
            db: &forte_sdk::forte_db::Database,
        ) -> Result<Response<Body>, Error> {
            match action_name {
                #(#action_matches)*
                _ => Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from(format!("Action '{}' not found", action_name)))
                    .unwrap()),
            }
        }
    }
}

fn generate_enqueue_module(queue_tasks: &[QueueTaskInfo]) -> TokenStream {
    if queue_tasks.is_empty() {
        return quote! {};
    }

    let enqueue_fns: Vec<TokenStream> = queue_tasks
        .iter()
        .map(|qt| {
            let fn_name = format_ident!("{}", qt.name);
            let task_name_str = &qt.name;
            let module_path: Vec<_> = vec![format_ident!("queue_task"), format_ident!("{}", qt.name)];

            quote! {
                pub async fn #fn_name(input: crate::#(#module_path)::*::Input) -> forte_sdk::anyhow::Result<()> {
                    let payload = forte_sdk::serde_json::to_string(&input)?;
                    let id = forte_sdk::Uuid::now_v7().to_string();
                    let now = forte_sdk::now().to_rfc3339();
                    forte_sdk::forte_db::turso()
                        .execute_raw(
                            "INSERT INTO __forte_queue (id, task_name, payload, status, retry_count, max_retries, created_at, updated_at) VALUES (?, ?, ?, 'pending', 0, 3, ?, ?)",
                            vec![
                                forte_sdk::forte_db::text_value(&id),
                                forte_sdk::forte_db::text_value(#task_name_str),
                                forte_sdk::forte_db::text_value(&payload),
                                forte_sdk::forte_db::text_value(&now),
                                forte_sdk::forte_db::text_value(&now),
                            ],
                            false,
                        )
                        .await?;
                    Ok(())
                }
            }
        })
        .collect();

    quote! {
        pub mod enqueue {
            #(#enqueue_fns)*
        }
    }
}

fn generate_queue_task_execute_handler(queue_tasks: &[QueueTaskInfo]) -> TokenStream {
    if queue_tasks.is_empty() {
        return quote! {
            async fn handle_queue_task_execute(
                _body_bytes: &[u8],
            ) -> Result<Response<Body>, Error> {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("No queue tasks defined"))
                    .unwrap())
            }
        };
    }

    let task_matches: Vec<TokenStream> = queue_tasks
        .iter()
        .map(|qt| {
            let name = &qt.name;
            let module_path: Vec<_> = vec![format_ident!("queue_task"), format_ident!("{}", qt.name)];

            quote! {
                #name => {
                    let input: crate::#(#module_path)::*::Input = forte_sdk::serde_json::from_str(payload)
                        .map_err(|e| Error::msg(e.to_string()))?;
                    crate::#(#module_path)::*::handle(input).await
                }
            }
        })
        .collect();

    quote! {
        async fn handle_queue_task_execute(
            body_bytes: &[u8],
        ) -> Result<Response<Body>, Error> {
            let request: forte_sdk::serde_json::Value = forte_sdk::serde_json::from_slice(body_bytes)
                .map_err(|e| Error::msg(e.to_string()))?;
            let task_name = request["task_name"]
                .as_str()
                .ok_or_else(|| Error::msg("missing task_name"))?;
            let payload = request["payload"]
                .as_str()
                .ok_or_else(|| Error::msg("missing payload"))?;

            let result = match task_name {
                #(#task_matches)*
                _ => {
                    return Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::from(format!("Queue task '{}' not found", task_name)))
                        .unwrap());
                }
            };

            match result {
                Ok(()) => Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::empty())
                    .unwrap()),
                Err(e) => Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(format!("{:?}", e)))
                    .unwrap()),
            }
        }
    }
}

fn generate_queue_task_mod(queue_tasks: &[QueueTaskInfo]) -> String {
    let mut content = String::new();
    content.push_str("// Auto-generated by forte build\n\n");
    for qt in queue_tasks {
        content.push_str(&format!("pub mod {};\n", qt.name));
    }
    content
}

fn generate_fe_paths(pages: &[PageInfo]) -> String {
    let mut content = String::new();
    content.push_str("// Auto-generated by forte build\n\n");
    content.push_str("export const paths = {\n");

    for page in pages {
        // Convert [id] to :id format for the key
        let route_path = page
            .route_segments
            .iter()
            .map(|seg| match seg {
                RouteSegment::Static(s) => s.clone(),
                RouteSegment::Dynamic(name) => format!(":{}", name),
            })
            .collect::<Vec<_>>();
        let route_path = if route_path.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", route_path.join("/"))
        };

        if let Some(path_params) = &page.path_params {
            // Has dynamic params: "/post/:id": ({id}: {id: string}) => `/post/${id}`
            let param_names: Vec<&str> = path_params.iter().map(|p| p.name.as_str()).collect();
            let param_types: Vec<String> = path_params
                .iter()
                .map(|p| format!("{}: {}", p.name, rust_type_to_ts(&p.inner_type)))
                .collect();

            let path_template = page
                .route_segments
                .iter()
                .map(|seg| match seg {
                    RouteSegment::Static(s) => s.clone(),
                    RouteSegment::Dynamic(name) => format!("${{{}}}", name),
                })
                .collect::<Vec<_>>()
                .join("/");

            content.push_str(&format!(
                "  \"{}\": ({{{}}}: {{{}}}) => `/{}`",
                route_path,
                param_names.join(", "),
                param_types.join("; "),
                path_template
            ));
        } else {
            // No params: "/": () => "/"
            content.push_str(&format!("  \"{}\": () => \"{}\"", route_path, route_path));
        }

        content.push_str(",\n");
    }

    content.push_str("} as const;\n");
    content
}

fn rust_type_to_ts(rust_type: &str) -> &str {
    match rust_type {
        "String" | "&str" => "string",
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64" | "isize"
        | "usize" => "number",
        "bool" => "boolean",
        _ => "string",
    }
}
