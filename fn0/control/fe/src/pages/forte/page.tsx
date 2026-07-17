import type { HeadDescriptor } from "@forte/react";
import { useEffect } from "react";
import { SiteFooter } from "../../components/SiteFooter";
import { GITHUB_URL, SiteHeader } from "../../components/SiteHeader";
import { Terminal, type TerminalLine } from "../../components/Terminal";
import type { Props } from "./.props";

const forteTitle = "Forte — the full-stack framework built on fn0";
const forteDescription =
    "Every request hits Rust first: your handler returns Props as JSON, React server-renders them into HTML and hydrates on the client. One repo, one deploy command.";

export function head(_props: Props): HeadDescriptor[] {
    return [
        { title: forteTitle },
        { name: "description", content: forteDescription },
        { property: "og:title", content: forteTitle },
        { property: "og:description", content: forteDescription },
        { property: "og:url", content: "https://fn0.dev/forte" },
    ];
}

const addPageLines: TerminalLine[] = [
    { kind: "cmd", text: "forte add page blog/[slug]", delay: 0.3, duration: 1.2 },
    { kind: "ok", text: "created rs/src/pages/blog/[slug]/mod.rs", delay: 1.8 },
    { kind: "ok", text: "created fe/src/pages/blog/[slug]/page.tsx", delay: 2.1 },
    { kind: "cmd", text: "forte dev", delay: 2.7, duration: 0.6 },
    {
        kind: "out",
        text: "generated .props.ts · paths.generated.ts · router",
        delay: 3.6,
    },
    { kind: "ok", text: "ready — http://localhost:5173/blog/hello-world", delay: 4.0 },
];

const gettingStartedLines: TerminalLine[] = [
    { kind: "cmd", text: "cargo binstall forte-cli", delay: 0.3, duration: 1.2 },
    { kind: "cmd", text: "forte init my-app", delay: 1.8, duration: 0.9 },
    { kind: "ok", text: "my-app created", delay: 2.9 },
    { kind: "cmd", text: "cd my-app && forte dev", delay: 3.4, duration: 1.1 },
    { kind: "out", text: "dev server running", delay: 4.7 },
    { kind: "link", text: "http://localhost:5173", href: "/docs/forte/cli", delay: 5.1 },
];

const remainingFeatures = [
    {
        title: "Typed server actions",
        body: "Write a Rust handler with Input / Output; the browser gets a generated, Zod-validated TypeScript client. await submit({ message }) — that's the whole integration.",
        href: "/docs/forte/actions",
        linkLabel: "docs/forte/actions →",
    },
    {
        title: "Queue & cron tasks",
        body: "Background jobs with enqueue!, scheduled tasks with a cron.yaml. Same repo, same types, deployed together.",
        href: "/docs/forte/cli#cron-jobs",
        linkLabel: "docs/forte/cli →",
    },
    {
        title: "Testing built in",
        body: "#[forte_sdk::test] runs handlers against the in-memory database and storage — fast, isolated, no Docker required.",
        href: "/docs/forte/testing",
        linkLabel: "docs/forte/testing →",
    },
];

function Eyebrow({ children }: { children: React.ReactNode }) {
    return (
        <p className="font-mono text-sm text-brand-400">
            <span className="select-none">$ </span>
            {children}
        </p>
    );
}

function FlowNode({
    step,
    name,
    sub,
    star,
}: {
    step: string;
    name: string;
    sub: React.ReactNode;
    star?: boolean;
}) {
    return (
        <div
            className={`flex min-w-0 flex-1 flex-col gap-1.5 rounded-lg border p-4 ${
                star
                    ? "border-brand-600/70 bg-brand-600/10 shadow-[0_12px_40px_-18px_rgb(101_79_240/0.5)]"
                    : "border-ink-700 bg-ink-900"
            }`}
        >
            <span
                className={`font-mono text-[0.62rem] tracking-[0.1em] ${
                    star ? "text-brand-300" : "text-ink-400"
                }`}
            >
                {step}
            </span>
            <span className="text-[0.92rem] font-semibold">{name}</span>
            <span className="font-mono text-[0.68rem] leading-relaxed text-ink-400">
                {sub}
            </span>
        </div>
    );
}

function FlowArrow() {
    return (
        <span
            aria-hidden
            className="shrink-0 self-center rotate-90 text-lg text-ink-400 select-none lg:rotate-0"
        >
            →
        </span>
    );
}

function TraceArrow() {
    return (
        <span
            aria-hidden
            className="rotate-90 text-center text-xl text-brand-400 select-none lg:rotate-0"
        >
            →
        </span>
    );
}

function TreeRow({
    indent,
    file,
    badge,
}: {
    indent?: boolean;
    file: string;
    badge?: "you" | "gen";
}) {
    return (
        <div className="flex items-center gap-3 whitespace-nowrap">
            {indent && <span className="text-ink-700 select-none">├─</span>}
            <span className={badge === "gen" ? "text-ink-400" : "text-ink-100"}>
                {file}
            </span>
            {badge === "you" && <YouWriteBadge />}
            {badge === "gen" && <GeneratedBadge />}
        </div>
    );
}

function YouWriteBadge() {
    return (
        <span className="shrink-0 rounded-full border border-brand-400/50 px-2 py-px font-mono text-[0.6rem] tracking-[0.07em] text-brand-300">
            YOU WRITE
        </span>
    );
}

function GeneratedBadge() {
    return (
        <span className="shrink-0 rounded-full border border-emerald-400/40 px-2 py-px font-mono text-[0.6rem] tracking-[0.07em] text-emerald-400">
            GENERATED
        </span>
    );
}

function EnvChip({ command, effect }: { command: string; effect: string }) {
    return (
        <div className="flex flex-wrap items-baseline gap-x-4 gap-y-1 rounded-lg border border-ink-700 bg-ink-900 px-4 py-3">
            <span className="w-44 shrink-0 font-mono text-[0.78rem] text-ink-100">
                {command}
            </span>
            <span className="text-sm text-ink-400">{effect}</span>
        </div>
    );
}

export default function FortePage(props: Props) {
    useEffect(() => {
        document.title = "Forte — the full-stack framework built on fn0";
    }, []);

    return (
        <div className="site min-h-screen">
            <SiteHeader />
            <main>
                <section className="mx-auto max-w-6xl px-5 pt-16 pb-14 lg:pt-24">
                    <Eyebrow>forte</Eyebrow>
                    <h1 className="mt-4 max-w-3xl text-4xl leading-tight font-semibold tracking-tight sm:text-5xl">
                        Every request hits Rust first
                    </h1>
                    <p className="mt-5 max-w-2xl text-lg leading-relaxed text-ink-400">
                        Forte is the full-stack framework built on fn0. Your{" "}
                        <strong className="font-semibold text-ink-100">Rust handler</strong>{" "}
                        receives the request and returns{" "}
                        <strong className="font-semibold text-ink-100">Props as JSON</strong>;{" "}
                        <strong className="font-semibold text-ink-100">React</strong>{" "}
                        server-renders them into HTML and hydrates on the client. One repo,
                        one deploy command.
                    </p>
                    <div className="mt-8 flex flex-wrap items-center gap-4">
                        <a
                            href="/docs/forte/quick-reference"
                            className="rounded-md bg-brand-600 px-5 py-3 font-medium text-white transition-colors hover:bg-brand-500"
                        >
                            Read the docs
                        </a>
                        <a
                            href={GITHUB_URL}
                            className="rounded-md border border-ink-700 px-5 py-3 font-medium text-ink-100 transition-colors hover:border-ink-500"
                        >
                            GitHub →
                        </a>
                    </div>
                </section>

                <section className="mx-auto max-w-6xl px-5 py-14">
                    <Eyebrow>request lifecycle</Eyebrow>
                    <h2 className="mt-3 text-3xl font-semibold tracking-tight">
                        Rust computes, React renders
                    </h2>
                    <p className="mt-4 max-w-xl leading-relaxed text-ink-400">
                        A page is one Rust handler and one React component. In between flows
                        a single JSON document — the Props. Forte generates the TypeScript
                        types for it, so the compiler catches what used to be a runtime bug.
                    </p>

                    <div className="mt-10 flex flex-col gap-2.5 lg:flex-row lg:items-stretch">
                        <FlowNode step="01 · REQUEST" name="Browser" sub="GET /hello" />
                        <FlowArrow />
                        <FlowNode
                            step="02 · RUST"
                            name="Your handler"
                            sub={
                                <>
                                    backend.wasm
                                    <br />
                                    pages/hello/mod.rs
                                </>
                            }
                        />
                        <FlowArrow />
                        <FlowNode
                            step="03 · PROPS"
                            name="JSON across the wire"
                            sub="typed by codegen"
                            star
                        />
                        <FlowArrow />
                        <FlowNode
                            step="04 · REACT SSR"
                            name="Your component"
                            sub={
                                <>
                                    server.js
                                    <br />
                                    pages/hello/page.tsx
                                </>
                            }
                        />
                        <FlowArrow />
                        <FlowNode
                            step="05 · HTML"
                            name="Streamed + hydrated"
                            sub="interactive in the browser"
                        />
                    </div>

                    <div className="mt-7 grid items-center gap-3 lg:grid-cols-[1fr_auto_auto_auto_1fr]">
                        <div className="code-card self-stretch">
                            <div className="code-card-title">rs/src/pages/hello/mod.rs</div>
                            <pre
                                dangerouslySetInnerHTML={{
                                    __html: props.rustSnippetHtml,
                                }}
                            />
                        </div>
                        <TraceArrow />
                        <div className="code-card code-card-accent">
                            <div className="code-card-title">
                                props · serialized by forte-json
                            </div>
                            <pre
                                dangerouslySetInnerHTML={{
                                    __html: props.propsJsonHtml,
                                }}
                            />
                        </div>
                        <TraceArrow />
                        <div className="code-card self-stretch">
                            <div className="code-card-title">
                                fe/src/pages/hello/page.tsx
                            </div>
                            <pre
                                dangerouslySetInnerHTML={{
                                    __html: props.tsxSnippetHtml,
                                }}
                            />
                        </div>
                    </div>
                    <p className="mt-4 text-sm text-ink-400">
                        Follow{" "}
                        <code className="trace-hl font-mono text-[0.9em] text-ink-100">
                            message
                        </code>{" "}
                        across the three panels — one field, one type, checked end to end.
                    </p>

                    <p className="mt-6 max-w-2xl border-l-[3px] border-ink-700 pl-4 text-[0.9rem] leading-relaxed text-ink-400">
                        Need plain JSON instead of a page? Handlers under{" "}
                        <code className="rounded bg-ink-800 px-1.5 py-0.5 font-mono text-[0.85em] text-ink-100">
                            rs/src/apis/
                        </code>{" "}
                        skip the SSR step and return the response directly at{" "}
                        <code className="rounded bg-ink-800 px-1.5 py-0.5 font-mono text-[0.85em] text-ink-100">
                            /api/…
                        </code>
                        .
                    </p>
                </section>

                <section className="mx-auto max-w-6xl px-5 py-14">
                    <Eyebrow>file conventions</Eyebrow>
                    <h2 className="mt-3 text-3xl font-semibold tracking-tight">
                        The directory is the router
                    </h2>
                    <p className="mt-4 max-w-xl leading-relaxed text-ink-400">
                        Drop a handler in{" "}
                        <code className="rounded bg-ink-800 px-1.5 py-0.5 font-mono text-[0.85em] text-ink-100">
                            rs/src/pages/
                        </code>{" "}
                        and a component in{" "}
                        <code className="rounded bg-ink-800 px-1.5 py-0.5 font-mono text-[0.85em] text-ink-100">
                            fe/src/pages/
                        </code>{" "}
                        — Forte generates everything between them: the router, the Zod-typed
                        props, the path helpers. Change the Rust{" "}
                        <code className="rounded bg-ink-800 px-1.5 py-0.5 font-mono text-[0.85em] text-ink-100">
                            Props
                        </code>{" "}
                        and the TypeScript follows; drift becomes a compile error, not a bug
                        report.
                    </p>

                    <div className="mt-8 grid items-start gap-8 lg:grid-cols-2">
                        <div>
                            <div className="code-card">
                                <div className="code-card-title">
                                    my-app/ — after `forte add page blog/[slug]`
                                </div>
                                <div className="overflow-x-auto p-4 font-mono text-[0.78rem] leading-[2.05]">
                                    <TreeRow file="rs/src/" />
                                    <TreeRow
                                        indent
                                        file="pages/blog/[slug]/mod.rs"
                                        badge="you"
                                    />
                                    <TreeRow indent file="route_generated.rs" badge="gen" />
                                    <TreeRow file="fe/src/" />
                                    <TreeRow
                                        indent
                                        file="pages/blog/[slug]/page.tsx"
                                        badge="you"
                                    />
                                    <TreeRow
                                        indent
                                        file="pages/blog/[slug]/.props.ts"
                                        badge="gen"
                                    />
                                    <TreeRow
                                        indent
                                        file="actions/.generated/submit.ts"
                                        badge="gen"
                                    />
                                    <TreeRow indent file="paths.generated.ts" badge="gen" />
                                </div>
                            </div>
                            <div className="mt-4 flex flex-wrap items-center gap-x-5 gap-y-2 text-sm text-ink-400">
                                <span className="flex items-center gap-2">
                                    <YouWriteBadge /> two files per page
                                </span>
                                <span className="flex items-center gap-2">
                                    <GeneratedBadge /> kept in sync on every build
                                </span>
                            </div>
                        </div>
                        <Terminal title="~/my-app" lines={addPageLines} cursorDelay={4.4} />
                    </div>
                </section>

                <section className="mx-auto max-w-6xl px-5 py-14">
                    <Eyebrow>doc-db</Eyebrow>
                    <div className="mt-3 grid items-start gap-10 lg:grid-cols-2">
                        <div>
                            <h2 className="text-3xl font-semibold tracking-tight">
                                A database with no setup step
                            </h2>
                            <p className="mt-4 max-w-xl leading-relaxed text-ink-400">
                                Declare a struct, mark its keys, and{" "}
                                <code className="rounded bg-ink-800 px-1.5 py-0.5 font-mono text-[0.85em] text-ink-100">
                                    #[forte_doc]
                                </code>{" "}
                                derives typed get / put / query / delete — plus optimistic
                                transactions. No connection strings, no migrations, nothing
                                to provision.
                            </p>
                            <div className="mt-6 flex flex-col gap-2.5">
                                <EnvChip
                                    command="forte dev"
                                    effect="local libSQL under .forte/data/ — nothing to install"
                                />
                                <EnvChip
                                    command="#[forte_sdk::test]"
                                    effect="in-memory database — fast, isolated, no Docker"
                                />
                                <EnvChip
                                    command="forte deploy"
                                    effect="managed database, injected by the fn0 runtime"
                                />
                            </div>
                            <p className="mt-4 font-medium text-brand-300">
                                Same code in all three. Only the environment changes.
                            </p>
                            <a
                                href="/docs/doc-db/overview"
                                className="mt-4 inline-block font-mono text-sm text-brand-400 hover:underline"
                            >
                                docs/doc-db →
                            </a>
                        </div>
                        <div className="code-card">
                            <div className="code-card-title">rs/src/db.rs</div>
                            <pre
                                dangerouslySetInnerHTML={{
                                    __html: props.docDbSnippetHtml,
                                }}
                            />
                        </div>
                    </div>
                </section>

                <section className="mx-auto max-w-6xl px-5 py-14">
                    <Eyebrow>object storage</Eyebrow>
                    <div className="mt-3 grid items-start gap-10 lg:grid-cols-2">
                        <div className="code-card order-last lg:order-first">
                            <div className="code-card-title">
                                rs/src/actions/upload_avatar.rs
                            </div>
                            <pre
                                dangerouslySetInnerHTML={{
                                    __html: props.objectStorageSnippetHtml,
                                }}
                            />
                        </div>
                        <div>
                            <h2 className="text-3xl font-semibold tracking-tight">
                                Files, without the S3 ceremony
                            </h2>
                            <p className="mt-4 max-w-xl leading-relaxed text-ink-400">
                                Every project gets its own private bucket, wired in with zero
                                configuration.
                            </p>
                            <ul className="mt-6 flex flex-col gap-2.5">
                                <li className="flex gap-3 text-[0.95rem] leading-relaxed text-ink-400">
                                    <span className="text-brand-400 select-none">▸</span>
                                    Credentials never enter your code — the fn0 runtime signs
                                    every request out of band.
                                </li>
                                <li className="flex gap-3 text-[0.95rem] leading-relaxed text-ink-400">
                                    <span className="text-brand-400 select-none">▸</span>
                                    Presigned URLs let browsers upload and download directly,
                                    without routing through your app.
                                </li>
                                <li className="flex gap-3 text-[0.95rem] leading-relaxed text-ink-400">
                                    <span className="text-brand-400 select-none">▸</span>
                                    <span>
                                        <code className="rounded bg-ink-800 px-1.5 py-0.5 font-mono text-[0.85em] text-ink-100">
                                            forte dev
                                        </code>{" "}
                                        serves the same API from your local filesystem; tests
                                        use{" "}
                                        <code className="rounded bg-ink-800 px-1.5 py-0.5 font-mono text-[0.85em] text-ink-100">
                                            object_storage::memory()
                                        </code>
                                        .
                                    </span>
                                </li>
                            </ul>
                            <a
                                href="/docs/object-storage/overview"
                                className="mt-6 inline-block font-mono text-sm text-brand-400 hover:underline"
                            >
                                docs/object-storage →
                            </a>
                        </div>
                    </div>
                </section>

                <section className="mx-auto max-w-6xl px-5 py-14">
                    <Eyebrow>and the rest</Eyebrow>
                    <h2 className="mt-3 text-3xl font-semibold tracking-tight">
                        Everything a service needs
                    </h2>
                    <div className="mt-8 grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
                        {remainingFeatures.map((feature) => (
                            <div
                                key={feature.title}
                                className="flex flex-col rounded-xl border border-ink-700 bg-ink-900 p-6"
                            >
                                <h3 className="font-semibold">{feature.title}</h3>
                                <p className="mt-2.5 flex-1 text-sm leading-relaxed text-ink-400">
                                    {feature.body}
                                </p>
                                <a
                                    href={feature.href}
                                    className="mt-4 font-mono text-[0.82rem] text-brand-400 hover:underline"
                                >
                                    {feature.linkLabel}
                                </a>
                            </div>
                        ))}
                    </div>
                </section>

                <section className="mx-auto grid max-w-6xl items-center gap-10 px-5 py-14 pb-24 lg:grid-cols-2">
                    <div>
                        <Eyebrow>getting started</Eyebrow>
                        <h2 className="mt-3 text-3xl font-semibold tracking-tight">
                            From zero to dev server
                        </h2>
                        <p className="mt-4 max-w-md leading-relaxed text-ink-400">
                            <code className="font-mono text-[0.9em] text-ink-100">
                                forte dev
                            </code>{" "}
                            gives you hot reload for React and automatic rebuilds for
                            Rust. When it works, <code className="font-mono text-[0.9em] text-ink-100">forte deploy</code>{" "}
                            ships it to fn0 Cloud.
                        </p>
                        <p className="mt-6 text-sm text-ink-400">
                            fn0.dev — the page you're reading — runs on Forte.
                        </p>
                    </div>
                    <Terminal
                        title="~/my-app"
                        lines={gettingStartedLines}
                        cursorDelay={5.5}
                    />
                </section>
            </main>
            <SiteFooter />
        </div>
    );
}
