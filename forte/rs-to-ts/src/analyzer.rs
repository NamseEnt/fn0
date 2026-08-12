use crate::name_resolution::{apply_name_resolution_to_type, resolve_type_names};
use crate::rust_to_ts::TypeConverter;
use crate::ts_codegen::{TsDefinition, TsType, format_definition, to_zod};
use rustc_driver::{Callbacks, Compilation};
use rustc_hir::def::DefKind;
use rustc_interface::interface::Compiler;
use rustc_middle::ty::{TyCtxt, Visibility};
use rustc_span::def_id::DefId;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Analyzer {
    pub ts_output_dir: String,
    pub hooks_output_dir: String,
    pub actions_output_dir: String,
}

fn get_module_actual_span<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId) -> rustc_span::Span {
    if let Some(local_def_id) = def_id.as_local() {
        let hir_id = tcx.local_def_id_to_hir_id(local_def_id);
        if let rustc_hir::Node::Item(item) = tcx.hir_node(hir_id)
            && let rustc_hir::ItemKind::Mod(_, mod_ref) = &item.kind
            && let Some(first_item_id) = mod_ref.item_ids.first()
        {
            let first_item_hir_id = first_item_id.hir_id();
            if let rustc_hir::Node::Item(first_item) = tcx.hir_node(first_item_hir_id) {
                return first_item.span;
            }
        }
    }
    tcx.def_span(def_id)
}

fn convert_rust_path_to_ts_path(rust_path: &str, ts_output_dir: &str) -> PathBuf {
    let path_str = rust_path.to_string();
    let path_parts: Vec<&str> = path_str.split('/').collect();
    let src_pages_idx = path_parts.iter().position(|&p| p == "pages");

    if let Some(idx) = src_pages_idx {
        let after_pages = &path_parts[idx + 1..];
        let relative_path = if after_pages.last() == Some(&"mod.rs") {
            after_pages[..after_pages.len() - 1].join("/")
        } else if let Some(last) = after_pages.last()
            && last.ends_with(".rs")
        {
            let file_stem = last.trim_end_matches(".rs");
            if after_pages.len() == 1 {
                file_stem.to_string()
            } else {
                format!(
                    "{}/{}",
                    after_pages[..after_pages.len() - 1].join("/"),
                    file_stem
                )
            }
        } else {
            after_pages.join("/")
        };

        let mut output_path = PathBuf::from(ts_output_dir);
        output_path.push(relative_path);
        output_path.push(".props.ts");

        output_path
    } else {
        PathBuf::from(ts_output_dir)
    }
}

fn generate_ts_file_content(
    rust_source_path: &str,
    ts_type: &TsType,
    definitions: &[TsDefinition],
) -> String {
    let mut file_content = String::new();
    file_content.push_str(&format!("// Auto-generated from {}\n\n", rust_source_path));
    file_content.push_str("import { z } from \"zod\";\n\n");

    let mut namespace_groups: HashMap<Vec<String>, Vec<&TsDefinition>> = HashMap::new();
    for def in definitions {
        namespace_groups
            .entry(def.namespace.clone())
            .or_default()
            .push(def);
    }

    if let Some(top_level_defs) = namespace_groups.get(&vec![]) {
        for def in top_level_defs {
            file_content.push_str(&format_definition(def));
            file_content.push_str("\n\n");
        }
    }

    let mut namespaces: Vec<Vec<String>> = namespace_groups
        .keys()
        .filter(|ns| !ns.is_empty())
        .cloned()
        .collect();
    namespaces.sort();

    for namespace in namespaces {
        file_content.push_str(&format!("export namespace {} {{\n", namespace.join(".")));

        if let Some(defs) = namespace_groups.get(&namespace) {
            for def in defs {
                let def_str = format_definition(def);
                for line in def_str.lines() {
                    file_content.push_str(&format!("    {}\n", line));
                }
                file_content.push('\n');
            }
        }

        file_content.push_str("}\n\n");
    }

    let props_zod = to_zod(ts_type);
    file_content.push_str(&format!(
        "export const PropsSchema = {};\n\nexport type Props = z.infer<typeof PropsSchema>;\n",
        props_zod
    ));

    file_content
}

fn filename_to_path_string(filename: &rustc_span::FileName) -> String {
    if let rustc_span::FileName::Real(path) = filename {
        path.clone()
            .into_local_path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("{:?}", filename))
    } else {
        format!("{:?}", filename)
    }
}

fn filename_to_local_path(filename: &rustc_span::FileName) -> Option<String> {
    if let rustc_span::FileName::Real(path) = filename {
        path.clone()
            .into_local_path()
            .map(|p| p.to_string_lossy().to_string())
    } else {
        None
    }
}

fn discover_module_paths<'tcx>(tcx: TyCtxt<'tcx>) -> Vec<(DefId, String)> {
    let items = tcx.hir_crate_items(());
    let modules = Mutex::new(Vec::new());
    let _ = items.par_items(|item_id| {
        let owner_id = item_id.owner_id;
        let def_id: DefId = owner_id.to_def_id();
        if tcx.def_kind(def_id) == DefKind::Mod {
            let span = get_module_actual_span(tcx, def_id);
            let source_map = tcx.sess.source_map();
            let filename = source_map.span_to_filename(span);
            if let rustc_span::FileName::Real(path) = filename
                && let Some(local_path) = path.into_local_path()
            {
                let path_str = local_path.to_string_lossy().to_string();
                let mut modules = modules.lock().unwrap();
                modules.push((def_id, path_str));
            }
        }
        Ok(())
    });
    modules.into_inner().unwrap()
}

fn is_page_path(path_str: &str) -> bool {
    if !path_str.contains("src/pages") {
        return false;
    }
    let is_mod_rs = path_str.ends_with("mod.rs");
    let is_single_page_rs = !is_mod_rs && path_str.ends_with(".rs") && !path_str.contains("/api/");
    if !is_mod_rs && !is_single_page_rs {
        return false;
    }
    path_str.split('/').any(|segment| segment == "pages")
}

fn is_hook_path(path_str: &str) -> bool {
    path_str.contains("src/hooks/") && path_str.ends_with(".rs") && !path_str.ends_with("mod.rs")
}

fn is_action_path(path_str: &str) -> bool {
    let Some(after) = path_str.split("src/actions/").nth(1) else {
        return false;
    };
    match after.rsplit_once('/') {
        Some((dir, "mod.rs")) => !dir.contains('/'),
        Some(_) => false,
        None => after.ends_with(".rs") && after != "mod.rs",
    }
}

fn action_name_from_path(path: &str) -> &str {
    let mut segments = path.rsplit('/');
    match segments.next() {
        Some("mod.rs") => segments.next().unwrap_or("unknown"),
        Some(file) => file.trim_end_matches(".rs"),
        None => "unknown",
    }
}

fn scan_io_module<'tcx>(
    tcx: TyCtxt<'tcx>,
    hir_id: rustc_hir::HirId,
) -> (bool, Option<DefId>, Option<DefId>) {
    let mut input_def_id: Option<DefId> = None;
    let mut output_def_id: Option<DefId> = None;
    let mut handler_found = false;

    if let rustc_hir::Node::Item(item) = tcx.hir_node(hir_id)
        && let rustc_hir::ItemKind::Mod(_, mod_ref) = &item.kind
    {
        for item_id in mod_ref.item_ids {
            let item_hir_id = item_id.hir_id();
            if let rustc_hir::Node::Item(child_item) = tcx.hir_node(item_hir_id) {
                let child_def_id = child_item.owner_id.to_def_id();
                let item_name_str = tcx.def_path_str(child_def_id);
                if let Some((_, name)) = item_name_str.rsplit_once("::") {
                    match name {
                        "Input" => {
                            let kind = tcx.def_kind(child_def_id);
                            if kind == DefKind::Struct {
                                input_def_id = Some(child_def_id);
                            }
                        }
                        "Output" => {
                            let kind = tcx.def_kind(child_def_id);
                            if kind == DefKind::Struct || kind == DefKind::Enum {
                                output_def_id = Some(child_def_id);
                            }
                        }
                        "handler" => {
                            if tcx.def_kind(child_def_id) == DefKind::Fn
                                && matches!(tcx.visibility(child_def_id), Visibility::Public)
                            {
                                handler_found = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    (handler_found, input_def_id, output_def_id)
}

fn scan_page_module<'tcx>(
    tcx: TyCtxt<'tcx>,
    hir_id: rustc_hir::HirId,
    filename: &rustc_span::FileName,
) -> (bool, Option<DefId>, Option<DefKind>) {
    let mut handler_found = false;
    let mut props_def_id: Option<DefId> = None;
    let mut props_kind: Option<DefKind> = None;

    if let rustc_hir::Node::Item(item) = tcx.hir_node(hir_id)
        && let rustc_hir::ItemKind::Mod(_, mod_ref) = &item.kind
    {
        for item_id in mod_ref.item_ids {
            let item_hir_id = item_id.hir_id();
            if let rustc_hir::Node::Item(child_item) = tcx.hir_node(item_hir_id) {
                let child_def_id = child_item.owner_id.to_def_id();
                let item_name_str = tcx.def_path_str(child_def_id);
                if let Some((_, name)) = item_name_str.rsplit_once("::") {
                    if name == "handler" {
                        if tcx.def_kind(child_def_id) == DefKind::Fn {
                            if matches!(tcx.visibility(child_def_id), Visibility::Public) {
                                handler_found = true;
                            } else {
                                eprintln!(
                                    "Error: handler in {} must be public",
                                    filename_to_path_string(filename)
                                );
                                std::process::exit(1);
                            }
                        }
                    } else if name == "Props" {
                        let item_def_kind = tcx.def_kind(child_def_id);
                        if item_def_kind == DefKind::Struct
                            || item_def_kind == DefKind::Enum
                            || item_def_kind == DefKind::TyAlias
                        {
                            props_def_id = Some(child_def_id);
                            props_kind = Some(item_def_kind);
                        } else {
                            eprintln!(
                                "Error: Props in {} must be a struct, enum, or type alias",
                                filename_to_path_string(filename)
                            );
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    }

    (handler_found, props_def_id, props_kind)
}

fn resolve_names(converter: &mut TypeConverter<'_>, root_types: Vec<&mut TsType>) {
    let name_map = resolve_type_names(&converter.definitions);

    for def in &mut converter.definitions {
        if let Some(resolved) = name_map.get(&def.full_path) {
            def.namespace = resolved.namespace.clone();
            def.type_name = resolved.type_name.clone();
        }
    }

    for def in &mut converter.definitions {
        apply_name_resolution_to_type(&mut def.ty, &name_map);
    }

    for ty in root_types {
        apply_name_resolution_to_type(ty, &name_map);
    }
}

fn inline_root_reference(definitions: &mut Vec<TsDefinition>, ts_type: &mut TsType) {
    let TsType::Reference(root_ref_string) = &*ts_type else {
        return;
    };
    let found_idx = definitions.iter().position(|def| {
        let def_ref = if def.namespace.is_empty() {
            def.type_name.clone()
        } else {
            format!("{}.{}", def.namespace.join("."), def.type_name)
        };
        &def_ref == root_ref_string
    });
    if let Some(idx) = found_idx {
        let root_def = definitions.remove(idx);
        *ts_type = root_def.ty;
    }
}

impl Analyzer {
    fn process_pages<'tcx>(&self, tcx: TyCtxt<'tcx>) {
        let modules: Vec<DefId> = discover_module_paths(tcx)
            .into_iter()
            .filter(|(_, path_str)| is_page_path(path_str))
            .map(|(def_id, _)| def_id)
            .collect();

        let source_map = tcx.sess.source_map();
        for def_id in modules {
            let span = get_module_actual_span(tcx, def_id);
            let filename = source_map.span_to_filename(span);
            if let Some(path) = filename_to_local_path(&filename) {
                println!("Found: {}", path);
            }

            let local_def_id = match def_id.as_local() {
                Some(id) => id,
                None => continue,
            };
            let hir_id = tcx.local_def_id_to_hir_id(local_def_id);

            let (handler_found, props_def_id, props_kind) =
                scan_page_module(tcx, hir_id, &filename);

            if !handler_found {
                eprintln!(
                    "Error: handler not found or not public in {}",
                    filename_to_path_string(&filename)
                );
                std::process::exit(1);
            }

            let Some(props_id) = props_def_id else {
                eprintln!(
                    "Error: Props not found in {}",
                    filename_to_path_string(&filename)
                );
                std::process::exit(1);
            };

            let props_ty = tcx.type_of(props_id).instantiate_identity();
            let props_ty_str = format!("{:?}", props_ty);
            if props_ty_str.contains("Redirect") {
                continue;
            }

            let kind_name = match props_kind {
                Some(DefKind::Struct) => "struct",
                Some(DefKind::Enum) => "enum",
                Some(DefKind::TyAlias) => "alias",
                _ => "unknown",
            };
            let rust_source_path = filename_to_path_string(&filename);

            println!("{} -> Props ({})", rust_source_path, kind_name);

            let mut converter = TypeConverter::new(tcx);
            let context = format!("{:?}", filename);
            let mut ts_type = converter.convert_type(props_ty, &context);
            resolve_names(&mut converter, vec![&mut ts_type]);
            inline_root_reference(&mut converter.definitions, &mut ts_type);

            let file_content =
                generate_ts_file_content(&rust_source_path, &ts_type, &converter.definitions);

            println!("self.ts_output_dir: {}", self.ts_output_dir);

            let ts_output_path =
                convert_rust_path_to_ts_path(&rust_source_path, &self.ts_output_dir);

            if let Some(parent) = ts_output_path.parent()
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                eprintln!("Error creating directory {}: {}", parent.display(), e);
                std::process::exit(1);
            }

            if let Err(e) = std::fs::write(&ts_output_path, &file_content) {
                eprintln!("Error writing file {}: {}", ts_output_path.display(), e);
                std::process::exit(1);
            }

            println!(
                "Generated: {} -> {}",
                rust_source_path,
                ts_output_path.canonicalize().unwrap().display()
            );
        }
    }

    fn process_hooks<'tcx>(&self, tcx: TyCtxt<'tcx>) {
        let modules: Vec<DefId> = discover_module_paths(tcx)
            .into_iter()
            .filter(|(_, path_str)| is_hook_path(path_str))
            .map(|(def_id, _)| def_id)
            .collect();

        let source_map = tcx.sess.source_map();
        for def_id in modules {
            let span = get_module_actual_span(tcx, def_id);
            let filename = source_map.span_to_filename(span);
            if let Some(path) = filename_to_local_path(&filename) {
                println!("Found hook: {}", path);
            }

            let local_def_id = match def_id.as_local() {
                Some(id) => id,
                None => continue,
            };
            let hir_id = tcx.local_def_id_to_hir_id(local_def_id);

            let (handler_found, input_def_id, output_def_id) = scan_io_module(tcx, hir_id);

            if !handler_found || input_def_id.is_none() || output_def_id.is_none() {
                continue;
            }

            let input_id = input_def_id.unwrap();
            let output_id = output_def_id.unwrap();

            let rust_source_path = match filename_to_local_path(&filename) {
                Some(p) => p,
                None => continue,
            };

            let hook_name = rust_source_path
                .rsplit('/')
                .next()
                .unwrap_or("unknown")
                .trim_end_matches(".rs");

            let mut converter = TypeConverter::new(tcx);

            let input_ty = tcx.type_of(input_id).instantiate_identity();
            let mut input_ts_type = converter.convert_type(input_ty, &rust_source_path);

            let output_ty = tcx.type_of(output_id).instantiate_identity();
            let mut output_ts_type = converter.convert_type(output_ty, &rust_source_path);

            resolve_names(
                &mut converter,
                vec![&mut input_ts_type, &mut output_ts_type],
            );

            inline_root_reference(&mut converter.definitions, &mut input_ts_type);
            inline_root_reference(&mut converter.definitions, &mut output_ts_type);

            let file_content = generate_hook_file_content(
                &rust_source_path,
                hook_name,
                &input_ts_type,
                &output_ts_type,
                &converter.definitions,
            );

            let mut output_path = PathBuf::from(&self.hooks_output_dir);
            let use_hook_name = format!(
                "use{}{}.ts",
                hook_name.chars().next().unwrap().to_uppercase(),
                &hook_name[1..]
            );
            output_path.push(&use_hook_name);

            if let Some(parent) = output_path.parent()
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                eprintln!("Error creating directory {}: {}", parent.display(), e);
                std::process::exit(1);
            }

            if let Err(e) = std::fs::write(&output_path, &file_content) {
                eprintln!("Error writing file {}: {}", output_path.display(), e);
                std::process::exit(1);
            }

            println!(
                "Generated hook: {} -> {}",
                rust_source_path,
                output_path.canonicalize().unwrap().display()
            );
        }
    }

    fn process_actions<'tcx>(&self, tcx: TyCtxt<'tcx>) {
        let modules: Vec<DefId> = discover_module_paths(tcx)
            .into_iter()
            .filter(|(_, path_str)| is_action_path(path_str))
            .map(|(def_id, _)| def_id)
            .collect();

        let source_map = tcx.sess.source_map();
        let mut action_names: Vec<String> = Vec::new();

        for def_id in modules {
            let span = get_module_actual_span(tcx, def_id);
            let filename = source_map.span_to_filename(span);
            if let Some(path) = filename_to_local_path(&filename) {
                println!("Found action: {}", path);
            }

            let local_def_id = match def_id.as_local() {
                Some(id) => id,
                None => continue,
            };
            let hir_id = tcx.local_def_id_to_hir_id(local_def_id);

            let (handler_found, input_def_id, output_def_id) = scan_io_module(tcx, hir_id);

            if !handler_found || input_def_id.is_none() || output_def_id.is_none() {
                continue;
            }

            let input_id = input_def_id.unwrap();
            let output_id = output_def_id.unwrap();

            let rust_source_path = match filename_to_local_path(&filename) {
                Some(p) => p,
                None => continue,
            };

            let action_name = action_name_from_path(&rust_source_path);

            action_names.push(action_name.to_string());

            let mut converter = TypeConverter::new(tcx);

            let input_ty = tcx.type_of(input_id).instantiate_identity();
            let mut input_ts_type = converter.convert_type(input_ty, &rust_source_path);

            let output_ty = tcx.type_of(output_id).instantiate_identity();
            let mut output_ts_type = converter.convert_type(output_ty, &rust_source_path);

            resolve_names(
                &mut converter,
                vec![&mut input_ts_type, &mut output_ts_type],
            );

            inline_root_reference(&mut converter.definitions, &mut input_ts_type);
            inline_root_reference(&mut converter.definitions, &mut output_ts_type);

            let file_content = generate_action_file_content(
                &rust_source_path,
                action_name,
                &input_ts_type,
                &output_ts_type,
                &converter.definitions,
            );

            let mut output_path = PathBuf::from(&self.actions_output_dir);
            let file_name = format!("{}.ts", action_name);
            output_path.push(&file_name);

            if let Some(parent) = output_path.parent()
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                eprintln!("Error creating directory {}: {}", parent.display(), e);
                std::process::exit(1);
            }

            if let Err(e) = std::fs::write(&output_path, &file_content) {
                eprintln!("Error writing file {}: {}", output_path.display(), e);
                std::process::exit(1);
            }

            println!(
                "Generated action: {} -> {}",
                rust_source_path,
                output_path.canonicalize().unwrap().display()
            );
        }

        if !action_names.is_empty() {
            let index_content = generate_actions_index(&action_names);
            let mut index_path = PathBuf::from(&self.actions_output_dir);
            index_path.push("index.ts");

            if let Some(parent) = index_path.parent()
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                eprintln!("Error creating directory {}: {}", parent.display(), e);
                std::process::exit(1);
            }

            if let Err(e) = std::fs::write(&index_path, &index_content) {
                eprintln!("Error writing file {}: {}", index_path.display(), e);
                std::process::exit(1);
            }

            println!("Generated actions index: {}", index_path.display());
        }
    }
}

impl Callbacks for Analyzer {
    fn after_analysis<'tcx>(&mut self, _compiler: &Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        self.process_pages(tcx);
        self.process_hooks(tcx);
        self.process_actions(tcx);
        Compilation::Stop
    }
}

fn generate_hook_file_content(
    rust_source_path: &str,
    hook_name: &str,
    input_type: &TsType,
    output_type: &TsType,
    definitions: &[TsDefinition],
) -> String {
    let mut file_content = String::new();
    file_content.push_str(&format!("// Auto-generated from {}\n\n", rust_source_path));
    file_content.push_str("import { z } from \"zod\";\n");
    file_content.push_str("import { useForteHook } from \"@forte/react\";\n\n");

    for def in definitions {
        if def.namespace.is_empty() {
            let zod_schema = to_zod(&def.ty);
            file_content.push_str(&format!(
                "const {}Schema = {};\n\n",
                def.type_name, zod_schema
            ));
        }
    }

    let input_zod = to_zod(input_type);
    file_content.push_str(&format!("const InputSchema = {};\n\n", input_zod));

    let output_zod = to_zod(output_type);
    file_content.push_str(&format!("const OutputSchema = {};\n\n", output_zod));

    let capitalized_name = format!(
        "{}{}",
        hook_name.chars().next().unwrap().to_uppercase(),
        &hook_name[1..]
    );

    file_content.push_str(&format!(
        "export function use{}(input: z.infer<typeof InputSchema>) {{\n  return useForteHook(\"{}\", input, OutputSchema);\n}}\n",
        capitalized_name, hook_name
    ));

    file_content
}

fn generate_action_file_content(
    rust_source_path: &str,
    action_name: &str,
    input_type: &TsType,
    output_type: &TsType,
    definitions: &[TsDefinition],
) -> String {
    let mut file_content = String::new();
    file_content.push_str(&format!("// Auto-generated from {}\n\n", rust_source_path));
    file_content.push_str("import { z } from \"zod\";\n");
    file_content.push_str("import { callAction } from \"@forte/react\";\n\n");

    for def in definitions {
        if def.namespace.is_empty() {
            let zod_schema = to_zod(&def.ty);
            file_content.push_str(&format!(
                "const {}Schema = {};\n\n",
                def.type_name, zod_schema
            ));
        }
    }

    let input_zod = to_zod(input_type);
    file_content.push_str(&format!("const InputSchema = {};\n\n", input_zod));

    let output_zod = to_zod(output_type);
    file_content.push_str(&format!("const OutputSchema = {};\n\n", output_zod));

    let camel_case_name = to_camel_case(action_name);

    file_content.push_str(&format!(
        "export function {}(input: z.infer<typeof InputSchema>) {{\n  return callAction(\"{}\", input, OutputSchema);\n}}\n",
        camel_case_name, action_name
    ));

    file_content
}

fn generate_actions_index(action_names: &[String]) -> String {
    let mut content = String::new();
    content.push_str("// Auto-generated by forte dev\n\n");

    for name in action_names {
        let camel_case_name = to_camel_case(name);
        content.push_str(&format!(
            "export {{ {} }} from \"./{}\";\n",
            camel_case_name, name
        ));
    }

    content
}

fn to_camel_case(snake_case: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in snake_case.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}
