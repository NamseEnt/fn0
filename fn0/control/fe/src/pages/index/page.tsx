import { Fragment } from "react";
import { SiteFooter } from "../../components/SiteFooter";
import { GITHUB_URL, SiteHeader } from "../../components/SiteHeader";
import { Terminal, type TerminalLine } from "../../components/Terminal";
import { WaitlistForm } from "../../components/WaitlistForm";
import type { Props } from "./.props";

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
    ["CPU time", "50 ms"],
    ["Max duration", "15 seconds"],
    ["Subrequests", "50 per request"],
];

type QuotaRow = {
    name: string;
    value: string;
    note?: string;
};

type QuotaGroup = {
    title?: string;
    rows: QuotaRow[];
};

const dollarQuotaGroups: QuotaGroup[] = [
    {
        title: "Projects & domains",
        rows: [
            { name: "Projects", value: "1" },
            {
                name: "Custom domains",
                value: "1",
                note: "Automatic TLS — point a CNAME and you're done.",
            },
            { name: "fn0.dev subdomain", value: "Included" },
        ],
    },
    {
        title: "Compute",
        rows: [
            {
                name: "CPU pool",
                value: "500 min / mo",
                note: "≈ 2M server-rendered pages at ~15 ms each. Pooled monthly, not daily — a launch-day spike won't hit a wall.",
            },
            {
                name: "CPU per request",
                value: "50 ms",
                note: "Only your code counts. Waiting on a database or an LLM API costs nothing.",
            },
        ],
    },
    {
        title: "Network",
        rows: [
            {
                name: "Compute egress",
                value: "20 GB / mo",
                note: "SSR pages and API responses leaving your handlers.",
            },
            {
                name: "Object storage downloads",
                value: "Unlimited",
                note: "Served through the CDN cache — never metered, never counted as egress.",
            },
        ],
    },
    {
        title: "Document database",
        rows: [
            { name: "Active databases", value: "1" },
            { name: "Storage", value: "500 MB" },
            { name: "Row reads", value: "150M / mo" },
            {
                name: "Row writes",
                value: "1M / mo",
                note: "A busy community site writes ~300k a month.",
            },
        ],
    },
    {
        title: "Object storage",
        rows: [
            { name: "Storage", value: "10 GB" },
            { name: "Uploads", value: "100k / mo" },
            {
                name: "Downloads",
                value: "Unlimited",
                note: "Served through the CDN cache.",
            },
        ],
    },
    {
        title: "Queues & cron",
        rows: [
            {
                name: "Queue tasks & cron runs",
                value: "Shared CPU pool",
                note: "They spend the same CPU minutes as requests — nothing extra to buy.",
            },
            {
                name: "Cron jobs",
                value: "10 / project",
                note: "1 minute minimum interval.",
            },
            { name: "Queue message size", value: "128 KB" },
            { name: "Queue backlog", value: "100k messages" },
        ],
    },
];

const freeQuotaGroups: QuotaGroup[] = [
    {
        rows: [
            { name: "Projects", value: "1" },
            { name: "fn0.dev subdomain", value: "Included" },
            { name: "Custom domains", value: "—" },
            {
                name: "Monthly quotas",
                value: "TBA",
                note: "Sized for trying things out and development traffic.",
            },
        ],
    },
];

const roadmap = [
    {
        name: "Monitoring & logs",
        note: "See what your service is doing without leaving fn0.",
    },
    {
        name: "Pay-as-you-go overage",
        note: "Keep growing past the included quotas without hitting a wall.",
    },
    {
        name: "Bring your own R2 / Turso",
        note: "Connect your own bucket or database — your resources, your limits.",
    },
    {
        name: "A higher tier",
        note: "For services that outgrow a dollar.",
    },
];

function QuotaTable({ groups }: { groups: QuotaGroup[] }) {
    return (
        <div className="overflow-x-auto rounded-xl border border-ink-700">
            <table className="w-full text-sm">
                <tbody>
                    {groups.map((group, groupIndex) => (
                        <Fragment key={groupIndex}>
                            {group.title && (
                                <tr className="border-b border-ink-800">
                                    <td
                                        colSpan={3}
                                        className="px-5 pt-5 pb-2 font-mono text-xs tracking-widest text-brand-400 uppercase"
                                    >
                                        {group.title}
                                    </td>
                                </tr>
                            )}
                            {group.rows.map((row) => (
                                <tr
                                    key={row.name}
                                    className="border-b border-ink-800 last:border-b-0"
                                >
                                    <td className="w-0 px-5 py-3 align-top whitespace-nowrap text-ink-400">
                                        {row.name}
                                    </td>
                                    <td className="w-0 py-3 pr-6 pl-1 align-top font-mono whitespace-nowrap text-ink-100">
                                        {row.value}
                                    </td>
                                    <td className="min-w-56 px-5 py-3 align-top leading-relaxed text-ink-400">
                                        {row.note}
                                    </td>
                                </tr>
                            ))}
                        </Fragment>
                    ))}
                </tbody>
            </table>
        </div>
    );
}

function PlannedBadge() {
    return (
        <span className="rounded-full border border-ink-700 px-2.5 py-0.5 font-mono text-xs text-ink-500">
            planned
        </span>
    );
}

export default function IndexPage(props: Props) {
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
                            <pre
                                dangerouslySetInnerHTML={{
                                    __html: props.forteSnippetHtml,
                                }}
                            />
                        </div>
                    </div>
                </section>

                <section id="plans" className="mx-auto max-w-6xl px-5 py-16">
                    <div className="mx-auto max-w-3xl">
                        <div className="text-center">
                            <Eyebrow>fn0 cloud</Eyebrow>
                            <h2 className="mt-3 text-3xl font-semibold tracking-tight">
                                fn0 Cloud is coming
                            </h2>
                            <p className="mx-auto mt-4 max-w-lg leading-relaxed text-ink-400">
                                A managed fn0 — you deploy, we run it. Two plans are in
                                the works.
                            </p>
                        </div>

                        <div className="mt-10 rounded-2xl border border-ink-700 bg-ink-900 p-6 sm:p-8">
                            <div className="flex items-baseline justify-between">
                                <span className="font-mono text-sm text-ink-400">free</span>
                                <PlannedBadge />
                            </div>
                            <p className="mt-3 text-3xl font-semibold">$0</p>
                            <p className="mt-2 text-sm leading-relaxed text-ink-400">
                                For hobby projects and trying things out.
                            </p>
                            <div className="mt-5">
                                <QuotaTable groups={freeQuotaGroups} />
                            </div>
                        </div>

                        <div className="mt-6 rounded-2xl border border-brand-600/40 bg-ink-900 p-6 sm:p-8">
                            <div className="flex items-baseline justify-between">
                                <span className="font-mono text-sm text-brand-400">
                                    one dollar
                                </span>
                                <PlannedBadge />
                            </div>
                            <p className="mt-3 text-3xl font-semibold">
                                $1
                                <span className="text-base font-normal text-ink-400">
                                    {" "}
                                    / month
                                </span>
                            </p>
                            <p className="mt-1 font-mono text-brand-300">
                                ≈ 2,000,000 server-rendered requests, every month
                            </p>
                            <p className="mt-2 text-sm leading-relaxed text-ink-400">
                                For small production services — a real domain, a
                                database, storage, queues, and cron behind one deploy
                                command.
                            </p>
                            <div className="mt-6">
                                <QuotaTable groups={dollarQuotaGroups} />
                            </div>
                            <p className="mt-4 text-sm">
                                <a
                                    href="/docs/fn0/limits"
                                    className="font-medium text-brand-400 hover:text-brand-300"
                                >
                                    Full limits &amp; quotas in the docs →
                                </a>
                            </p>
                        </div>

                        <div className="mt-6 rounded-2xl border border-ink-700 bg-ink-900 p-6 sm:p-8">
                            <div className="flex items-baseline justify-between">
                                <span className="font-mono text-sm text-ink-400">
                                    on the roadmap
                                </span>
                                <PlannedBadge />
                            </div>
                            <ul className="mt-5 grid gap-5 sm:grid-cols-2">
                                {roadmap.map((item) => (
                                    <li key={item.name}>
                                        <p className="font-medium text-ink-100">
                                            {item.name}
                                        </p>
                                        <p className="mt-1 text-sm leading-relaxed text-ink-400">
                                            {item.note}
                                        </p>
                                    </li>
                                ))}
                            </ul>
                        </div>

                    </div>
                </section>

                <section id="waitlist" className="mx-auto max-w-6xl px-5 py-16">
                    <div className="mx-auto max-w-2xl">
                        <div className="text-center">
                            <Eyebrow>waitlist</Eyebrow>
                            <h2 className="mt-3 text-3xl font-semibold tracking-tight">
                                Join the waitlist
                            </h2>
                            <p className="mx-auto mt-4 max-w-lg leading-relaxed text-ink-400">
                                fn0 Cloud isn't open yet. Leave your email and we'll let
                                you know the moment it is.
                            </p>
                        </div>
                        <div className="mt-8">
                            <WaitlistForm />
                        </div>
                    </div>
                </section>

                <section className="mx-auto grid max-w-6xl items-start gap-10 px-5 py-16 pb-24 lg:grid-cols-2">
                    <div>
                        <Eyebrow>limits</Eyebrow>
                        <h2 className="mt-3 text-3xl font-semibold tracking-tight">
                            Per-request limits
                        </h2>
                        <p className="mt-4 max-w-lg leading-relaxed text-ink-400">
                            Applied to every invocation on fn0 Cloud, on every plan.
                            Run fn0 yourself and these limits are yours to change.
                        </p>
                        <a
                            href="/docs/fn0/limits"
                            className="mt-4 inline-block font-medium text-brand-400 hover:text-brand-300"
                        >
                            All limits &amp; quotas →
                        </a>
                    </div>
                    <div className="overflow-x-auto rounded-xl border border-ink-700">
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
