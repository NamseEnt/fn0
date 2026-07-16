import { useState } from "react";
import { waitlistJoin } from "../actions/.generated/waitlist_join";

const TIERS = [
    { value: "free", label: "Free" },
    { value: "one_dollar", label: "$1 / mo" },
] as const;

type Tier = (typeof TIERS)[number]["value"];

export function WaitlistForm({ compact = false }: { compact?: boolean }) {
    const [email, setEmail] = useState("");
    const [tier, setTier] = useState<Tier>("free");
    const [busy, setBusy] = useState(false);
    const [done, setDone] = useState(false);
    const [error, setError] = useState<string | null>(null);

    async function onSubmit(e: React.FormEvent) {
        e.preventDefault();
        if (busy || !email.trim()) return;
        setBusy(true);
        setError(null);
        try {
            const res = await waitlistJoin({
                email: email.trim(),
                tier: compact ? "undecided" : tier,
            });
            if (res.t === "Ok") {
                setDone(true);
            } else if (res.t === "InvalidEmail") {
                setError("That email address doesn't look right.");
            } else {
                setError("Something went wrong. Please try again.");
            }
        } catch {
            setError("Something went wrong. Please try again.");
        }
        setBusy(false);
    }

    if (done) {
        return (
            <p className="rounded-lg border border-ink-700 bg-ink-900 px-5 py-4 text-sm">
                <span className="text-emerald-400">✓</span> You're on the list. We'll
                email you when fn0 Cloud opens.
            </p>
        );
    }

    return (
        <form onSubmit={onSubmit} className="flex flex-col gap-3">
            <div className="flex flex-col gap-3 sm:flex-row">
                <input
                    type="email"
                    required
                    placeholder="you@example.com"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    disabled={busy}
                    className="min-w-0 flex-1 rounded-md border border-ink-700 bg-ink-900 px-4 py-2.5 text-sm placeholder:text-ink-500 focus:border-brand-500 focus:outline-none"
                />
                {!compact && (
                    <fieldset className="flex rounded-md border border-ink-700 p-1">
                        <legend className="sr-only">
                            Plan you're interested in
                        </legend>
                        {TIERS.map((t) => (
                            <label
                                key={t.value}
                                className={`cursor-pointer rounded px-3.5 py-1.5 text-sm transition-colors ${
                                    tier === t.value
                                        ? "bg-brand-600 font-medium text-white"
                                        : "text-ink-400 hover:text-ink-100"
                                }`}
                            >
                                <input
                                    type="radio"
                                    name="tier"
                                    value={t.value}
                                    checked={tier === t.value}
                                    onChange={() => setTier(t.value)}
                                    className="sr-only"
                                />
                                {t.label}
                            </label>
                        ))}
                    </fieldset>
                )}
                <button
                    type="submit"
                    disabled={busy || !email.trim()}
                    className="rounded-md bg-brand-600 px-5 py-2.5 text-sm font-medium text-white transition-colors hover:bg-brand-500 disabled:opacity-50"
                >
                    {busy ? "Joining…" : "Join the waitlist"}
                </button>
            </div>
            {error && <p className="text-sm text-red-400">{error}</p>}
        </form>
    );
}
