use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Deployment {
    Wasm,
    Forte,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CodeKind {
    Wasm,
    Js,
}

#[derive(Default)]
pub struct DeploymentMap {
    deployments: HashMap<String, Deployment>,
    artifacts: HashMap<String, CodeKind>,
}

impl DeploymentMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_deployment(&mut self, code_id: &str, deployment: Deployment) {
        match &deployment {
            Deployment::Wasm => {
                self.artifacts
                    .insert(format!("{code_id}::backend"), CodeKind::Wasm);
            }
            Deployment::Forte => {
                self.artifacts
                    .insert(format!("{code_id}::backend"), CodeKind::Wasm);
                self.artifacts
                    .insert(format!("{code_id}::frontend"), CodeKind::Js);
            }
        }
        self.deployments.insert(code_id.to_string(), deployment);
    }

    pub fn unregister_deployment(&mut self, code_id: &str) {
        if let Some(deployment) = self.deployments.remove(code_id) {
            match deployment {
                Deployment::Wasm => {
                    self.artifacts.remove(&format!("{code_id}::backend"));
                }
                Deployment::Forte => {
                    self.artifacts.remove(&format!("{code_id}::backend"));
                    self.artifacts.remove(&format!("{code_id}::frontend"));
                }
            }
        }
    }

    pub fn deployment(&self, code_id: &str) -> Option<Deployment> {
        self.deployments.get(code_id).cloned()
    }

    #[allow(dead_code)]
    pub(crate) fn artifact_kind(&self, artifact_id: &str) -> Option<CodeKind> {
        self.artifacts.get(artifact_id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_query_wasm() {
        let mut map = DeploymentMap::new();
        map.register_deployment("app-1", Deployment::Wasm);

        assert!(matches!(map.deployment("app-1"), Some(Deployment::Wasm)));
        assert_eq!(map.artifact_kind("app-1::backend"), Some(CodeKind::Wasm));
    }

    #[test]
    fn register_and_query_forte() {
        let mut map = DeploymentMap::new();
        map.register_deployment("app-1", Deployment::Forte);

        assert!(matches!(map.deployment("app-1"), Some(Deployment::Forte)));
        assert_eq!(map.artifact_kind("app-1::backend"), Some(CodeKind::Wasm));
        assert_eq!(map.artifact_kind("app-1::frontend"), Some(CodeKind::Js));
    }

    #[test]
    fn unregister_removes_all_artifacts() {
        let mut map = DeploymentMap::new();
        map.register_deployment("app-1", Deployment::Forte);
        map.unregister_deployment("app-1");

        assert!(map.deployment("app-1").is_none());
        assert!(map.artifact_kind("app-1::backend").is_none());
        assert!(map.artifact_kind("app-1::frontend").is_none());
    }

    #[test]
    fn re_register_switches_kind() {
        let mut map = DeploymentMap::new();
        map.register_deployment("app-1", Deployment::Wasm);
        map.unregister_deployment("app-1");
        map.register_deployment("app-1", Deployment::Forte);

        assert!(matches!(map.deployment("app-1"), Some(Deployment::Forte)));
        assert_eq!(map.artifact_kind("app-1::backend"), Some(CodeKind::Wasm));
        assert_eq!(map.artifact_kind("app-1::frontend"), Some(CodeKind::Js));
    }
}
