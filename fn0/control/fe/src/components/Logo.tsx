export function Logo({ className = "" }: { className?: string }) {
    return (
        <span
            className={`inline-flex items-center gap-1.5 font-mono font-semibold ${className}`}
        >
            <span className="text-brand-400">&gt;</span>
            <span>fn0</span>
            <span className="logo-cursor" aria-hidden="true" />
        </span>
    );
}
