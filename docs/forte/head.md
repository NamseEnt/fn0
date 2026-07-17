# Per-page Head

The SSR `<head>` is assembled from two sources:

- `fe/src/app.tsx` — app-wide defaults and shared resources.
- An optional `head` export in the page module — page-specific metadata.

## App-wide defaults in `fe/src/app.tsx`

Document metadata defaults (title, description, Open Graph tags) belong in a `head` data export. The `Head` component is reserved for shared resources that pages never override: charset, favicon, font links, inline CSS.

```tsx
// fe/src/app.tsx
import type { HeadDescriptor } from "@forte/react";

export const head: HeadDescriptor[] = [
    { title: "My App" },
    { name: "description", content: "The default description." },
    { property: "og:title", content: "My App" },
];

export function Head() {
    return (
        <>
            <meta charSet="utf-8" />
            <link rel="icon" href="/favicon.svg" />
        </>
    );
}
```

## Page metadata via the `head` export

A page module may export a `head` function. It receives the same props as the page component (including `params`) and returns descriptors:

```tsx
// fe/src/pages/product/[id]/page.tsx
import type { HeadDescriptor } from "@forte/react";
import type { Props } from "./.props";

export function head(props: Props & { params: { id: string } }): HeadDescriptor[] {
    if (props.t !== "Ok") return [{ title: "Not found — My App" }];
    return [
        { title: `${props.productName} — My App` },
        { property: "og:title", content: props.productName },
    ];
}

export default function ProductPage(props: Props & { params: { id: string } }) {
    // ...
}
```

`head` must be a pure function of its props: it runs only during SSR and cannot call hooks. Everything the tags need should already be in `Props` — compute it in the Rust handler.

## Descriptor shapes

| Descriptor | Rendered tag |
|---|---|
| `{ title: "..." }` | `<title>...</title>` |
| `{ name: "...", content: "..." }` | `<meta name="..." content="...">` |
| `{ property: "...", content: "..." }` | `<meta property="..." content="...">` |
| `{ tagName: "link" \| "meta", ...attrs }` | `<link ...>` / `<meta ...>` with arbitrary attributes (escape hatch) |

## Merge semantics

The app `head` and the page `head` are merged per key, and the page wins:

- `{ title }` replaces the app title.
- `{ name, content }` replaces the app entry with the same `name`; `{ property, content }` replaces the same `property`.
- App entries the page does not override are kept as-is, so a page that only sets `{ title }` still inherits the app description and Open Graph defaults.
- `{ tagName, ... }` entries are never deduplicated — they are always appended, app entries first.

If a page has no `head` export, the app defaults render unchanged. If `fe/src/app.tsx` has no `head` export either, no descriptor tags are emitted — apps predating this feature behave exactly as before.

Keep `<title>` and `<meta name>`/`<meta property>` tags out of the `Head` component: the merge only sees descriptors, so a title inside the component would duplicate a page-provided title (`forte dev` logs a warning when it detects this).
