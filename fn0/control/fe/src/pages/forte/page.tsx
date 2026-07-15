import { useEffect } from "react";
import { SiteFooter } from "../../components/SiteFooter";
import { GITHUB_URL, SiteHeader } from "../../components/SiteHeader";
import { Terminal, type TerminalLine } from "../../components/Terminal";

const gettingStartedLines: TerminalLine[] = [
    { kind: "cmd", text: "cargo binstall forte-cli", delay: 0.3, duration: 1.2 },
    { kind: "cmd", text: "forte init my-app", delay: 1.8, duration: 0.9 },
    { kind: "ok", text: "my-app created", delay: 2.9 },
    { kind: "cmd", text: "cd my-app && forte dev", delay: 3.4, duration: 1.1 },
    { kind: "out", text: "dev server running", delay: 4.7 },
    { kind: "link", text: "http://localhost:5173", href: "/docs/forte/cli", delay: 5.1 },
];

const features = [
    {
        title: "Server-rendered React",
        body: "Rust handlers compute Props; React renders them on the server and hydrates on the client. Streaming SSR out of the box.",
    },
    {
        title: "Typed server actions",
        body: "Write a Rust handler, get a typed TypeScript client with Zod validation — generated automatically, end to end.",
    },
    {
        title: "doc-db",
        body: "A transactional document database with typed documents and optimistic-conflict transactions. No connection strings to manage.",
    },
    {
        title: "Object storage",
        body: "Per-project S3-style object storage, wired in with zero configuration.",
    },
    {
        title: "Queue & cron tasks",
        body: "Background jobs with enqueue!, scheduled tasks with a cron.yaml. Same repo, same types.",
    },
    {
        title: "Testing built in",
        body: "#[forte_sdk::test] runs handlers against in-memory database and storage — fast, isolated, no Docker required.",
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

export default function FortePage() {
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
                        The full-stack framework built on fn0
                    </h1>
                    <p className="mt-5 max-w-xl text-lg leading-relaxed text-ink-400">
                        Rust on the server, React on the client, types across the wire.
                        One repo, one deploy command.
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
                    <Eyebrow>typed all the way</Eyebrow>
                    <h2 className="mt-3 text-3xl font-semibold tracking-tight">
                        Rust in, React out
                    </h2>
                    <p className="mt-4 max-w-xl leading-relaxed text-ink-400">
                        A page is a Rust handler and a React component. Forte generates
                        the TypeScript types between them, so the compiler catches what
                        used to be a runtime bug.
                    </p>
                    <div className="mt-8 grid gap-5 lg:grid-cols-2">
                        <div className="code-card">
                            <div className="code-card-title">rs/src/pages/hello/mod.rs</div>
                            <pre>{`#[derive(Serialize)]
pub struct Props {
    pub message: String,
}

pub async fn handler(
    _req: ForteRequest<'_>,
) -> Result<Props> {
    Ok(Props {
        message: "hello from Rust".into(),
    })
}`}</pre>
                        </div>
                        <div className="code-card">
                            <div className="code-card-title">
                                fe/src/pages/hello/page.tsx
                            </div>
                            <pre>{`import type { Props } from "./.props";
// ^ generated from the Rust Props type

export default function Hello(props: Props) {
    return <h1>{props.message}</h1>;
}`}</pre>
                        </div>
                    </div>
                </section>

                <section className="mx-auto max-w-6xl px-5 py-14">
                    <Eyebrow>batteries included</Eyebrow>
                    <h2 className="mt-3 text-3xl font-semibold tracking-tight">
                        Everything a service needs
                    </h2>
                    <div className="mt-8 grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
                        {features.map((feature) => (
                            <div
                                key={feature.title}
                                className="rounded-xl border border-ink-700 bg-ink-900 p-6"
                            >
                                <h3 className="font-semibold">{feature.title}</h3>
                                <p className="mt-2.5 text-sm leading-relaxed text-ink-400">
                                    {feature.body}
                                </p>
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
                        <p className="mt-6 text-sm text-ink-500">
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
