use crate::ts_codegen::{TsDefinition, TsField, TsType};
use rustc_hir::def::CtorKind;
use rustc_middle::ty::{AdtDef, GenericArgsRef, Ty, TyCtxt};
use rustc_span::def_id::DefId;
use std::collections::HashSet;

fn snake_to_camel(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = false;

    for ch in s.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result
}

pub struct TypeConverter<'tcx> {
    tcx: TyCtxt<'tcx>,
    visited: HashSet<DefId>,
    processed: HashSet<DefId>,
    pub definitions: Vec<TsDefinition>,
}

impl<'tcx> TypeConverter<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        Self {
            tcx,
            visited: HashSet::new(),
            processed: HashSet::new(),
            definitions: Vec::new(),
        }
    }

    pub fn convert_type(&mut self, ty: Ty<'tcx>, context: &str) -> TsType {
        match ty.kind() {
            rustc_middle::ty::TyKind::Bool => TsType::Primitive("boolean".to_string()),
            rustc_middle::ty::TyKind::Int(_)
            | rustc_middle::ty::TyKind::Uint(_)
            | rustc_middle::ty::TyKind::Float(_) => TsType::Primitive("number".to_string()),
            rustc_middle::ty::TyKind::Str => TsType::Primitive("string".to_string()),
            rustc_middle::ty::TyKind::Char => TsType::Primitive("string".to_string()),
            rustc_middle::ty::TyKind::Ref(_, inner_ty, _) => self.convert_type(*inner_ty, context),
            rustc_middle::ty::TyKind::Array(inner_ty, _)
            | rustc_middle::ty::TyKind::Slice(inner_ty) => {
                TsType::Array(Box::new(self.convert_type(*inner_ty, context)))
            }
            rustc_middle::ty::TyKind::Tuple(types) => {
                let converted: Vec<TsType> = types
                    .iter()
                    .map(|t| self.convert_type(t, context))
                    .collect();
                TsType::Tuple(converted)
            }
            rustc_middle::ty::TyKind::Adt(adt_def, substs) => {
                if self.is_std_type(adt_def, "String") {
                    TsType::Primitive("string".to_string())
                } else if self.is_chrono_type(adt_def) {
                    TsType::Date
                } else if self.is_std_type(adt_def, "Option") {
                    let inner_ty = substs[0].expect_ty();
                    TsType::Undefined(Box::new(self.convert_type(inner_ty, context)))
                } else if self.is_std_type(adt_def, "Vec") {
                    let inner_ty = substs[0].expect_ty();
                    TsType::Array(Box::new(self.convert_type(inner_ty, context)))
                } else if self.is_std_type(adt_def, "HashMap")
                    || self.is_std_type(adt_def, "BTreeMap")
                {
                    let val_ty = substs[1].expect_ty();
                    TsType::Record(Box::new(self.convert_type(val_ty, context)))
                } else if self.is_std_type(adt_def, "HashSet")
                    || self.is_std_type(adt_def, "BTreeSet")
                {
                    let inner_ty = substs[0].expect_ty();
                    TsType::Array(Box::new(self.convert_type(inner_ty, context)))
                } else if self.is_std_type(adt_def, "Box")
                    || self.is_std_type(adt_def, "Rc")
                    || self.is_std_type(adt_def, "Arc")
                {
                    let inner_ty = substs[0].expect_ty();
                    self.convert_type(inner_ty, context)
                } else {
                    self.convert_adt(adt_def, substs, context)
                }
            }
            _ => {
                eprintln!("Error: Unsupported type: {:?} in {}", ty.kind(), context);
                std::process::exit(1);
            }
        }
    }

    fn is_chrono_type(&self, adt_def: &AdtDef<'tcx>) -> bool {
        let def_path = self.tcx.def_path_str(adt_def.did());
        // Simple check for chrono::DateTime, handling potential re-exports loosely or strictly
        // chrono::datetime::DateTime is the usual canonical path
        def_path.contains("chrono") && def_path.ends_with("DateTime")
    }

    fn is_std_type(&self, adt_def: &AdtDef<'tcx>, name: &str) -> bool {
        let def_path = self.tcx.def_path_str(adt_def.did());
        def_path == format!("std::string::{}", name)
            || def_path == format!("alloc::string::{}", name)
            || def_path == format!("std::option::{}", name)
            || def_path == format!("core::option::{}", name)
            || def_path == format!("std::vec::{}", name)
            || def_path == format!("alloc::vec::{}", name)
            || def_path == format!("std::collections::hash::map::{}", name)
            || def_path == format!("std::collections::hash::set::{}", name)
            || def_path == format!("std::collections::{}", name) // std::collections::HashMap
            || def_path == format!("hashbrown::map::{}", name) // hashbrown::map::HashMap
            || def_path == format!("hashbrown::{}", name) // hashbrown::HashMap
            || def_path == format!("std::collections::btree::map::{}", name)
            || def_path == format!("std::collections::btree::set::{}", name)
            || def_path == format!("alloc::collections::btree::map::{}", name)
            || def_path == format!("alloc::collections::btree::set::{}", name)
            || def_path == format!("std::boxed::{}", name)
            || def_path == format!("alloc::boxed::{}", name)
            || def_path == format!("std::rc::{}", name)
            || def_path == format!("alloc::rc::{}", name)
            || def_path == format!("std::sync::{}", name)
            || def_path == format!("alloc::sync::{}", name)
    }

    fn convert_adt(
        &mut self,
        adt_def: &AdtDef<'tcx>,
        substs: GenericArgsRef<'tcx>,
        context: &str,
    ) -> TsType {
        let def_id = adt_def.did();
        let type_name = self.tcx.def_path_str(def_id);

        if self.processed.contains(&def_id) {
            return TsType::Reference(type_name);
        }

        if !self.visited.insert(def_id) {
            return TsType::Reference(type_name);
        }

        let ts_type = if adt_def.is_struct() {
            self.convert_struct(def_id, substs, context)
        } else if adt_def.is_enum() {
            self.convert_enum(def_id, substs, adt_def, context)
        } else {
            eprintln!("Error: Unsupported ADT type: {} in {}", type_name, context);
            std::process::exit(1);
        };

        self.visited.remove(&def_id);
        self.processed.insert(def_id);

        self.definitions.push(TsDefinition {
            full_path: type_name.clone(),
            namespace: vec![],
            type_name: type_name.clone(),
            ty: ts_type,
        });

        TsType::Reference(type_name)
    }

    fn convert_struct(
        &mut self,
        def_id: DefId,
        substs: GenericArgsRef<'tcx>,
        context: &str,
    ) -> TsType {
        let _type_name = self.tcx.def_path_str(def_id);
        let fields: Vec<TsField> = self
            .tcx
            .adt_def(def_id)
            .all_fields()
            .map(|field| {
                let field_name = self.tcx.item_name(field.did).to_string();
                let field_ty = field.ty(self.tcx, substs);
                let field_context = format!("{}.{}", context, field_name);

                let (is_optional, actual_ty) =
                    if let rustc_middle::ty::TyKind::Adt(adt_def, substs) = field_ty.kind() {
                        if self.is_std_type(adt_def, "Option") {
                            let inner_ty = substs[0].expect_ty();
                            (true, self.convert_type(inner_ty, &field_context))
                        } else {
                            (false, self.convert_type(field_ty, &field_context))
                        }
                    } else {
                        (false, self.convert_type(field_ty, &field_context))
                    };

                TsField {
                    name: snake_to_camel(&field_name),
                    ty: actual_ty,
                    is_optional,
                }
            })
            .collect();

        TsType::Object(fields)
    }

    fn convert_enum(
        &mut self,
        _def_id: DefId,
        substs: GenericArgsRef<'tcx>,
        adt_def: &AdtDef<'tcx>,
        context: &str,
    ) -> TsType {
        let mut variants: Vec<TsType> = Vec::new();

        for variant in adt_def.variants() {
            let variant_ty = if variant.fields.is_empty() {
                TsType::Object(vec![TsField {
                    name: "t".to_string(),
                    ty: TsType::Literal(format!("\"{}\"", variant.name)),
                    is_optional: false,
                }])
            } else {
                let is_tuple_variant = variant.ctor_kind() == Some(CtorKind::Fn);
                let fields: Vec<TsField> = variant
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| {
                        let field_name = if is_tuple_variant {
                            format!("_{}", i)
                        } else {
                            self.tcx.item_name(field.did).to_string()
                        };
                        let field_ty = field.ty(self.tcx, substs);
                        let field_context = format!("{}::{}.{}", context, variant.name, field_name);

                        let (is_optional, actual_ty) =
                            if let rustc_middle::ty::TyKind::Adt(adt_def, substs) = field_ty.kind()
                            {
                                if self.is_std_type(adt_def, "Option") {
                                    let inner_ty = substs[0].expect_ty();
                                    (true, self.convert_type(inner_ty, &field_context))
                                } else {
                                    (false, self.convert_type(field_ty, &field_context))
                                }
                            } else {
                                (false, self.convert_type(field_ty, &field_context))
                            };

                        TsField {
                            name: snake_to_camel(&field_name),
                            ty: actual_ty,
                            is_optional,
                        }
                    })
                    .collect();

                let tag_field = TsField {
                    name: "t".to_string(),
                    ty: TsType::Literal(format!("\"{}\"", variant.name)),
                    is_optional: false,
                };

                // For Zod discriminated union, we need one flat object.
                // If it was a tuple variant in Rust, we usually map it to a property like "v" or spread it.
                // The prompt example shows: Ok { posts: ... } -> posts field directly.
                // So if it's struct-like variant, fields are spread.
                // If it's tuple-like variant, we usually wrap in "v" or similar.
                // The previous code did: Tag + (Tuple | Object).
                // But Zod discriminated union needs a SINGLE object with the tag inside.
                // So we must flatten.

                let mut flattened_fields = vec![tag_field];

                if is_tuple_variant {
                    // Tuple variant: e.g. Start(i32) -> { t: "Start", v: number } or { t: "Start", _0: number }
                    // Existing code logic was:
                    // value_field = { name: "v", ty: ... }
                    // We can stick to that convention: use "v" for the content of tuple variant.

                    if fields.len() == 1 {
                        flattened_fields.push(TsField {
                            name: "v".to_string(),
                            ty: fields[0].ty.clone(),
                            is_optional: fields[0].is_optional,
                        });
                    } else if !fields.is_empty() {
                        flattened_fields.push(TsField {
                            name: "v".to_string(),
                            ty: TsType::Tuple(fields.into_iter().map(|f| f.ty).collect()),
                            is_optional: false,
                        });
                    }
                } else {
                    // Struct variant: spread fields
                    flattened_fields.extend(fields);
                }

                TsType::Object(flattened_fields)
            };
            variants.push(variant_ty);
        }

        TsType::DiscriminatedUnion("t".to_string(), variants)
    }
}
