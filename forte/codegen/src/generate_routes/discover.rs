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
