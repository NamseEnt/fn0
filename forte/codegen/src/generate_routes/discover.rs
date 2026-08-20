use super::model::*;
use quote::quote;
use std::{fs, path::Path};

pub(super) fn discover_hooks(hooks_dir: &Path) -> Vec<HookInfo> {
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

pub(super) fn discover_actions(actions_dir: &Path) -> Vec<ActionInfo> {
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
                actions.push(ActionInfo { name: file_name });
            }
        } else if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
            let mod_rs = path.join("mod.rs");
            let Some(content) = fs::read_to_string(&mod_rs).ok() else {
                continue;
            };

            if has_action_handler(&content) {
                actions.push(ActionInfo { name: dir_name });
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

pub(super) fn discover_queue_tasks(queue_task_dir: &Path) -> Vec<QueueTaskInfo> {
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

pub(super) fn discover_admin_tasks(admin_dir: &Path) -> Vec<AdminTaskInfo> {
    let mut admin_tasks = Vec::new();

    if !admin_dir.exists() {
        return admin_tasks;
    }

    let Ok(entries) = fs::read_dir(admin_dir) else {
        return admin_tasks;
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
                admin_tasks.push(AdminTaskInfo { name: file_name });
            }
        }
    }

    admin_tasks
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
            syn::Item::Type(item_type) => {
                if item_type.ident == "Input" {
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
                if type_str.contains("Result") && type_str.contains("ForteResponse") {
                    return HandlerType::RawResponse;
                }
                if type_str.contains("Result") && type_str.contains("Props") {
                    if props_alias_contains(content, "ForteResponse") {
                        return HandlerType::RawResponse;
                    }
                    if props_alias_contains(content, "Redirect") {
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

fn props_alias_contains(content: &str, target_type_name: &str) -> bool {
    let Ok(syntax_tree) = syn::parse_file(content) else {
        return false;
    };

    for item in syntax_tree.items {
        if let syn::Item::Type(type_alias) = item
            && type_alias.ident == "Props"
        {
            let aliased_type = &type_alias.ty;
            let type_str = quote!(#aliased_type).to_string();
            return type_str.contains(target_type_name);
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

pub(super) fn discover_public(public_dir: &Path) -> Vec<StaticFileInfo> {
    let mut files = Vec::new();
    if !public_dir.exists() {
        return files;
    }
    collect_public_files(public_dir, public_dir, &mut files);
    files.sort_by(|a, b| a.route_path.cmp(&b.route_path));
    files
}

fn collect_public_files(base_dir: &Path, current_dir: &Path, files: &mut Vec<StaticFileInfo>) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            collect_public_files(base_dir, &path, files);
            continue;
        }

        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        if file_name == ".DS_Store" {
            continue;
        }

        let relative = path
            .strip_prefix(base_dir)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");

        files.push(StaticFileInfo {
            route_path: format!("/{}", relative),
            include_path: format!("../public/{}", relative),
            content_type: content_type_for(&path).to_string(),
        });
    }
}

fn content_type_for(path: &Path) -> &'static str {
    if path.file_name().and_then(|n| n.to_str()) == Some("apple-app-site-association") {
        return "application/json";
    }

    match path.extension().and_then(|e| e.to_str()) {
        Some("txt") => "text/plain; charset=utf-8",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("html") => "text/html; charset=utf-8",
        Some("webmanifest") => "application/manifest+json",
        Some("ico") => "image/x-icon",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

pub(super) fn discover_pages(pages_dir: &Path) -> Vec<PageInfo> {
    let mut pages = Vec::new();
    if !pages_dir.exists() {
        return pages;
    }
    discover_endpoints_recursive(pages_dir, pages_dir, &mut pages, "pages", &[], false);
    pages
}

pub(super) fn discover_apis(apis_dir: &Path) -> Vec<PageInfo> {
    let mut endpoints = Vec::new();
    if !apis_dir.exists() {
        return endpoints;
    }
    discover_endpoints_recursive(
        apis_dir,
        apis_dir,
        &mut endpoints,
        "apis",
        &["api".to_string()],
        true,
    );
    endpoints
}

pub(super) fn discover_websockets(ws_in_dir: &Path, ws_out_dir: &Path) -> Vec<WebSocketRouteInfo> {
    let mut routes = Vec::new();
    if ws_in_dir.exists() {
        discover_websockets_recursive(
            ws_in_dir,
            ws_in_dir,
            &mut routes,
            WebSocketDirection::Inbound,
            "ws_in",
        );
    }
    if ws_out_dir.exists() {
        discover_websockets_recursive(
            ws_out_dir,
            ws_out_dir,
            &mut routes,
            WebSocketDirection::Outbound,
            "ws_out",
        );
    }
    routes.sort_by(|left, right| {
        for (left_segment, right_segment) in
            left.route_segments.iter().zip(right.route_segments.iter())
        {
            match (left_segment, right_segment) {
                (RouteSegment::Static(left_value), RouteSegment::Static(right_value)) => {
                    let ordering = left_value.cmp(right_value);
                    if !ordering.is_eq() {
                        return ordering;
                    }
                }
                (RouteSegment::Static(_), RouteSegment::Dynamic(_)) => {
                    return std::cmp::Ordering::Less;
                }
                (RouteSegment::Dynamic(_), RouteSegment::Static(_)) => {
                    return std::cmp::Ordering::Greater;
                }
                (RouteSegment::Dynamic(left_value), RouteSegment::Dynamic(right_value)) => {
                    let ordering = left_value.cmp(right_value);
                    if !ordering.is_eq() {
                        return ordering;
                    }
                }
            }
        }
        right.route_segments.len().cmp(&left.route_segments.len())
    });
    routes
}

pub(super) fn discover_websocket_singletons(singleton_dir: &Path) -> Vec<WebSocketRouteInfo> {
    let mut routes = Vec::new();
    if singleton_dir.exists() {
        discover_websocket_singletons_recursive(singleton_dir, singleton_dir, &mut routes);
    }
    routes.sort_by(|left, right| left.route_path.cmp(&right.route_path));
    routes
}

fn discover_websocket_singletons_recursive(
    base_dir: &Path,
    current_dir: &Path,
    routes: &mut Vec<WebSocketRouteInfo>,
) {
    let entries = fs::read_dir(current_dir)
        .unwrap_or_else(|error| panic!("Failed to read {}: {error}", current_dir.display()));
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            discover_websocket_singletons_recursive(base_dir, &path, routes);
            continue;
        }
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let relative_path = path.strip_prefix(base_dir).unwrap();
        let relative_without_extension = relative_path.with_extension("");
        let source_segments: Vec<String> = relative_without_extension
            .components()
            .map(|component| component.as_os_str().to_string_lossy().to_string())
            .collect();
        if source_segments
            .iter()
            .any(|segment| segment.starts_with('[') && segment.ends_with(']'))
        {
            panic!(
                "WebSocket singleton {} cannot use dynamic path segments",
                path.display()
            );
        }
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("Failed to read {}: {error}", path.display()));
        let handlers = singleton_handlers(&content).unwrap_or_else(|error| {
            panic!("Invalid WebSocket singleton {}: {error}", path.display())
        });
        let singleton_id = source_segments.join("/");
        let module_suffix = source_segments.join("_");
        let mut route_segment_names = vec!["ws_singleton".to_string()];
        route_segment_names.extend(source_segments.iter().cloned());
        routes.push(WebSocketRouteInfo {
            direction: WebSocketDirection::Singleton,
            module_name: format!("websockets_singleton_{module_suffix}"),
            module_path: format!("ws_singleton/{}", relative_path.to_string_lossy()),
            module_segments: source_segments,
            route_path: format!("/{}", route_segment_names.join("/")),
            route_segments: route_segment_names
                .into_iter()
                .map(RouteSegment::Static)
                .collect(),
            path_params: None,
            has_on_connect: handlers.has_on_connect,
            has_on_disconnect: handlers.has_on_disconnect,
            singleton_id: Some(singleton_id),
        });
    }
}

struct SingletonHandlers {
    has_on_connect: bool,
    has_on_disconnect: bool,
}

fn singleton_handlers(content: &str) -> Result<SingletonHandlers, String> {
    let syntax_tree = syn::parse_file(content).map_err(|error| error.to_string())?;
    let mut connect = None;
    let mut on_connect = None;
    let mut on_message = None;
    let mut on_disconnect = None;
    for item in syntax_tree.items {
        let syn::Item::Fn(function) = item else {
            continue;
        };
        match function.sig.ident.to_string().as_str() {
            "connect" => connect = Some(function),
            "on_connect" => on_connect = Some(function),
            "on_message" => on_message = Some(function),
            "on_disconnect" => on_disconnect = Some(function),
            _ => {}
        }
    }
    let connect = connect.ok_or_else(|| "missing required connect function".to_string())?;
    validate_singleton_function(&connect, None, "SingletonConnectionOptions")?;
    let on_message =
        on_message.ok_or_else(|| "missing required on_message function".to_string())?;
    validate_singleton_function(&on_message, Some("MessageEvent"), "()")?;
    if let Some(function) = &on_connect {
        validate_singleton_function(function, Some("SingletonConnectEvent"), "()")?;
    }
    if let Some(function) = &on_disconnect {
        validate_singleton_function(function, Some("DisconnectEvent"), "()")?;
    }
    Ok(SingletonHandlers {
        has_on_connect: on_connect.is_some(),
        has_on_disconnect: on_disconnect.is_some(),
    })
}

fn validate_singleton_function(
    function: &syn::ItemFn,
    expected_input: Option<&str>,
    expected_result: &str,
) -> Result<(), String> {
    let function_name = function.sig.ident.to_string();
    if !matches!(function.vis, syn::Visibility::Public(_)) || function.sig.asyncness.is_none() {
        return Err(format!("{function_name} must be public and async"));
    }
    let expected_inputs = usize::from(expected_input.is_some());
    if function.sig.inputs.len() != expected_inputs {
        return Err(format!(
            "{function_name} must accept {expected_inputs} argument(s)"
        ));
    }
    if let Some(expected_input) = expected_input {
        let Some(syn::FnArg::Typed(argument)) = function.sig.inputs.first() else {
            return Err(format!("{function_name} must accept {expected_input}"));
        };
        if !type_is_named(&argument.ty, expected_input) {
            return Err(format!("{function_name} must accept {expected_input}"));
        }
    }
    let syn::ReturnType::Type(_, return_type) = &function.sig.output else {
        return Err(format!(
            "{function_name} must return Result<{expected_result}>"
        ));
    };
    if !result_type_matches(return_type, expected_result) {
        return Err(format!(
            "{function_name} must return Result<{expected_result}>"
        ));
    }
    Ok(())
}

fn type_is_named(rust_type: &syn::Type, expected: &str) -> bool {
    let syn::Type::Path(type_path) = rust_type else {
        return false;
    };
    type_path
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == expected)
}

fn result_type_matches(rust_type: &syn::Type, expected: &str) -> bool {
    let syn::Type::Path(type_path) = rust_type else {
        return false;
    };
    let Some(result_segment) = type_path.path.segments.last() else {
        return false;
    };
    if result_segment.ident != "Result" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &result_segment.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(success_type)) = arguments.args.first() else {
        return false;
    };
    if expected == "()" {
        matches!(success_type, syn::Type::Tuple(tuple) if tuple.elems.is_empty())
    } else {
        type_is_named(success_type, expected)
    }
}

fn discover_websockets_recursive(
    base_dir: &Path,
    current_dir: &Path,
    routes: &mut Vec<WebSocketRouteInfo>,
    direction: WebSocketDirection,
    module_root: &str,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            discover_websockets_recursive(base_dir, &path, routes, direction, module_root);
            continue;
        }
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let Some(content) = fs::read_to_string(&path).ok() else {
            continue;
        };
        let Some((has_on_connect, has_on_message, has_on_disconnect)) =
            websocket_handlers(&content)
        else {
            continue;
        };
        match direction {
            WebSocketDirection::Inbound if !has_on_connect || !has_on_message => {
                panic!(
                    "Inbound WebSocket route {} must define public async on_connect and on_message functions",
                    path.display()
                );
            }
            WebSocketDirection::Outbound if has_on_connect || !has_on_message => {
                panic!(
                    "Outbound WebSocket route {} must define public async on_message and must not define on_connect",
                    path.display()
                );
            }
            _ => {}
        }

        let relative_path = path.strip_prefix(base_dir).unwrap();
        let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
        let parent_segments: Vec<String> = relative_path
            .parent()
            .map(|parent| {
                parent
                    .components()
                    .map(|component| component.as_os_str().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default();
        let mut source_segments = parent_segments;
        if file_name != "index" && file_name != "mod" {
            source_segments.push(file_name);
        }
        if source_segments
            .last()
            .is_some_and(|segment| segment == "index")
        {
            source_segments.pop();
        }
        if direction == WebSocketDirection::Outbound
            && source_segments
                .iter()
                .any(|segment| segment.starts_with('[') && segment.ends_with(']'))
        {
            panic!(
                "Outbound WebSocket route {} cannot use dynamic path segments",
                path.display()
            );
        }
        let route_prefix = match direction {
            WebSocketDirection::Inbound => "ws",
            WebSocketDirection::Outbound => "ws_out",
            WebSocketDirection::Singleton => unreachable!(),
        };
        let module_prefix = match direction {
            WebSocketDirection::Inbound => "ws_in",
            WebSocketDirection::Outbound => "ws_out",
            WebSocketDirection::Singleton => unreachable!(),
        };
        let mut route_segment_names = vec![route_prefix.to_string()];
        route_segment_names.extend(source_segments.iter().cloned());
        let module_suffix = if source_segments.is_empty() {
            module_prefix.to_string()
        } else {
            std::iter::once(module_prefix.to_string())
                .chain(source_segments.iter().cloned())
                .map(|segment| {
                    if segment.starts_with('[') && segment.ends_with(']') {
                        format!("_{}_", &segment[1..segment.len() - 1])
                    } else {
                        segment.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join("_")
        };
        let mut module_segments = vec![module_prefix.to_string()];
        module_segments.extend(source_segments.iter().map(|segment| {
            if segment.starts_with('[') && segment.ends_with(']') {
                format!("_{}_", &segment[1..segment.len() - 1])
            } else {
                segment.clone()
            }
        }));
        let route_segments = route_segment_names
            .iter()
            .map(|segment| {
                if segment.starts_with('[') && segment.ends_with(']') {
                    RouteSegment::Dynamic(segment[1..segment.len() - 1].to_string())
                } else {
                    RouteSegment::Static(segment.clone())
                }
            })
            .collect();
        routes.push(WebSocketRouteInfo {
            direction,
            module_name: format!("websockets_{module_suffix}"),
            module_path: format!("{module_root}/{}", relative_path.to_string_lossy()),
            module_segments,
            route_path: format!("/{}", route_segment_names.join("/")),
            route_segments,
            path_params: parse_path_params(&content),
            has_on_connect,
            has_on_disconnect,
            singleton_id: None,
        });
    }
}

fn websocket_handlers(content: &str) -> Option<(bool, bool, bool)> {
    let syntax_tree = syn::parse_file(content).ok()?;
    let mut has_any = false;
    let mut has_on_connect = false;
    let mut has_on_message = false;
    let mut has_on_disconnect = false;
    for item in syntax_tree.items {
        let syn::Item::Fn(function) = item else {
            continue;
        };
        let is_public_async =
            matches!(function.vis, syn::Visibility::Public(_)) && function.sig.asyncness.is_some();
        if !is_public_async {
            continue;
        }
        match function.sig.ident.to_string().as_str() {
            "on_connect" => {
                has_any = true;
                has_on_connect = true;
            }
            "on_message" => {
                has_any = true;
                has_on_message = true;
            }
            "on_disconnect" => {
                has_any = true;
                has_on_disconnect = true;
            }
            _ => {}
        }
    }
    has_any.then_some((has_on_connect, has_on_message, has_on_disconnect))
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
            let response_kind = match handler_type {
                HandlerType::None => continue,
                HandlerType::Props => HandlerResponseKind::PropsJson,
                HandlerType::Redirect => HandlerResponseKind::RedirectOnly,
                HandlerType::RawResponse => {
                    if !is_api {
                        panic!(
                            "ForteResponse handlers are only supported under src/apis/ — {} is a page route and must return Props for SSR",
                            path.display()
                        );
                    }
                    HandlerResponseKind::RawResponse
                }
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
            let cache_static = has_cache_static_attribute(&content);
            let has_cache_static_eligible = has_public_fn(&content, "cache_static_eligible");

            pages.push(PageInfo {
                module_name,
                module_path,
                route_path,
                route_segments: parsed_route_segments,
                path_params,
                search_params,
                response_kind,
                is_api,
                cache_static,
                has_cache_static_eligible,
            });
        }
    }
}

fn has_public_fn(content: &str, name: &str) -> bool {
    let Ok(syntax_tree) = syn::parse_file(content) else {
        return false;
    };

    syntax_tree.items.iter().any(|item| {
        let syn::Item::Fn(function) = item else {
            return false;
        };
        function.sig.ident == name && matches!(function.vis, syn::Visibility::Public(_))
    })
}

fn has_cache_static_attribute(content: &str) -> bool {
    let Ok(syntax_tree) = syn::parse_file(content) else {
        return false;
    };

    syntax_tree.items.iter().any(|item| {
        let syn::Item::Fn(function) = item else {
            return false;
        };
        function.sig.ident == "handler"
            && function.attrs.iter().any(|attribute| {
                attribute
                    .path()
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "cache_static")
            })
    })
}

#[cfg(test)]
mod websocket_singleton_tests {
    use super::*;

    const VALID_SINGLETON: &str = r#"
pub async fn connect() -> Result<SingletonConnectionOptions> { todo!() }
pub async fn on_connect(event: SingletonConnectEvent) -> Result<()> { todo!() }
pub async fn on_message(event: MessageEvent) -> Result<()> { todo!() }
pub async fn on_disconnect(event: DisconnectEvent) -> Result<()> { todo!() }
"#;

    #[test]
    fn discovers_nested_singleton_id_and_route() {
        let directory = tempfile::tempdir().unwrap();
        let nested = directory.path().join("feeds/us");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("market.rs"), VALID_SINGLETON).unwrap();
        let routes = discover_websocket_singletons(directory.path());
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].singleton_id.as_deref(), Some("feeds/us/market"));
        assert_eq!(routes[0].route_path, "/ws_singleton/feeds/us/market");
        assert!(routes[0].has_on_connect);
        assert!(routes[0].has_on_disconnect);
    }

    #[test]
    #[should_panic(expected = "missing required connect function")]
    fn rejects_missing_connect() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("market.rs"),
            "pub async fn on_message(event: MessageEvent) -> Result<()> { todo!() }",
        )
        .unwrap();
        discover_websocket_singletons(directory.path());
    }

    #[test]
    #[should_panic(expected = "cannot use dynamic path segments")]
    fn rejects_dynamic_paths() {
        let directory = tempfile::tempdir().unwrap();
        let dynamic = directory.path().join("[market]");
        fs::create_dir_all(&dynamic).unwrap();
        fs::write(dynamic.join("feed.rs"), VALID_SINGLETON).unwrap();
        discover_websocket_singletons(directory.path());
    }

    #[test]
    #[should_panic(expected = "on_message must accept MessageEvent")]
    fn rejects_invalid_callback_shape() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("market.rs"),
            r#"
pub async fn connect() -> Result<SingletonConnectionOptions> { todo!() }
pub async fn on_message(event: DisconnectEvent) -> Result<()> { todo!() }
"#,
        )
        .unwrap();
        discover_websocket_singletons(directory.path());
    }
}
