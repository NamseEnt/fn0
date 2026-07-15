import { Logo } from "./Logo";
import { GITHUB_URL } from "./SiteHeader";

export function SiteFooter() {
    return (
        <footer className="border-t border-ink-800">
            <div className="mx-auto flex max-w-6xl flex-col gap-8 px-5 py-12 sm:flex-row sm:items-start sm:justify-between">
                <div>
                    <Logo className="text-lg" />
                    <p className="mt-3 max-w-xs text-sm text-ink-400">
                        The open-source, batteries-included FaaS platform.
                    </p>
                </div>
                <nav className="grid grid-cols-2 gap-x-16 gap-y-2.5 text-sm">
                    <a href="/docs" className="text-ink-400 hover:text-ink-100">
                        Docs
                    </a>
                    <a href={GITHUB_URL} className="text-ink-400 hover:text-ink-100">
                        GitHub
                    </a>
                    <a href="/forte" className="text-ink-400 hover:text-ink-100">
                        Forte
                    </a>
                    <a
                        href="mailto:projectluda@gmail.com"
                        className="text-ink-400 hover:text-ink-100"
                    >
                        Contact
                    </a>
                </nav>
            </div>
            <div className="mx-auto max-w-6xl px-5 pb-10 text-xs text-ink-500">
                AGPL-3.0 · Commercial license available —{" "}
                <a href="mailto:projectluda@gmail.com" className="hover:text-ink-400">
                    projectluda@gmail.com
                </a>
            </div>
        </footer>
    );
}
