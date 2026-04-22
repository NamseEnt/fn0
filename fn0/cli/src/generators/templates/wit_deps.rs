use std::collections::HashMap;

pub struct WitDeps;

impl WitDeps {
    pub fn get_all_deps() -> HashMap<String, String> {
        let mut deps = HashMap::new();

        deps.insert(
            "wasi-cli-0.2.6/package.wit".to_string(),
            Self::wasi_cli_0_2_6(),
        );
        deps.insert(
            "wasi-clocks-0.2.6/package.wit".to_string(),
            Self::wasi_clocks_0_2_6(),
        );
        deps.insert(
            "wasi-http-0.2.6/package.wit".to_string(),
            Self::wasi_http_0_2_6(),
        );
        deps.insert(
            "wasi-io-0.2.6/package.wit".to_string(),
            Self::wasi_io_0_2_6(),
        );
        deps.insert(
            "wasi-random-0.2.6/package.wit".to_string(),
            Self::wasi_random_0_2_6(),
        );

        deps
    }

    fn wasi_cli_0_2_6() -> String {
        include_str!("wit_deps_content/wasi-cli-0.2.6.wit").to_string()
    }

    fn wasi_clocks_0_2_6() -> String {
        include_str!("wit_deps_content/wasi-clocks-0.2.6.wit").to_string()
    }

    fn wasi_http_0_2_6() -> String {
        include_str!("wit_deps_content/wasi-http-0.2.6.wit").to_string()
    }

    fn wasi_io_0_2_6() -> String {
        include_str!("wit_deps_content/wasi-io-0.2.6.wit").to_string()
    }

    fn wasi_random_0_2_6() -> String {
        include_str!("wit_deps_content/wasi-random-0.2.6.wit").to_string()
    }
}
