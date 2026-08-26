import { useEffect, useState } from "react";
import type { Props } from "./.props";
import { projectTraces } from "../../../../actions/.generated/project_traces";

const PAGE_LIMIT = 100;

const START_PRESETS: Array<{ label: string; value: string }> = [
    { label: "15m", value: "-15m" },
    { label: "1h", value: "-1h" },
    { label: "6h", value: "-6h" },
    { label: "24h", value: "-24h" },
];

type TraceSummary = {
    traceId: string;
    rootService: string;
    rootName: string;
    start: string;
    end: string;
    duration: string;
    spanCount: number;
};

export default function ProjectTracesPage(props: Props) {
    if (props.t !== "Ok") {
        return (
            <div style={container}>
                <p>Project not found, or you do not have access to it.</p>
                <p>
                    <a href="/projects">Back to projects</a>
                </p>
            </div>
        );
    }
    return <TracesView projectId={props.projectId} name={props.name} />;
}

function TracesView({ projectId, name }: { projectId: string; name: string }) {
    const [start, setStart] = useState("-1h");
    const [status, setStatus] = useState("");
    const [minDuration, setMinDuration] = useState("");
    const [query, setQuery] = useState("");
    const [useRegex, setUseRegex] = useState(false);
    const [traces, setTraces] = useState<TraceSummary[]>([]);
    const [error, setError] = useState<string | null>(null);
    const [busy, setBusy] = useState(false);
    const [hasMore, setHasMore] = useState(false);

    function filters() {
        const text = query.trim();
        return {
            status: status || undefined,
            minDuration: minDuration.trim() || undefined,
            nameContains: useRegex || !text ? undefined : text,
            nameRegex: useRegex && text ? text : undefined,
        };
    }

    async function search() {
        setBusy(true);
        setError(null);
        const res = await projectTraces({
            projectId,
            start,
            limit: PAGE_LIMIT,
            ...filters(),
        });
        setBusy(false);
        if (res.t === "Ok") {
            setTraces(res.traces);
            setHasMore(res.traces.length === PAGE_LIMIT);
        } else {
            setError(errorMessage(res));
        }
    }

    async function loadMore() {
        const beforeStart = traces[traces.length - 1]?.start;
        if (!beforeStart || busy) return;
        setBusy(true);
        const res = await projectTraces({
            projectId,
            start,
            limit: PAGE_LIMIT,
            beforeStart,
            ...filters(),
        });
        setBusy(false);
        if (res.t === "Ok") {
            setTraces((prev) => {
                const seen = new Set(prev.map((trace) => trace.traceId));
                return [...prev, ...res.traces.filter((trace) => !seen.has(trace.traceId))];
            });
            setHasMore(res.traces.length === PAGE_LIMIT);
        } else {
            setError(errorMessage(res));
        }
    }

    useEffect(() => {
        search();
    }, []);

    function onSubmit(e: React.FormEvent) {
        e.preventDefault();
        if (!busy) search();
    }

    return (
        <div style={container}>
            <p style={{ marginBottom: 4 }}>
                <a href="/projects">Projects</a> /{" "}
                <a href={`/projects/${encodeURIComponent(projectId)}/logs`}>logs</a> · traces
            </p>
            <h1 style={{ margin: "0 0 16px" }}>
                {name}{" "}
                <span style={{ fontFamily: "monospace", fontSize: 14, color: "#666" }}>
                    {projectId}
                </span>
            </h1>

            <form onSubmit={onSubmit} style={{ display: "grid", gap: 8, marginBottom: 16 }}>
                <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
                    <span>Range:</span>
                    {START_PRESETS.map((preset) => (
                        <button
                            key={preset.value}
                            type="button"
                            onClick={() => setStart(preset.value)}
                            style={start === preset.value ? presetActive : preset_}
                        >
                            {preset.label}
                        </button>
                    ))}
                    <input
                        type="text"
                        value={start}
                        onChange={(e) => setStart(e.target.value)}
                        style={{ width: 200, padding: 6 }}
                        aria-label="start (relative like -1h or absolute)"
                    />
                    <select
                        value={status}
                        onChange={(e) => setStatus(e.target.value)}
                        style={{ padding: 6 }}
                    >
                        <option value="">any status</option>
                        <option value="error">error</option>
                        <option value="ok">ok</option>
                        <option value="unset">unset</option>
                    </select>
                    <input
                        type="text"
                        placeholder="min span duration, e.g. 250ms"
                        value={minDuration}
                        onChange={(e) => setMinDuration(e.target.value)}
                        style={{ width: 190, padding: 6 }}
                    />
                </div>
                <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                    <input
                        type="text"
                        placeholder={
                            useRegex
                                ? "whole-match regex over span names"
                                : "text contained in a span name"
                        }
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        style={{ flex: 1, padding: 6 }}
                    />
                    <label style={{ display: "flex", gap: 4, alignItems: "center" }}>
                        <input
                            type="checkbox"
                            checked={useRegex}
                            onChange={(e) => setUseRegex(e.target.checked)}
                        />
                        regex
                    </label>
                    <button type="submit" disabled={busy}>
                        {busy ? "Searching…" : "Search"}
                    </button>
                </div>
            </form>

            {error && <p style={{ color: "crimson" }}>{error}</p>}

            {traces.length === 0 && !busy && !error ? (
                <p>No traces in this range.</p>
            ) : (
                <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 14 }}>
                    <thead>
                        <tr>
                            <th style={cell}>Start</th>
                            <th style={cell}>Service</th>
                            <th style={cell}>Root span</th>
                            <th style={{ ...cell, textAlign: "right" }}>Duration</th>
                            <th style={{ ...cell, textAlign: "right" }}>Spans</th>
                        </tr>
                    </thead>
                    <tbody>
                        {traces.map((trace) => (
                            <tr key={trace.traceId}>
                                <td style={{ ...cell, whiteSpace: "nowrap", color: "#666" }}>
                                    {formatTimestamp(trace.start)}
                                </td>
                                <td style={cell}>{trace.rootService}</td>
                                <td style={cell}>
                                    <a
                                        href={`/projects/${encodeURIComponent(projectId)}/traces/${trace.traceId}`}
                                        style={{ fontFamily: "ui-monospace, monospace" }}
                                    >
                                        {trace.rootName}
                                    </a>
                                </td>
                                <td style={{ ...cell, textAlign: "right", whiteSpace: "nowrap" }}>
                                    {formatDuration(trace.duration)}
                                </td>
                                <td style={{ ...cell, textAlign: "right" }}>{trace.spanCount}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            )}

            {hasMore && (
                <button onClick={loadMore} disabled={busy} style={{ marginTop: 12 }}>
                    {busy ? "Loading…" : "Load older"}
                </button>
            )}
        </div>
    );
}

function formatTimestamp(nanoseconds: string): string {
    const milliseconds = Number(BigInt(nanoseconds) / 1_000_000n);
    return new Date(milliseconds).toISOString().replace("T", " ").replace("Z", "");
}

function formatDuration(nanoseconds: string): string {
    const ns = Number(nanoseconds);
    if (ns < 1_000) return `${ns}ns`;
    if (ns < 1_000_000) return `${(ns / 1_000).toFixed(1)}µs`;
    if (ns < 1_000_000_000) return `${(ns / 1_000_000).toFixed(1)}ms`;
    return `${(ns / 1_000_000_000).toFixed(2)}s`;
}

function errorMessage(res: { t: string } & Record<string, unknown>): string {
    if (res.t === "NotLoggedIn") return "Not signed in.";
    if (res.t === "NotFound") return "Project not found.";
    if (res.t === "Error" && typeof res.message === "string") return res.message;
    return "Something went wrong reading traces.";
}

const container: React.CSSProperties = {
    maxWidth: 1000,
    margin: "2rem auto",
    fontFamily: "system-ui",
    padding: "0 16px",
};

const cell: React.CSSProperties = {
    padding: "6px 8px",
    borderBottom: "1px solid #eee",
    textAlign: "left",
};

const preset_: React.CSSProperties = {
    padding: "2px 8px",
    border: "1px solid #ccc",
    background: "#fff",
    cursor: "pointer",
};

const presetActive: React.CSSProperties = {
    ...preset_,
    background: "#246",
    color: "#fff",
    borderColor: "#246",
};
