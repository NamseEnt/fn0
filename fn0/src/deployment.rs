use std::collections::HashMap;

type CodeId = String;
type DeploymentId = String;

pub struct Deployment {
    pub id: DeploymentId,
    pub codes: HashMap<CodeId, CodeManifest>,
}

pub struct CodeManifest {
    pub kind: CodeKind,
    /// Codes can communicate with each other using this ID like internal://<code_id>
    pub code_id: CodeId,
}

#[derive(Clone, Copy)]
pub enum CodeKind {
    Wasm,
    Js,
}

#[derive(Default)]
pub struct DeploymentMap {
    code_id_deployment_id_map: HashMap<CodeId, DeploymentId>,
    code_manifest_map: HashMap<CodeId, CodeManifest>,
}

impl DeploymentMap {
    pub fn new() -> Self {
        Self {
            code_id_deployment_id_map: Default::default(),
            code_manifest_map: Default::default(),
        }
    }

    pub fn register_code(&mut self, code_id: &str, kind: CodeKind) {
        self.code_id_deployment_id_map
            .insert(code_id.to_string(), "default".to_string());
        self.code_manifest_map.insert(
            code_id.to_string(),
            CodeManifest {
                kind,
                code_id: code_id.to_string(),
            },
        );
    }

    pub fn unregister_code(&mut self, code_id: &str) {
        self.code_id_deployment_id_map.remove(code_id);
        self.code_manifest_map.remove(code_id);
    }

    pub fn is_code_in_same_deployment(
        &self,
        code_id_a: &CodeId,
        code_id_b: &CodeId,
    ) -> Option<bool> {
        Some(
            self.code_id_deployment_id_map.get(code_id_a)?
                == self.code_id_deployment_id_map.get(code_id_b)?,
        )
    }

    pub fn code_kind(&self, code_id: &str) -> Option<CodeKind> {
        self.code_manifest_map
            .get(code_id)
            .map(|manifest| manifest.kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_unregister() {
        let mut map = DeploymentMap::new();

        map.register_code("app-1", CodeKind::Wasm);
        assert!(map.code_kind("app-1").is_some());

        map.unregister_code("app-1");
        assert!(map.code_kind("app-1").is_none());
    }

    #[test]
    fn unregister_nonexistent_does_not_panic() {
        let mut map = DeploymentMap::new();
        map.unregister_code("nonexistent");
    }

    #[test]
    fn re_register_after_unregister() {
        let mut map = DeploymentMap::new();

        map.register_code("app-1", CodeKind::Wasm);
        map.unregister_code("app-1");
        map.register_code("app-1", CodeKind::Js);

        assert!(matches!(map.code_kind("app-1"), Some(CodeKind::Js)));
    }

    #[test]
    fn unregister_preserves_other_codes() {
        let mut map = DeploymentMap::new();

        map.register_code("app-1", CodeKind::Wasm);
        map.register_code("app-2", CodeKind::Js);

        map.unregister_code("app-1");

        assert!(map.code_kind("app-1").is_none());
        assert!(map.code_kind("app-2").is_some());
    }

    #[test]
    fn same_deployment_after_unregister() {
        let mut map = DeploymentMap::new();

        map.register_code("a", CodeKind::Wasm);
        map.register_code("b", CodeKind::Wasm);
        assert_eq!(map.is_code_in_same_deployment(&"a".to_string(), &"b".to_string()), Some(true));

        map.unregister_code("a");
        assert_eq!(map.is_code_in_same_deployment(&"a".to_string(), &"b".to_string()), None);
    }
}
