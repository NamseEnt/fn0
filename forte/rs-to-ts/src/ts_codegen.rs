use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

#[derive(Debug, Clone)]
pub enum TsType {
    Literal(String),
    Primitive(String),
    Date,
    JsonValue,
    Array(Box<TsType>),
    Tuple(Vec<TsType>),
    DiscriminatedUnion(String, Vec<TsType>),
    Object(Vec<TsField>),
    Record(Box<TsType>),
    Undefined(Box<TsType>),
    Reference(String),
    LazyReference(String),
}

#[derive(Debug, Clone)]
pub struct TsField {
    pub name: String,
    pub ty: TsType,
    pub is_optional: bool,
}

pub fn strip_undefined(ty: &TsType) -> String {
    match ty {
        TsType::Undefined(inner) => format!("{}", inner),
        _ => format!("{}", ty),
    }
}

pub fn to_zod(ty: &TsType) -> String {
    match ty {
        TsType::Literal(s) => format!("z.literal({s})"),
        TsType::Primitive(s) => match s.as_str() {
            "string" => "z.string()".to_string(),
            "number" => "z.number()".to_string(),
            "boolean" => "z.boolean()".to_string(),
            _ => panic!("Unexpected primitive type: {s}"),
        },
        TsType::Date => "z.coerce.date()".to_string(),
        TsType::JsonValue => "z.json()".to_string(),
        TsType::Array(inner) => format!("z.array({})", to_zod(inner)),
        TsType::Tuple(types) => {
            let elems = types.iter().map(to_zod).collect::<Vec<_>>().join(", ");
            format!("z.tuple([{elems}])",)
        }
        TsType::DiscriminatedUnion(tag, variants) => {
            let elems = variants
                .iter()
                .map(to_zod)
                .collect::<Vec<_>>()
                .join(",\n    ");
            format!("z.discriminatedUnion(\"{tag}\", [\n    {elems}\n  ])")
        }
        TsType::Object(fields) => {
            let mut s = "z.object({\n".to_string();
            for field in fields {
                let base_zod = to_zod(&field.ty);
                let final_zod = if field.is_optional {
                    format!("{base_zod}.optional()",)
                } else {
                    base_zod
                };
                s.push_str(&format!("    {}: {},\n", field.name, final_zod));
            }
            s.push_str("  })");
            s
        }
        TsType::Record(inner) => format!("z.record(z.string(), {})", to_zod(inner)),
        TsType::Undefined(inner) => format!("{}.optional()", to_zod(inner)),
        TsType::Reference(name) => format!("{name}Schema"),
        TsType::LazyReference(name) => format!("z.lazy(() => {name}Schema)"),
    }
}

impl fmt::Display for TsType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TsType::Literal(s) => write!(f, "{s}"),
            TsType::Primitive(s) => write!(f, "{s}"),
            TsType::Date => write!(f, "Date"),
            TsType::JsonValue => write!(f, "unknown"),
            TsType::Array(inner) => write!(f, "{inner}[]"),
            TsType::Tuple(types) => {
                write!(f, "[")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{ty}")?;
                }
                write!(f, "]")
            }
            TsType::DiscriminatedUnion(_, variants) => {
                for (i, ty) in variants.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{ty}")?;
                }
                Ok(())
            }
            TsType::Object(fields) => {
                write!(f, "{{ ")?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    let optional_marker = if field.is_optional { "?" } else { "" };
                    let ty_str = if field.is_optional {
                        strip_undefined(&field.ty)
                    } else {
                        format!("{}", field.ty)
                    };
                    write!(f, "{}{}: {}", field.name, optional_marker, ty_str)?;
                }
                write!(f, " }}")
            }
            TsType::Record(inner) => write!(f, "Record<string, {}>", inner),
            TsType::Undefined(inner) => write!(f, "{} | undefined", inner),
            TsType::Reference(name) => write!(f, "{name}"),
            TsType::LazyReference(name) => write!(f, "{name}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TsDefinition {
    pub full_path: String,
    pub namespace: Vec<String>,
    pub type_name: String,
    pub ty: TsType,
}

pub fn format_definition(def: &TsDefinition) -> String {
    let zod_schema = to_zod(&def.ty);
    format!(
        "export const {}Schema = {};\n\nexport type {} = z.infer<typeof {}Schema>;",
        def.type_name, zod_schema, def.type_name, def.type_name
    )
}

fn emitted_name(def: &TsDefinition) -> String {
    if def.namespace.is_empty() {
        def.type_name.clone()
    } else {
        format!("{}.{}", def.namespace.join("."), def.type_name)
    }
}

fn collect_references(ty: &TsType, references: &mut Vec<String>) {
    match ty {
        TsType::Reference(name) | TsType::LazyReference(name) => references.push(name.clone()),
        TsType::Array(inner) => collect_references(inner, references),
        TsType::Tuple(types) => {
            for inner in types {
                collect_references(inner, references);
            }
        }
        TsType::DiscriminatedUnion(_, variants) => {
            for variant in variants {
                collect_references(variant, references);
            }
        }
        TsType::Object(fields) => {
            for field in fields {
                collect_references(&field.ty, references);
            }
        }
        TsType::Undefined(inner) => collect_references(inner, references),
        TsType::Record(inner) => collect_references(inner, references),
        TsType::Literal(_) | TsType::Primitive(_) | TsType::Date | TsType::JsonValue => {}
    }
}

fn rewrite_references(ty: &mut TsType, should_lazy: &dyn Fn(&str) -> bool) {
    match ty {
        TsType::Reference(name) => {
            if should_lazy(name) {
                *ty = TsType::LazyReference(name.clone());
            }
        }
        TsType::Array(inner) => rewrite_references(inner, should_lazy),
        TsType::Tuple(types) => {
            for inner in types {
                rewrite_references(inner, should_lazy);
            }
        }
        TsType::DiscriminatedUnion(_, variants) => {
            for variant in variants {
                rewrite_references(variant, should_lazy);
            }
        }
        TsType::Object(fields) => {
            for field in fields {
                rewrite_references(&mut field.ty, should_lazy);
            }
        }
        TsType::Undefined(inner) => rewrite_references(inner, should_lazy),
        TsType::Record(inner) => rewrite_references(inner, should_lazy),
        TsType::Literal(_) | TsType::Primitive(_) | TsType::Date | TsType::JsonValue => {}
        TsType::LazyReference(_) => {}
    }
}

// Orders definitions so every emitted schema references only schemas declared
// before it, and rewrites the references that cannot be ordered that way
// (mutual recursion, cross-namespace edges, unknown names) into `z.lazy`.
pub fn order_definitions(definitions: &mut Vec<TsDefinition>, roots: &mut [&mut TsType]) {
    let names: Vec<String> = definitions.iter().map(emitted_name).collect();
    let name_to_index: HashMap<&str, usize> = names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.as_str(), index))
        .collect();

    let mut group_ids: Vec<usize> = Vec::with_capacity(definitions.len());
    let mut groups: Vec<(Vec<String>, Vec<usize>)> = Vec::new();
    for (index, def) in definitions.iter().enumerate() {
        let group_id = groups
            .iter()
            .position(|(namespace, _)| *namespace == def.namespace)
            .unwrap_or_else(|| {
                groups.push((def.namespace.clone(), Vec::new()));
                groups.len() - 1
            });
        groups[group_id].1.push(index);
        group_ids.push(group_id);
    }

    let mut sorted_members: Vec<Vec<usize>> = groups.iter().map(|_| Vec::new()).collect();
    let mut cycle_names: HashSet<String> = HashSet::new();

    for (group_id, (_, members)) in groups.iter().enumerate() {
        let member_set: HashSet<usize> = members.iter().copied().collect();
        let mut indegree: HashMap<usize, usize> =
            members.iter().map(|&member| (member, 0)).collect();
        let mut dependents: HashMap<usize, Vec<usize>> = HashMap::new();

        for &member in members {
            let mut references: Vec<String> = Vec::new();
            collect_references(&definitions[member].ty, &mut references);
            let mut seen: HashSet<String> = HashSet::new();
            for reference in references {
                if !seen.insert(reference.clone()) {
                    continue;
                }
                let Some(&target) = name_to_index.get(reference.as_str()) else {
                    continue;
                };
                if member_set.contains(&target) {
                    dependents.entry(target).or_default().push(member);
                    *indegree.entry(member).or_default() += 1;
                }
            }
        }

        let mut queue: VecDeque<usize> = members
            .iter()
            .copied()
            .filter(|&member| indegree[&member] == 0)
            .collect();
        let mut remaining: HashSet<usize> = member_set;
        while let Some(member) = queue.pop_front() {
            remaining.remove(&member);
            sorted_members[group_id].push(member);
            if let Some(dependents_of_member) = dependents.get(&member) {
                for &dependent in dependents_of_member {
                    if remaining.contains(&dependent) {
                        let degree = indegree.get_mut(&dependent).unwrap();
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent);
                        }
                    }
                }
            }
        }
        for &member in members {
            if remaining.contains(&member) {
                sorted_members[group_id].push(member);
                cycle_names.insert(names[member].clone());
            }
        }
    }

    let name_to_group: HashMap<&str, usize> = names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.as_str(), group_ids[index]))
        .collect();

    for (index, def) in definitions.iter_mut().enumerate() {
        let referrer_group = group_ids[index];
        rewrite_references(&mut def.ty, &|target| match name_to_group.get(target) {
            None => true,
            Some(&target_group) => target_group != referrer_group || cycle_names.contains(target),
        });
    }
    for root in roots {
        rewrite_references(root, &|target| match name_to_group.get(target) {
            None => true,
            Some(_) => cycle_names.contains(target),
        });
    }

    let mut reordered: Vec<TsDefinition> = Vec::with_capacity(definitions.len());
    for members in sorted_members.iter() {
        for &member in members {
            reordered.push(definitions[member].clone());
        }
    }
    *definitions = reordered;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition(type_name: &str, namespace: &[&str], ty: TsType) -> TsDefinition {
        TsDefinition {
            full_path: format!("crate::{type_name}"),
            namespace: namespace.iter().map(|s| s.to_string()).collect(),
            type_name: type_name.to_string(),
            ty,
        }
    }

    fn reference(name: &str) -> TsType {
        TsType::Reference(name.to_string())
    }

    fn object_field(name: &str, ty: TsType) -> TsField {
        TsField {
            name: name.to_string(),
            ty,
            is_optional: false,
        }
    }

    #[test]
    fn json_value_renders_as_z_json() {
        assert_eq!(to_zod(&TsType::JsonValue), "z.json()");
    }

    #[test]
    fn lazy_reference_renders_as_z_lazy() {
        assert_eq!(
            to_zod(&TsType::LazyReference("Value".to_string())),
            "z.lazy(() => ValueSchema)"
        );
    }

    #[test]
    fn definitions_are_reordered_by_dependency() {
        let mut definitions = vec![
            definition(
                "A",
                &[],
                TsType::Object(vec![object_field("b", reference("B"))]),
            ),
            definition("B", &[], TsType::Primitive("string".to_string())),
        ];
        let mut root = reference("A");
        order_definitions(&mut definitions, std::slice::from_mut(&mut &mut root));

        assert_eq!(definitions[0].type_name, "B");
        assert_eq!(definitions[1].type_name, "A");
        assert_eq!(
            format_definition(&definitions[1]),
            "export const ASchema = z.object({\n    b: BSchema,\n  });\n\nexport type A = z.infer<typeof ASchema>;"
        );
    }

    #[test]
    fn cyclic_definitions_use_lazy_references() {
        let mut definitions = vec![
            definition(
                "A",
                &[],
                TsType::Object(vec![object_field("b", reference("B"))]),
            ),
            definition(
                "B",
                &[],
                TsType::Object(vec![object_field("a", reference("A"))]),
            ),
        ];
        let mut root = reference("A");
        order_definitions(&mut definitions, std::slice::from_mut(&mut &mut root));

        assert!(format_definition(&definitions[0]).contains("z.lazy(() => BSchema)"));
        assert!(format_definition(&definitions[1]).contains("z.lazy(() => ASchema)"));
    }

    #[test]
    fn cross_namespace_references_become_lazy() {
        let mut definitions = vec![
            definition(
                "Top",
                &[],
                TsType::Object(vec![object_field("sub", reference("ns.Sub"))]),
            ),
            definition("Sub", &["ns"], TsType::Primitive("string".to_string())),
        ];
        let mut root = reference("Top");
        order_definitions(&mut definitions, std::slice::from_mut(&mut &mut root));

        assert_eq!(
            to_zod(&definitions[0].ty),
            "z.object({\n    sub: z.lazy(() => ns.SubSchema),\n  })"
        );
    }

    #[test]
    fn missing_references_become_lazy() {
        let mut definitions = vec![definition(
            "A",
            &[],
            TsType::Object(vec![object_field("ghost", reference("Ghost"))]),
        )];
        let mut root = reference("A");
        order_definitions(&mut definitions, std::slice::from_mut(&mut &mut root));

        assert_eq!(
            to_zod(&definitions[0].ty),
            "z.object({\n    ghost: z.lazy(() => GhostSchema),\n  })"
        );
    }
}
