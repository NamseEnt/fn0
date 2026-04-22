use schemars::{
    schema::{InstanceType, SchemaObject, SingleOrVec},
    schema_for,
};

#[path = "src/args.rs"]
#[allow(dead_code)]
mod args;

fn main() {
    let schema = schema_for!(args::HqArgs);

    let mut output = "import * as pulumi from '@pulumi/pulumi';\n".to_string();

    write_schema(&mut output, "HqArgs", schema.schema);
    for (name, def) in schema.definitions {
        write_schema(&mut output, &name, def.into_object());
    }

    let out_path = std::path::Path::new("../../infra/pulumi/hqArgs.schema.ts");
    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(out_path, output);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/args.rs");
}

fn write_schema(output: &mut String, name: &str, schema: SchemaObject) {
    *output += &format!("export interface {name} {{\n");
    if let Some(object) = schema.object {
        let required: std::collections::HashSet<_> = object.required.iter().cloned().collect();
        for (name, schema) in object.properties {
            let optional = !required.contains(&name);
            let opt_marker = if optional { "?" } else { "" };
            *output += &format!("  {name}{opt_marker}: pulumi.Input<");
            *output += &name_of_properties(schema.into_object());
            *output += ">;\n";
        }
    } else if let Some(subschema) = schema.subschemas {
        if let Some(one_of) = subschema.one_of {
            for schema in one_of {
                let object = schema.into_object().object.unwrap();
                for (name, schema) in object.properties {
                    *output += &format!("  {name}?: pulumi.Input<");
                    *output += &name_of_properties(schema.into_object());
                    *output += ">;\n";
                }
            }
        } else {
            todo!("{name} -> {:?}", subschema)
        }
    } else {
        todo!("{name} -> {:?}", schema)
    }
    *output += "}\n";
}

fn name_of_properties(schema: SchemaObject) -> String {
    if let Some(ref subschemas) = schema.subschemas {
        if let Some(ref any_of) = subschemas.any_of {
            for s in any_of {
                let obj = s.clone().into_object();
                if obj.instance_type == Some(SingleOrVec::Single(Box::new(InstanceType::Null))) {
                    continue;
                }
                return name_of_properties(obj);
            }
        }
    }
    if let Some(array) = schema.array {
        match array.items.unwrap() {
            SingleOrVec::Single(single) => {
                return format!("Array<{}>", name_of_properties(single.into_object()));
            }
            SingleOrVec::Vec(_items) => todo!(),
        }
    } else if let Some(reference) = schema.reference {
        return reference.split('/').next_back().unwrap().to_string();
    } else if let Some(object) = schema.object {
        if let Some(additional_properties) = object.additional_properties {
            return format!(
                "Record<string, {}>",
                name_of_properties(additional_properties.into_object())
            );
        } else {
            todo!("{:?}", object);
        }
    } else if let Some(instance_type) = schema.instance_type {
        match instance_type {
            SingleOrVec::Single(single) => return name_of_instance_type(&single),
            SingleOrVec::Vec(_items) => todo!(),
        }
    } else if schema.string.is_some() {
        return "string".to_string();
    } else if schema.number.is_some() {
        return "number".to_string();
    }

    todo!("{:?}", schema)
}

fn name_of_instance_type(instance_type: &InstanceType) -> String {
    match instance_type {
        InstanceType::Null => "null".to_string(),
        InstanceType::Boolean => "bool".to_string(),
        InstanceType::Object => {
            todo!()
        }
        InstanceType::Array => "array".to_string(),
        InstanceType::Number => "number".to_string(),
        InstanceType::String => "string".to_string(),
        InstanceType::Integer => "number".to_string(),
    }
}
