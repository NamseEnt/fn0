use crate::config::ProjectConfig;
use serde_json::json;

pub fn generate(config: &ProjectConfig) -> String {
    let ProjectConfig {
        name,
        package_manager,
        ..
    } = config;
    let package_json = json!({
        "name": name,
        "type": "module",
        "scripts": {
            "dev": format!("{package_manager} run src/index.ts"),
            "build": "rolldown -c && jco componentize -w wit -o dist/component.wasm dist/component.js"
        },
    });

    serde_json::to_string_pretty(&package_json).unwrap()
}
