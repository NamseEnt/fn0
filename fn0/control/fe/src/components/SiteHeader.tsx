import { Logo } from "./Logo";

export const GITHUB_URL = "https://github.com/NamseEnt/fn0";

export function SiteHeader() {
    return (
        <header className="border-b border-ink-800">
            <div className="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-x-6 gap-y-3 px-5 py-4">
                <a href="/" className="text-lg text-ink-100">
                    <Logo />
                </a>
                <nav className="flex flex-wrap items-center gap-x-6 gap-y-2 text-sm">
                    <a href="/docs" className="text-ink-400 transition-colors hover:text-ink-100">
                        Docs
                    </a>
                    <a href="/forte" className="text-ink-400 transition-colors hover:text-ink-100">
                        Forte
                    </a>
                    <a
                        href={GITHUB_URL}
                        className="text-ink-400 transition-colors hover:text-ink-100"
                    >
                        GitHub
                    </a>
                    <a
                        href="/#waitlist"
                        className="rounded-md bg-brand-600 px-3.5 py-2 font-medium text-white transition-colors hover:bg-brand-500"
                    >
                        Join waitlist
                    </a>
                </nav>
            </div>
        </header>
    );
}
