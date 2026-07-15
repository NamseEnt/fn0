import { useEffect } from "react";
import { SiteFooter } from "./SiteFooter";
import { SiteHeader } from "./SiteHeader";

export type DocsNav = {
    title: string;
    items: { route: string; title: string }[];
}[];

function docHref(route: string) {
    return route === "" ? "/docs" : `/docs/${route}`;
}

function DocsNavList({ nav, route }: { nav: DocsNav; route: string }) {
    return (
        <nav className="flex flex-col gap-6">
            {nav.map((section) => (
                <div key={section.title}>
                    <div className="mb-2 font-mono text-xs tracking-wide text-ink-500 uppercase">
                        {section.title}
                    </div>
                    <ul className="flex flex-col">
                        {section.items.map((item) => (
                            <li key={item.route}>
                                <a
                                    href={docHref(item.route)}
                                    className={`block rounded px-2.5 py-1.5 text-sm transition-colors ${
                                        item.route === route
                                            ? "bg-brand-600/15 font-medium text-brand-300"
                                            : "text-ink-400 hover:text-ink-100"
                                    }`}
                                >
                                    {item.title}
                                </a>
                            </li>
                        ))}
                    </ul>
                </div>
            ))}
        </nav>
    );
}

export function DocsLayout({
    nav,
    route,
    title,
    children,
}: {
    nav: DocsNav;
    route: string;
    title: string | null;
    children: React.ReactNode;
}) {
    useEffect(() => {
        document.title = title ? `${title} · fn0 docs` : "fn0 docs";
    }, [title]);

    return (
        <div className="site min-h-screen">
            <SiteHeader />
            <div className="mx-auto flex max-w-6xl items-start gap-10 px-5 py-10">
                <aside className="sticky top-10 hidden w-56 shrink-0 lg:block">
                    <DocsNavList nav={nav} route={route} />
                </aside>
                <div className="min-w-0 flex-1">
                    <details className="mb-8 rounded-lg border border-ink-700 bg-ink-900 px-4 py-3 lg:hidden">
                        <summary className="cursor-pointer font-mono text-sm text-ink-400">
                            All docs
                        </summary>
                        <div className="pt-4">
                            <DocsNavList nav={nav} route={route} />
                        </div>
                    </details>
                    {children}
                </div>
            </div>
            <SiteFooter />
        </div>
    );
}

export function DocNotFound() {
    return (
        <div className="py-10">
            <p className="font-mono text-sm text-brand-400">404</p>
            <h1 className="mt-2 text-2xl font-semibold">
                This doc page doesn't exist.
            </h1>
            <p className="mt-3 text-ink-400">
                It may have moved.{" "}
                <a href="/docs" className="text-brand-400 hover:underline">
                    Browse all docs
                </a>{" "}
                to find what you need.
            </p>
        </div>
    );
}
