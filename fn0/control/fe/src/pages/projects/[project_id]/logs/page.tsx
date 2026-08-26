import { useEffect, useState } from "react";
import type { Props } from "./.props";
import { projectLogs } from "../../../../actions/.generated/project_logs";

const PAGE_LIMIT = 200;

const START_PRESETS: Array<{ label: string; value: string }> = [
    { label: "15m", value: "-15m" },
    { label: "1h", value: "-1h" },
    { label: "6h", value: "-6h" },
    { label: "24h", value: "-24h" },
];

type Row = {
    timestamp: string;
    line: string;
    attributes: Array<{ key: string; value: string }>;
};

type Bucket = { bucketStart: string; bucketEnd: string; count: number };

export default function ProjectLogsPage(props: Props) {
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
    return <LogsView projectId={props.projectId} name={props.name} />;
}

function LogsView({ projectId, name }: { projectId: string; name: string }) {
    const [start, setStart] = useState("-1h");
    const [stream, setStream] = useState("");
    const [query, setQuery] = useState("");
    const [useRegex, setUseRegex] = useState(false);
    const [rows, setRows] = useState<Row[]>([]);
    const [histogram, setHistogram] = useState<Bucket[] | null>(null);
    const [expanded, setExpanded] = useState<Set<number>>(new Set());
    const [error, setError] = useState<string | null>(null);
    const [busy, setBusy] = useState(false);
    const [hasMore, setHasMore] = useState(false);

    function filters() {
        const text = query.trim();
        return {
            stream: stream || undefined,
            contains: useRegex || !text ? undefined : text,
            regex: useRegex && text ? text : undefined,
        };
    }

    async function search() {
        setBusy(true);
        setError(null);
        const res = await projectLogs({
            projectId,
            start,
            limit: PAGE_LIMIT,
            includeHistogram: true,
            ...filters(),
        });
        setBusy(false);
        if (res.t === "Ok") {
            setRows(res.rows);
            setHistogram(res.histogram ?? null);
            setExpanded(new Set());
            setHasMore(res.rows.length === PAGE_LIMIT);
        } else {
            setError(errorMessage(res));
        }
    }

    async function loadMore() {
        const before = rows[rows.length - 1]?.timestamp;
        if (!before || busy) return;
        setBusy(true);
        const res = await projectLogs({
            projectId,
            start,
            limit: PAGE_LIMIT,
            includeHistogram: false,
            before,
            ...filters(),
        });
        setBusy(false);
        if (res.t === "Ok") {
            setRows((prev) => [...prev, ...res.rows]);
            setHasMore(res.rows.length === PAGE_LIMIT);
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

    function toggle(index: number) {
        setExpanded((prev) => {
            const next = new Set(prev);
            if (next.has(index)) next.delete(index);
            else next.add(index);
            return next;
        });
    }

    return (
        <div style={container}>
            <p style={{ marginBottom: 4 }}>
                <a href="/projects">Projects</a> / logs
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
                        value={stream}
                        onChange={(e) => setStream(e.target.value)}
                        style={{ padding: 6 }}
                    >
                        <option value="">all streams</option>
                        <option value="stdout">stdout</option>
                        <option value="stderr">stderr</option>
                    </select>
                </div>
                <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                    <input
                        type="text"
                        placeholder={useRegex ? "regex over the line" : "text contained in the line"}
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

            {histogram && histogram.length > 0 && <Histogram buckets={histogram} />}

            {error && <p style={{ color: "crimson" }}>{error}</p>}

            {rows.length === 0 && !busy && !error ? (
                <p>No logs in this range.</p>
            ) : (
                <div style={{ fontFamily: "ui-monospace, monospace", fontSize: 13 }}>
                    {rows.map((row, index) => {
                        const rowStream = attributeValue(row, "stream");
                        return (
                            <div key={`${row.timestamp}-${index}`} style={rowStyle}>
                                <div
                                    style={{ display: "flex", gap: 8, cursor: "pointer" }}
                                    onClick={() => toggle(index)}
                                >
                                    <span style={{ color: "#888", whiteSpace: "nowrap" }}>
                                        {formatTimestamp(row.timestamp)}
                                    </span>
                                    {rowStream && <StreamBadge stream={rowStream} />}
                                    <span style={{ whiteSpace: "pre-wrap", wordBreak: "break-word" }}>
                                        {row.line}
                                    </span>
                                </div>
                                {expanded.has(index) && (
                                    <table style={{ margin: "6px 0 6px 24px", borderCollapse: "collapse" }}>
                                        <tbody>
                                            {row.attributes.map((attribute) => (
                                                <tr key={attribute.key}>
                                                    <td style={{ padding: "2px 12px 2px 0", color: "#666" }}>
                                                        {attribute.key}
                                                    </td>
                                                    <td style={{ padding: "2px 0" }}>{attribute.value}</td>
                                                </tr>
                                            ))}
                                        </tbody>
                                    </table>
                                )}
                            </div>
                        );
                    })}
                </div>
            )}

            {hasMore && (
                <button onClick={loadMore} disabled={busy} style={{ marginTop: 12 }}>
                    {busy ? "Loading…" : "Load older"}
                </button>
            )}
        </div>
    );
}

function Histogram({ buckets }: { buckets: Bucket[] }) {
    const width = 800;
    const height = 80;
    const max = Math.max(1, ...buckets.map((b) => b.count));
    const barWidth = width / buckets.length;
    return (
        <svg
            viewBox={`0 0 ${width} ${height}`}
            preserveAspectRatio="none"
            style={{ width: "100%", height: 80, marginBottom: 16, background: "#fafafa" }}
        >
            {buckets.map((bucket, index) => {
                const barHeight = (bucket.count / max) * height;
                return (
                    <rect
                        key={bucket.bucketStart}
                        x={index * barWidth}
                        y={height - barHeight}
                        width={Math.max(1, barWidth - 1)}
                        height={barHeight}
                        fill="#4a90d9"
                    >
                        <title>{`${formatTimestamp(bucket.bucketStart)}: ${bucket.count}`}</title>
                    </rect>
                );
            })}
        </svg>
    );
}

function StreamBadge({ stream }: { stream: string }) {
    const isStderr = stream === "stderr";
    return (
        <span
            style={{
                padding: "0 6px",
                borderRadius: 3,
                fontSize: 11,
                background: isStderr ? "#fde2e2" : "#e6eef7",
                color: isStderr ? "#a12" : "#246",
                whiteSpace: "nowrap",
            }}
        >
            {stream}
        </span>
    );
}

function attributeValue(row: Row, key: string): string | undefined {
    return row.attributes.find((attribute) => attribute.key === key)?.value;
}

function formatTimestamp(nanoseconds: string): string {
    const milliseconds = Number(BigInt(nanoseconds) / 1_000_000n);
    return new Date(milliseconds).toISOString().replace("T", " ").replace("Z", "");
}

function errorMessage(res: { t: string } & Record<string, unknown>): string {
    if (res.t === "NotLoggedIn") return "Not signed in.";
    if (res.t === "NotFound") return "Project not found.";
    if (res.t === "Error" && typeof res.message === "string") return res.message;
    return "Something went wrong reading logs.";
}

const container: React.CSSProperties = {
    maxWidth: 1000,
    margin: "2rem auto",
    fontFamily: "system-ui",
    padding: "0 16px",
};

const rowStyle: React.CSSProperties = {
    padding: "3px 0",
    borderBottom: "1px solid #f0f0f0",
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
