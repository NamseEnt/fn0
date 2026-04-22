use std::fmt;

#[derive(Debug, Clone)]
pub enum TsType {
    Literal(String),
    Primitive(String),
    Date,
    Array(Box<TsType>),
    Tuple(Vec<TsType>),
    DiscriminatedUnion(String, Vec<TsType>),
    Object(Vec<TsField>),
    Record(Box<TsType>),
    Undefined(Box<TsType>),
    Reference(String),
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

// Helper to generate Zod schema string
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
    }
}

// Display implementation kept for debugging or fallback types
impl fmt::Display for TsType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TsType::Literal(s) => write!(f, "{s}"),
            TsType::Primitive(s) => write!(f, "{s}"),
            TsType::Date => write!(f, "Date"),
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
