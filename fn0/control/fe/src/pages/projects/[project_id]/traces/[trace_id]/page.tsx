import { useEffect, useState } from "react";
import type { Props } from "./.props";
import { projectTraceSpans } from "../../../../../actions/.generated/project_trace_spans";

type AttributePair = { key: string; value: string };

type SpanEvent = {
    timestamp: string;
    name: string;
    attributes: AttributePair[];
};

type Span = {
    spanId: string;
    parentSpanId: string;
    name: string;
    kind: string;
    service: string;
    status: string;
    start: string;
    end: string;
    duration: string;
    attributes: AttributePair[];
    events: SpanEvent[];
};

type SpanRow = { span: Span; depth: number };

export default function ProjectTracePage(props: Props) {
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
    return (
        <TraceView projectId={props.projectId} name={props.name} traceId={props.traceId} />
    );
}

function TraceView({
    projectId,
    name,
    traceId,
}: {
    projectId: string;
    name: string;
    traceId: string;
}) {
    const [spans, setSpans] = useState<Span[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [expanded, setExpanded] = useState<Set<string>>(new Set());

    useEffect(() => {
        (async () => {
            const res = await projectTraceSpans({ projectId, traceId });
            if (res.t === "Ok") {
                setSpans(res.spans);
            } else {
                setError(errorMessage(res));
            }
        })();
    }, []);

    function toggle(spanId: string) {
        setExpanded((prev) => {
            const next = new Set(prev);
            if (next.has(spanId)) next.delete(spanId);
            else next.add(spanId);
            return next;
        });
    }

    const tracesHref = `/projects/${encodeURIComponent(projectId)}/traces`;

    return (
        <div style={container}>
            <p style={{ marginBottom: 4 }}>
                <a href="/projects">Projects</a> / <a href={tracesHref}>traces</a> / trace
            </p>
            <h1 style={{ margin: "0 0 4px" }}>
                {name}{" "}
                <span style={{ fontFamily: "monospace", fontSize: 14, color: "#666" }}>
                    {projectId}
                </span>
            </h1>
            <p style={{ margin: "0 0 16px", fontFamily: "ui-monospace, monospace", fontSize: 13 }}>
                {traceId}
                {spans && spans.length > 0 && (
                    <span style={{ color: "#666" }}>
                        {" "}
                        · {spans.length} span{spans.length === 1 ? "" : "s"} ·{" "}
                        {formatDuration(extentNs(spans).toString())}
                    </span>
                )}
            </p>

            {error && <p style={{ color: "crimson" }}>{error}</p>}
            {!spans && !error && <p>Loading…</p>}

            {spans && spans.length > 0 && (
                <Waterfall spans={spans} expanded={expanded} toggle={toggle} />
            )}
        </div>
    );
}

function Waterfall({
    spans,
    expanded,
    toggle,
}: {
    spans: Span[];
    expanded: Set<string>;
    toggle: (spanId: string) => void;
}) {
    const rows = spanTree(spans);
    const traceStart = minStart(spans);
    const extent = extentNs(spans);
    const services = new Set(spans.map((span) => span.service));

    return (
        <div style={{ fontSize: 13 }}>
            {rows.map(({ span, depth }) => {
                const leftPercent = percentOf(BigInt(span.start) - traceStart, extent);
                const widthPercent = percentOf(BigInt(span.duration), extent);
                const isError = span.status === "error";
                return (
                    <div key={span.spanId} style={rowStyle}>
                        <div
                            style={{ display: "flex", alignItems: "center", cursor: "pointer" }}
                            onClick={() => toggle(span.spanId)}
                        >
                            <div
                                style={{
                                    width: 340,
                                    minWidth: 340,
                                    paddingLeft: depth * 14,
                                    display: "flex",
                                    gap: 6,
                                    alignItems: "baseline",
                                    overflow: "hidden",
                                }}
                            >
                                <span
                                    style={{
                                        fontFamily: "ui-monospace, monospace",
                                        whiteSpace: "nowrap",
                                        overflow: "hidden",
                                        textOverflow: "ellipsis",
                                        color: isError ? "#a12" : undefined,
                                    }}
                                    title={span.name}
                                >
                                    {span.name}
                                </span>
                                {services.size > 1 && (
                                    <span style={{ fontSize: 11, color: "#888", whiteSpace: "nowrap" }}>
                                        {span.service}
                                    </span>
                                )}
                                {isError && <ErrorBadge />}
                            </div>
                            <div style={trackStyle}>
                                <div
                                    style={{
                                        position: "absolute",
                                        left: `${leftPercent}%`,
                                        width: `${Math.max(widthPercent, 0.5)}%`,
                                        top: 3,
                                        bottom: 3,
                                        borderRadius: 2,
                                        background: isError ? "#d64545" : "#4a90d9",
                                    }}
                                />
                            </div>
                            <div
                                style={{
                                    width: 80,
                                    minWidth: 80,
                                    textAlign: "right",
                                    color: "#666",
                                    whiteSpace: "nowrap",
                                }}
                            >
                                {formatDuration(span.duration)}
                            </div>
                        </div>
                        {expanded.has(span.spanId) && <SpanDetails span={span} />}
                    </div>
                );
            })}
        </div>
    );
}

function SpanDetails({ span }: { span: Span }) {
    return (
        <div style={{ margin: "6px 0 10px 24px", fontFamily: "ui-monospace, monospace" }}>
            <table style={{ borderCollapse: "collapse" }}>
                <tbody>
                    <DetailRow name="span_id" value={span.spanId} />
                    {span.parentSpanId && (
                        <DetailRow name="parent_span_id" value={span.parentSpanId} />
                    )}
                    <DetailRow name="kind" value={span.kind} />
                    <DetailRow name="status" value={span.status} />
                    <DetailRow name="start" value={formatTimestamp(span.start)} />
                    {span.attributes.map((attribute) => (
                        <DetailRow key={attribute.key} name={attribute.key} value={attribute.value} />
                    ))}
                </tbody>
            </table>
            {span.events.length > 0 && (
                <div style={{ marginTop: 6 }}>
                    {span.events.map((event, eventIndex) => (
                        <div key={`${event.timestamp}-${eventIndex}`} style={{ marginBottom: 4 }}>
                            <span style={{ color: "#666" }}>{formatTimestamp(event.timestamp)}</span>{" "}
                            <strong>{event.name}</strong>
                            {event.attributes.map((attribute) => (
                                <span key={attribute.key} style={{ marginLeft: 8, color: "#444" }}>
                                    {attribute.key}={attribute.value}
                                </span>
                            ))}
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}

function DetailRow({ name, value }: { name: string; value: string }) {
    return (
        <tr>
            <td style={{ padding: "2px 12px 2px 0", color: "#666", verticalAlign: "top" }}>
                {name}
            </td>
            <td style={{ padding: "2px 0", wordBreak: "break-all" }}>{value}</td>
        </tr>
    );
}

function ErrorBadge() {
    return (
        <span
            style={{
                padding: "0 6px",
                borderRadius: 3,
                fontSize: 11,
                background: "#fde2e2",
                color: "#a12",
                whiteSpace: "nowrap",
            }}
        >
            error
        </span>
    );
}

// A span whose parent is outside the fetched set (dropped by retention or
// belonging to another project) renders as a root.
function spanTree(spans: Span[]): SpanRow[] {
    const knownIds = new Set(spans.map((span) => span.spanId));
    const childrenByParent = new Map<string, Span[]>();
    const roots: Span[] = [];
    for (const span of spans) {
        if (span.parentSpanId && knownIds.has(span.parentSpanId)) {
            const siblings = childrenByParent.get(span.parentSpanId) ?? [];
            siblings.push(span);
            childrenByParent.set(span.parentSpanId, siblings);
        } else {
            roots.push(span);
        }
    }
    const rows: SpanRow[] = [];
    function visit(span: Span, depth: number) {
        rows.push({ span, depth });
        for (const child of childrenByParent.get(span.spanId) ?? []) {
            visit(child, depth + 1);
        }
    }
    for (const root of roots) visit(root, 0);
    return rows;
}

function minStart(spans: Span[]): bigint {
    let min = BigInt(spans[0].start);
    for (const span of spans) {
        const start = BigInt(span.start);
        if (start < min) min = start;
    }
    return min;
}

function extentNs(spans: Span[]): bigint {
    const start = minStart(spans);
    let maxEnd = start;
    for (const span of spans) {
        const end = BigInt(span.end);
        if (end > maxEnd) maxEnd = end;
    }
    const extent = maxEnd - start;
    return extent > 0n ? extent : 1n;
}

function percentOf(part: bigint, whole: bigint): number {
    return Number((part * 10_000n) / whole) / 100;
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
    if (res.t === "NotFound") return "Trace not found in this project.";
    if (res.t === "Error" && typeof res.message === "string") return res.message;
    return "Something went wrong reading the trace.";
}

const container: React.CSSProperties = {
    maxWidth: 1100,
    margin: "2rem auto",
    fontFamily: "system-ui",
    padding: "0 16px",
};

const rowStyle: React.CSSProperties = {
    padding: "2px 0",
    borderBottom: "1px solid #f5f5f5",
};

const trackStyle: React.CSSProperties = {
    position: "relative",
    flex: 1,
    height: 20,
    background: "#fafafa",
    margin: "0 10px",
};
