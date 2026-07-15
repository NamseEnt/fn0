import { SiteFooter } from "../../components/SiteFooter";
import { GITHUB_URL, SiteHeader } from "../../components/SiteHeader";
import { Terminal, type TerminalLine } from "../../components/Terminal";
import { WaitlistForm } from "../../components/WaitlistForm";

const heroLines: TerminalLine[] = [
    { kind: "cmd", text: "fn0 init my-app", delay: 0.3, duration: 1.0 },
    { kind: "ok", text: "my-app created", delay: 1.5 },
    { kind: "cmd", text: "cd my-app && fn0 deploy", delay: 2.0, duration: 1.3 },
    { kind: "out", text: "compiling to wasm ......... done", delay: 3.5 },
    { kind: "out", text: "uploading bundle .......... done", delay: 3.9 },
    { kind: "ok", text: "deployed in 4.2s", delay: 4.3 },
    {
        kind: "link",
        text: "https://my-app.fn0.dev",
        href: "https://fn0.dev",
        delay: 4.7,
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

const features = [
    {
        title: (
            <>
                Easier to build. <br className="hidden sm:block" />
                Easier to deploy.
            </>
        ),
        body: (
            <>
                <code>fn0 init</code> scaffolds a project; <code>fn0 deploy</code>{" "}
                ships it. No Dockerfiles, no YAML, no infrastructure to babysit.
            </>
        ),
    },
    {
        title: <>Any language that compiles to Wasm</>,
        body: (
            <>
                fn0 runs WASI 0.2 components on wasmtime. Rust and
                JavaScript/TypeScript are supported today; more arrive as the
                ecosystem grows.
            </>
        ),
    },
    {
        title: <>Local is production</>,
        body: (
            <>
                <code>fn0 local</code> runs the same runtime on your machine. What
                works locally works deployed — no staging surprises.
            </>
        ),
    },
    {
        title: <>Open source, no lock-in</>,
        body: (
            <>
                AGPL-3.0 on GitHub. Self-host on AWS or OCI with adapters, or let fn0
                Cloud run it for you.
            </>
        ),
    },
];

const limits = [
    ["Request headers", "128 KB"],
    ["Request body", "100 MB"],
    ["Response headers", "128 KB"],
    ["Response body", "Unlimited"],
    ["Memory", "128 MB"],
    ["CPU time", "10 ms"],
    ["Max duration", "15 seconds"],
    ["Subrequests", "50 per request"],
];

export default function IndexPage() {
    return (
        <div className="site min-h-screen">
            <SiteHeader />
            <main>
                <section className="mx-auto grid max-w-6xl items-center gap-12 px-5 pt-16 pb-20 lg:grid-cols-2 lg:pt-24">
                    <div>
                        <Eyebrow>fn0 — pronounced f-n-zero</Eyebrow>
                        <h1 className="mt-4 text-4xl leading-tight font-semibold tracking-tight sm:text-5xl">
                            You build the service.
                            <br />
                            <span className="text-brand-400">We handle the rest.</span>
                        </h1>
                        <p className="mt-5 max-w-lg text-lg leading-relaxed text-ink-400">
                            fn0 is an open-source, batteries-included FaaS platform powered
                            by WebAssembly — a reasonable alternative to Cloudflare
                            Workers.
                        </p>
                        <div className="mt-8 flex flex-wrap items-center gap-4">
                            <a
                                href="#waitlist"
                                className="rounded-md bg-brand-600 px-5 py-3 font-medium text-white transition-colors hover:bg-brand-500"
                            >
                                Join the cloud waitlist
                            </a>
                            <a
                                href={GITHUB_URL}
                                className="rounded-md border border-ink-700 px-5 py-3 font-medium text-ink-100 transition-colors hover:border-ink-500"
                            >
                                GitHub →
                            </a>
                        </div>
                        <p className="mt-5 font-mono text-sm text-ink-500">
                            cargo binstall fn0-cli
                        </p>
                    </div>
                    <Terminal title="~/my-app" lines={heroLines} cursorDelay={5.1} />
                </section>

                <section className="mx-auto max-w-6xl px-5 py-16">
                    <Eyebrow>why fn0</Eyebrow>
                    <h2 className="mt-3 text-3xl font-semibold tracking-tight">
                        All-in-one, without the ceremony
                    </h2>
                    <div className="mt-8 grid gap-5 sm:grid-cols-2">
                        {features.map((feature, i) => (
                            <div
                                key={i}
                                className="rounded-xl border border-ink-700 bg-ink-900 p-6"
                            >
                                <h3 className="text-lg font-semibold [&_code]:font-mono [&_code]:text-brand-300">
                                    {feature.title}
                                </h3>
                                <p className="mt-2.5 leading-relaxed text-ink-400 [&_code]:font-mono [&_code]:text-[0.85em] [&_code]:text-ink-100">
                                    {feature.body}
                                </p>
                            </div>
                        ))}
                    </div>
                </section>

                <section className="mx-auto max-w-6xl px-5 py-16">
                    <div className="grid items-center gap-10 rounded-2xl border border-ink-700 bg-ink-900 p-8 sm:p-12 lg:grid-cols-2">
                        <div>
                            <Eyebrow>full-stack</Eyebrow>
                            <h2 className="mt-3 text-3xl font-semibold tracking-tight">
                                Forte — the full-stack framework built on fn0
                            </h2>
                            <p className="mt-4 leading-relaxed text-ink-400">
                                Server-rendered React pages, typed server actions, a
                                document database, and object storage — one repo, one
                                deploy command. This page is served by Forte on fn0.
                            </p>
                            <a
                                href="/forte"
                                className="mt-6 inline-block font-medium text-brand-400 hover:text-brand-300"
                            >
                                Meet Forte →
                            </a>
                        </div>
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
        message: "typed, server-rendered".into(),
    })
}`}</pre>
                        </div>
                    </div>
                </section>

                <section id="waitlist" className="mx-auto max-w-6xl px-5 py-16">
                    <Eyebrow>fn0 cloud</Eyebrow>
                    <h2 className="mt-3 text-3xl font-semibold tracking-tight">
                        fn0 Cloud is coming
                    </h2>
                    <p className="mt-4 max-w-lg leading-relaxed text-ink-400">
                        A managed fn0 — you deploy, we run it. Two plans are in the
                        works. Join the waitlist and we'll email you when it opens.
                    </p>
                    <div className="mt-8 grid max-w-2xl gap-5 sm:grid-cols-2">
                        <div className="rounded-xl border border-ink-700 bg-ink-900 p-6">
                            <div className="flex items-baseline justify-between">
                                <span className="font-mono text-sm text-ink-400">free</span>
                                <span className="rounded-full border border-ink-700 px-2.5 py-0.5 font-mono text-xs text-ink-500">
                                    planned
                                </span>
                            </div>
                            <p className="mt-3 text-3xl font-semibold">$0</p>
                            <p className="mt-2 text-sm leading-relaxed text-ink-400">
                                For hobby projects and trying things out.
                            </p>
                        </div>
                        <div className="rounded-xl border border-brand-600/40 bg-ink-900 p-6">
                            <div className="flex items-baseline justify-between">
                                <span className="font-mono text-sm text-brand-400">
                                    one dollar
                                </span>
                                <span className="rounded-full border border-ink-700 px-2.5 py-0.5 font-mono text-xs text-ink-500">
                                    planned
                                </span>
                            </div>
                            <p className="mt-3 text-3xl font-semibold">
                                $1
                                <span className="text-base font-normal text-ink-400">
                                    {" "}
                                    / month
                                </span>
                            </p>
                            <p className="mt-2 text-sm leading-relaxed text-ink-400">
                                For small production services.
                            </p>
                        </div>
                    </div>
                    <div className="mt-8 max-w-2xl">
                        <WaitlistForm />
                    </div>
                </section>

                <section className="mx-auto max-w-6xl px-5 py-16 pb-24">
                    <Eyebrow>limits</Eyebrow>
                    <h2 className="mt-3 text-3xl font-semibold tracking-tight">
                        fn0 Cloud limits
                    </h2>
                    <p className="mt-4 max-w-lg leading-relaxed text-ink-400">
                        Per request, on fn0 Cloud. Run fn0 yourself and these limits are
                        yours to change.
                    </p>
                    <div className="mt-8 max-w-2xl overflow-x-auto rounded-xl border border-ink-700">
                        <table className="w-full text-sm">
                            <tbody>
                                {limits.map(([name, value]) => (
                                    <tr
                                        key={name}
                                        className="border-b border-ink-800 last:border-b-0"
                                    >
                                        <td className="px-5 py-3 text-ink-400">{name}</td>
                                        <td className="px-5 py-3 text-right font-mono text-ink-100">
                                            {value}
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </section>
            </main>
            <SiteFooter />
        </div>
    );
}
