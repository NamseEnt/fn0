pub fn generate() -> String {
    r#"import { defineConfig } from 'rolldown'

const nodeBuiltins = [
  'fs', 'path', 'crypto', 'buffer', 'stream', 'util', 'events',
  'child_process', 'os', 'http', 'https', 'net', 'tls', 'zlib',
  'url', 'querystring', 'string_decoder', 'assert', 'constants',
  'module', 'process', 'vm', 'cluster', 'dns', 'domain', 'dgram',
  'readline', 'repl', 'tty', 'v8', 'worker_threads', 'perf_hooks',
  'async_hooks', 'timers', 'inspector'
];

export default defineConfig({
  input: 'dist/server/entry.mjs',
  external: (id) => {
    if (id.startsWith('wasi:') || id.startsWith('node:')) return true;
    if (nodeBuiltins.includes(id)) return true;
    return false;
  },
  output: {
    file: 'dist/component.js',
    format: 'esm',
    inlineDynamicImports: true,
  },
  platform: 'browser',
})
"#
    .to_string()
}
