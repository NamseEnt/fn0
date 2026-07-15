export type TerminalLine =
    | { kind: "cmd"; text: string; delay: number; duration: number }
    | { kind: "ok" | "out"; text: string; delay: number }
    | { kind: "link"; text: string; href: string; delay: number };

export function Terminal({
    title,
    lines,
    cursorDelay,
}: {
    title: string;
    lines: TerminalLine[];
    cursorDelay?: number;
}) {
    return (
        <div className="terminal">
            <div className="terminal-head">
                <span className="terminal-dot" />
                <span className="terminal-dot" />
                <span className="terminal-dot" />
                <span className="terminal-title">{title}</span>
            </div>
            <div className="terminal-body">
                {lines.map((line, i) => (
                    <TerminalLineView key={i} line={line} />
                ))}
                {cursorDelay !== undefined && (
                    <div
                        className="term-line"
                        style={{ "--d": `${cursorDelay}s` } as React.CSSProperties}
                    >
                        <span className="text-brand-400">$ </span>
                        <span className="term-cursor" />
                    </div>
                )}
            </div>
        </div>
    );
}

function TerminalLineView({ line }: { line: TerminalLine }) {
    const delay = { "--d": `${line.delay}s` } as React.CSSProperties;
    switch (line.kind) {
        case "cmd":
            return (
                <div className="term-line" style={delay}>
                    <span className="text-brand-400">$ </span>
                    <span
                        className="term-cmd-text"
                        style={
                            {
                                "--d": `${line.delay}s`,
                                "--dur": `${line.duration}s`,
                                "--steps": line.text.length,
                            } as React.CSSProperties
                        }
                    >
                        {line.text}
                    </span>
                </div>
            );
        case "ok":
            return (
                <div className="term-line" style={delay}>
                    <span className="text-emerald-400">✓ </span>
                    {line.text}
                </div>
            );
        case "out":
            return (
                <div className="term-line text-ink-500" style={delay}>
                    {"  "}
                    {line.text}
                </div>
            );
        case "link":
            return (
                <div className="term-line" style={delay}>
                    <span className="text-brand-400">→ </span>
                    <a href={line.href} className="text-brand-300 underline underline-offset-4">
                        {line.text}
                    </a>
                </div>
            );
    }
}
