pub fn generate() -> String {
    r#"package example:hono;

world component {
    export wasi:http/incoming-handler@0.2.6;
}
"#.to_string()
}
