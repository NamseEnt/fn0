use doc_db::Database;

const CONTROL_PROJECT_ID: &str = "fn0-control";

pub fn build() -> Database {
    let dodb_configured = dodb_configured(
        std::env::var_os("DODB_ADDR").is_some(),
        std::env::var_os("DODB_SERVER_NAME").is_some(),
        std::env::var_os("DODB_ROOT_CERT_PEM_BASE64").is_some(),
    );
    if dodb_configured {
        let config = doc_db::DodbConfig::from_env().expect("DODB config is invalid");
        let connection = doc_db::DodbConnection::connect_lazy(&config)
            .expect("DODB connection configuration is invalid");
        return doc_db::dodb_with_connection(&connection, CONTROL_PROJECT_ID)
            .expect("fn0-control dodb tenant is valid");
    }
    tracing::warn!("using legacy Turso control database");
    let group_token = std::env::var("TURSO_GROUP_TOKEN").expect("TURSO_GROUP_TOKEN must be set");
    let host_suffix =
        std::env::var("TURSO_DB_HOST_SUFFIX").expect("TURSO_DB_HOST_SUFFIX must be set");
    let url = format!("https://{CONTROL_PROJECT_ID}{host_suffix}");
    doc_db::turso_with_config(url, group_token)
}

fn dodb_configured(address: bool, server_name: bool, root_certificate: bool) -> bool {
    address || server_name || root_certificate
}

#[cfg(test)]
mod tests {
    use super::dodb_configured;

    #[test]
    fn dodb_configuration_selects_dodb_and_absence_selects_legacy_fallback() {
        assert!(!dodb_configured(false, false, false));
        assert!(dodb_configured(true, true, true));
        assert!(dodb_configured(true, false, false));
    }
}
