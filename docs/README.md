# fn0 / Forte Documentation

## Getting Started

- [Setup](setup.md) — Prerequisites, installation, environment variables

## Forte Framework

Forte is the full-stack web framework built on fn0.

- [Overview](forte/overview.md) — Architecture and key packages
- [Project Structure](forte/project-structure.md) — Directory layout and conventions
- [CLI Reference](forte/cli.md) — All `forte` commands (including login and cron jobs)
- [Pages](forte/pages.md) — Page handlers, path/search params, redirects, cookies
- [API Endpoints](forte/apis.md) — JSON API handlers under `/api/`, no SSR
- [Actions & Tasks](forte/actions.md) — Server actions, hooks, queue tasks, admin tasks
- [Frontend Runtime](forte/frontend.md) — `@forte/react` API, `__FORTE_BASE_URL__`, SSR/hydration lifecycle
- [SDK Reference](forte/sdk.md) — `ForteRequest`, HTTP client, cookies, re-exported crates
- [Code Generation](forte/codegen.md) — How `forte-codegen` and `forte-rs-to-ts` work
- [Testing](forte/testing.md) — `#[forte_sdk::test]`, in-memory DB/storage, testing handlers
- [Troubleshooting](forte/troubleshooting.md) — Common build, dev, and deployment issues

## doc-db

- [Overview](doc-db/overview.md) — Document database API, transactions, `#[forte_doc]` macro

## object-storage

- [Overview](object-storage/overview.md) — Per-project S3-style object store, hooks-only credential model

## fn0 Platform

- [Overview](fn0/overview.md) — FaaS runtime, execution model, limits, cluster architecture

## Development

- [Development Workflow](development.md) — Code style, testing, build targets, release process
