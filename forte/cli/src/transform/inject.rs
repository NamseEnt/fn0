pub fn inject_hmr_code(code: &str, module_id: &str, has_react_components: bool) -> String {
    let hmr_preamble = format!(
        r#"// HMR Preamble
const __MODULE_ID__ = "{}";
if (import.meta.hot === undefined) {{
  import.meta.hot = window.__forte_createHotContext(__MODULE_ID__);
}}
"#,
        module_id
    );

    // For React components, add self-accepting HMR
    let hmr_postamble = if has_react_components {
        r#"
// HMR Accept
if (import.meta.hot) {
  import.meta.hot.accept();
}
"#
        .to_string()
    } else {
        // Non-React modules also self-accept by default
        // This will cause their importers to re-execute
        r#"
// HMR Accept
if (import.meta.hot) {
  import.meta.hot.accept();
}
"#
        .to_string()
    };

    format!("{}{}\n{}", hmr_preamble, code, hmr_postamble)
}

pub fn get_react_refresh_preamble() -> &'static str {
    r#"// React Refresh Runtime Setup
import RefreshRuntime from '/.forte/deps/react-refresh-runtime.js';

RefreshRuntime.injectIntoGlobalHook(window);
window.$RefreshReg$ = () => {};
window.$RefreshSig$ = () => (type) => type;
window.$RefreshRuntime$ = RefreshRuntime;

// Will be set per-module
window.__currentModuleId = '';
"#
}

pub fn inject_react_refresh_code(code: &str, module_id: &str) -> String {
    let preamble = format!(
        r#"// React Refresh Per-Module Setup
window.__currentModuleId = "{}";
var prevRefreshReg = window.$RefreshReg$;
var prevRefreshSig = window.$RefreshSig$;
window.$RefreshReg$ = (type, id) => {{
  window.$RefreshRuntime$.register(type, "{}" + " " + id);
}};
window.$RefreshSig$ = window.$RefreshRuntime$.createSignatureFunctionForTransform;

"#,
        module_id, module_id
    );

    let postamble = r#"
// React Refresh Cleanup
window.$RefreshReg$ = prevRefreshReg;
window.$RefreshSig$ = prevRefreshSig;

// Schedule refresh
if (import.meta.hot) {
  import.meta.hot.accept();
  if (window.$RefreshRuntime$ && window.$RefreshRuntime$.performReactRefresh) {
    window.$RefreshRuntime$.performReactRefresh();
  }
}
"#;

    format!("{}{}\n{}", preamble, code, postamble)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_hmr_code() {
        let code = "export function App() { return <div>Hello</div>; }";
        let result = inject_hmr_code(code, "/src/App.tsx", true);

        assert!(result.contains("__MODULE_ID__"));
        assert!(result.contains("/src/App.tsx"));
        assert!(result.contains("import.meta.hot.accept()"));
    }
}
