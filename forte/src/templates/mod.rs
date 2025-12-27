pub mod error_boundary;
pub mod frontend_ssr;

// Root configuration files

pub const FORTE_TOML: &str = r#"[env]
required = ["DATABASE_URL"]
optional = ["REDIS_URL", "LOG_LEVEL"]

[env.defaults]
LOG_LEVEL = "info"
PORT = "3000"
RUST_PORT = "8080"

[type_mappings]
"chrono::DateTime<Utc>" = "string"
"chrono::NaiveDate" = "string"
"uuid::Uuid" = "string"

[proxy]
forward_headers = ["Cookie", "Authorization", "Accept-Language"]
timeout_ms = 5000

[build]
output_dir = "dist"
"#;

pub const ENV_TEMPLATE: &str = r#"# Add your environment variables here
# This file should be in .gitignore
DATABASE_URL=
"#;

pub const ENV_DEV: &str = r#"# Development environment variables
DATABASE_URL=postgres://localhost/myapp_dev
LOG_LEVEL=debug
PORT=3000
RUST_PORT=8080
"#;

pub const ENV_PROD: &str = r#"# Production environment variables
DATABASE_URL=
LOG_LEVEL=info
PORT=3000
RUST_PORT=8080
"#;

pub const GITIGNORE: &str = r#"# Rust
target/
Cargo.lock

# Node
node_modules/
dist/
.vite/

# Environment
.env
.env.local

# Generated
.generated/

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db
"#;

pub fn readme(project_name: &str) -> String {
    format!(
        r#"# {}

A Forte project - Full-stack Rust+React with type-safe routing.

## Getting Started

```bash
# Start development server
forte dev

# Build for production
forte build

# Run tests
forte test
```

## Project Structure

- `backend/` - Rust backend with Axum
- `frontend/` - React frontend with Vite
- `.generated/` - Auto-generated code (do not edit)

## Adding a New Page

1. Create a new route in `backend/src/routes/`:
   ```
   backend/src/routes/about/props.rs
   ```

2. Define your data types:
   ```rust
   pub struct PageProps {{
       pub message: String,
   }}

   pub async fn get_props() -> PageProps {{
       PageProps {{
           message: "Hello from Forte!".into(),
       }}
   }}
   ```

3. Create the UI in `frontend/src/app/`:
   ```tsx
   // frontend/src/app/about/page.tsx
   import type {{ PageProps }} from "./props.gen";

   export default function AboutPage({{ message }}: PageProps) {{
       return <h1>{{message}}</h1>;
   }}
   ```

4. The CLI will automatically:
   - Generate TypeScript types
   - Create routing configuration
   - Hot reload your changes

## Learn More

See the [Forte documentation](https://github.com/yourusername/forte) for more details.
"#,
        project_name
    )
}

// Backend templates

pub fn backend_cargo_toml(_project_name: &str) -> String {
    r#"[package]
name = "backend"
version = "0.1.0"
edition = "2021"

[lib]
name = "backend"
path = "src/lib.rs"

[[bin]]
name = "backend"
path = "../.generated/backend/main.rs"

[dependencies]
axum = "0.7"
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
validator = { version = "0.18", features = ["derive"] }
"#.to_string()
}

pub const BACKEND_LIB: &str = r#"// Library crate for backend routes
// This is used by the generated binary in .generated/backend/main.rs

pub mod routes;
"#;

pub const BACKEND_ROUTES_MOD: &str = r#"// [Generated] Do not edit manually
// This file will be managed by the Forte CLI

pub mod index;
"#;

pub const BACKEND_INDEX_MOD: &str = r#"pub mod props;

pub use props::*;
"#;

pub const BACKEND_INDEX_PROPS: &str = r#"use serde::Serialize;

#[derive(Serialize)]
pub struct PageProps {
    pub message: String,
}

pub async fn get_props() -> PageProps {
    PageProps {
        message: "Welcome to Forte!".into(),
    }
}
"#;

// Frontend templates

pub fn frontend_package_json(project_name: &str) -> String {
    format!(
        r#"{{
  "name": "{}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "tsx server.ts",
    "build": "vite build && vite build --ssr",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "express": "^4.18.0",
    "@forte/runtime": "file:src/forte"
  }},
  "devDependencies": {{
    "@types/express": "^4.17.0",
    "@types/node": "^20.10.0",
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "@vitejs/plugin-react": "^4.2.0",
    "typescript": "^5.3.0",
    "vite": "^5.0.0",
    "tsx": "^4.7.0",
    "esbuild": "^0.19.0"
  }}
}}
"#,
        project_name
    )
}

pub const FRONTEND_TSCONFIG: &str = r#"{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
"#;

pub const FRONTEND_VITE_CONFIG: &str = r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 3000,
  },
})
"#;

pub const FRONTEND_SERVER_WRAPPER: &str = r#"// [Generated] Server entry point - imports the generated SSR server
// This file exists in the frontend directory to ensure proper module resolution
import '../.generated/frontend/server.ts';
"#;

pub const FRONTEND_ROOT_LAYOUT: &str = r#"import * as React from 'react';

interface LayoutProps {
  children: React.ReactNode;
}

export default function RootLayout({ children }: LayoutProps) {
  return (
    <html lang="en">
      <head>
        <meta charSet="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>Forte App</title>
      </head>
      <body>
        <div id="root">
          <main>{children}</main>
        </div>
      </body>
    </html>
  );
}
"#;

pub const FRONTEND_INDEX_PAGE: &str = r#"import * as React from 'react';
import type { PageProps } from "./props.gen";

export default function IndexPage({ message }: PageProps) {
  return (
    <div>
      <h1>{message}</h1>
      <p>Edit backend/src/routes/index/props.rs to change this message.</p>
    </div>
  );
}
"#;

// Forte Runtime Library Templates

pub const FORTE_RUNTIME_PACKAGE_JSON: &str = r#"{
  "name": "@forte/runtime",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "main": "index.ts",
  "types": "index.ts"
}
"#;

pub const FORTE_RUNTIME_INDEX: &str = r#"// Forte Runtime Library
// Auto-generated client-side utilities

export { RouterProvider, useRouter } from './Router';
export { Form } from './Form';
export { Link } from './Link';
"#;

pub const FORTE_RUNTIME_FORM: &str = r#"import * as React from 'react';

interface FormProps extends React.FormHTMLAttributes<HTMLFormElement> {
  children: React.ReactNode;
}

/**
 * Forte Form component
 * Handles POST actions with progressive enhancement
 */
export function Form({ children, action, method = 'POST', ...props }: FormProps) {
  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    // For now, use default form submission
    // Future: client-side fetch with optimistic updates
  };

  return (
    <form
      action={action}
      method={method}
      onSubmit={handleSubmit}
      {...props}
    >
      {children}
    </form>
  );
}
"#;

pub const FORTE_RUNTIME_ROUTER: &str = r#"import * as React from 'react';

interface RouterContextValue {
  navigate: (href: string) => Promise<void>;
  prefetch: (href: string) => Promise<void>;
  currentPath: string;
}

const RouterContext = React.createContext<RouterContextValue | null>(null);

export function useRouter() {
  const context = React.useContext(RouterContext);
  const isServer = typeof window === 'undefined';

  if (!context) {
    // On server, return no-op router
    if (isServer) {
      return {
        navigate: async () => {},
        prefetch: async () => {},
        currentPath: '/'
      };
    }
    throw new Error('useRouter must be used within a RouterProvider');
  }
  return context;
}

interface RouterProviderProps {
  children: React.ReactNode;
  initialPath?: string;
}

export function RouterProvider({ children, initialPath }: RouterProviderProps) {
  const isServer = typeof window === 'undefined';
  const [currentPath, setCurrentPath] = React.useState(
    initialPath || (!isServer ? window.location.pathname : '/')
  );
  const prefetchCache = React.useRef<Map<string, any>>(new Map());

  const prefetch = React.useCallback(async (href: string) => {
    // Skip on server
    if (isServer) return;

    // Skip if already in cache
    if (prefetchCache.current.has(href)) {
      return;
    }

    try {
      const response = await fetch(href, {
        headers: { 'Accept': 'application/json' }
      });

      if (response.ok) {
        const data = await response.json();
        prefetchCache.current.set(href, data);
      }
    } catch (err) {
      // Ignore prefetch errors
      console.warn('[Forte Router] Prefetch failed for:', href, err);
    }
  }, [isServer]);

  const navigate = React.useCallback(async (href: string) => {
    // Skip on server
    if (isServer) return;

    try {
      // Check cache first
      let pageProps = prefetchCache.current.get(href);

      if (!pageProps) {
        // Fetch from server
        const response = await fetch(href, {
          headers: { 'Accept': 'application/json' }
        });

        if (!response.ok) {
          // Fall back to full page navigation on error
          window.location.href = href;
          return;
        }

        pageProps = await response.json();
      }

      // Update URL without reload
      window.history.pushState({ path: href }, '', href);

      // Update current path
      setCurrentPath(href);

      // Trigger custom event for page updates
      window.dispatchEvent(new CustomEvent('forte:navigate', {
        detail: { href, pageProps }
      }));

      // Clear cache entry after use
      prefetchCache.current.delete(href);
    } catch (err) {
      console.error('[Forte Router] Navigation failed:', err);
      // Fall back to full page navigation
      window.location.href = href;
    }
  }, [isServer]);

  // Handle browser back/forward
  React.useEffect(() => {
    // Skip on server
    if (isServer) return;

    const handlePopState = () => {
      setCurrentPath(window.location.pathname);
      window.dispatchEvent(new CustomEvent('forte:navigate', {
        detail: { href: window.location.pathname }
      }));
    };

    window.addEventListener('popstate', handlePopState);
    return () => window.removeEventListener('popstate', handlePopState);
  }, [isServer]);

  const value = React.useMemo(() => ({
    navigate,
    prefetch,
    currentPath
  }), [navigate, prefetch, currentPath]);

  return (
    <RouterContext.Provider value={value}>
      {children}
    </RouterContext.Provider>
  );
}
"#;

pub const FORTE_RUNTIME_LINK: &str = r#"import * as React from 'react';
import { useRouter } from './Router';

interface LinkProps extends React.AnchorHTMLAttributes<HTMLAnchorElement> {
  href: string;
  children: React.ReactNode;
  prefetch?: boolean;
}

/**
 * Forte Link component
 * Client-side navigation with progressive enhancement
 */
export function Link({ href, children, prefetch = false, onClick, ...props }: LinkProps) {
  const router = useRouter();
  const [isPrefetching, setIsPrefetching] = React.useState(false);

  const handleClick = async (e: React.MouseEvent<HTMLAnchorElement>) => {
    // Allow default behavior if:
    // - Command/Ctrl click (open in new tab)
    // - Middle mouse button click
    // - Target is set (e.g., target="_blank")
    if (
      e.ctrlKey ||
      e.metaKey ||
      e.button !== 0 ||
      props.target
    ) {
      return;
    }

    // Prevent default navigation
    e.preventDefault();

    // Call custom onClick if provided
    if (onClick) {
      onClick(e);
    }

    // Navigate using client-side routing
    await router.navigate(href);
  };

  const handleMouseEnter = () => {
    if (prefetch && !isPrefetching) {
      setIsPrefetching(true);
      router.prefetch(href).catch(() => {
        // Ignore prefetch errors
      });
    }
  };

  return (
    <a
      href={href}
      onClick={handleClick}
      onMouseEnter={handleMouseEnter}
      {...props}
    >
      {children}
    </a>
  );
}
"#;
