import { useEffect, useState } from "react";
import type { Props } from "./.props";
import { issueToken } from "../../actions/.generated/issue_token";
import { listTokens } from "../../actions/.generated/list_tokens";
import { revokeToken } from "../../actions/.generated/revoke_token";

type TokenRow = {
    id: string;
    label: string;
    createdAt: string;
};

export default function TokensPage(props: Props) {
    const [tokens, setTokens] = useState<TokenRow[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [label, setLabel] = useState("");
    const [issuedToken, setIssuedToken] = useState<string | null>(null);
    const [busy, setBusy] = useState(false);

    async function refresh() {
        setLoading(true);
        const res = await listTokens({});
        if (res.t === "Ok") {
            setTokens(res.tokens);
            setError(null);
        } else {
            setError("Not signed in.");
        }
        setLoading(false);
    }

    useEffect(() => {
        refresh();
    }, []);

    async function onIssue(e: React.FormEvent) {
        e.preventDefault();
        if (!label.trim() || busy) return;
        setBusy(true);
        const res = await issueToken({ label: label.trim() });
        setBusy(false);
        if (res.t === "Ok") {
            setIssuedToken(res.token);
            setLabel("");
            refresh();
        } else if (res.t === "Error") {
            setError(res.message);
        } else {
            setError("Not signed in.");
        }
    }

    async function onRevoke(id: string) {
        if (busy) return;
        if (!confirm("Revoke this token? CLIs using it will stop working.")) return;
        setBusy(true);
        const res = await revokeToken({ id });
        setBusy(false);
        if (res.t === "Ok" || res.t === "NotFound") {
            refresh();
        } else if (res.t === "Error") {
            setError(res.message);
        } else {
            setError("Not signed in.");
        }
    }

    return (
        <div style={{ maxWidth: 720, margin: "2rem auto", fontFamily: "system-ui" }}>
            <h1>fn0 tokens</h1>
            <p>
                Signed in as <strong>{props.githubLogin}</strong> (id {props.githubId})
            </p>

            <h2>Issue a new token</h2>
            <form onSubmit={onIssue} style={{ display: "flex", gap: 8 }}>
                <input
                    type="text"
                    placeholder="label (e.g. laptop, ci)"
                    value={label}
                    onChange={(e) => setLabel(e.target.value)}
                    style={{ flex: 1, padding: 6 }}
                    disabled={busy}
                />
                <button type="submit" disabled={busy || !label.trim()}>
                    Issue
                </button>
            </form>

            {issuedToken && (
                <div
                    style={{
                        marginTop: 12,
                        padding: 12,
                        border: "1px solid #888",
                        background: "#fffbe6",
                    }}
                >
                    <p>
                        <strong>Copy this token now.</strong> You will not see it again.
                    </p>
                    <code style={{ wordBreak: "break-all" }}>{issuedToken}</code>
                    <div style={{ marginTop: 8 }}>
                        <button onClick={() => setIssuedToken(null)}>Done</button>
                    </div>
                </div>
            )}

            <h2 style={{ marginTop: 32 }}>Existing tokens</h2>
            {loading ? (
                <p>Loading…</p>
            ) : tokens.length === 0 ? (
                <p>No tokens yet.</p>
            ) : (
                <table style={{ width: "100%", borderCollapse: "collapse" }}>
                    <thead>
                        <tr>
                            <th style={cell}>Label</th>
                            <th style={cell}>Created</th>
                            <th style={cell}>Token id</th>
                            <th style={cell}></th>
                        </tr>
                    </thead>
                    <tbody>
                        {tokens.map((t) => (
                            <tr key={t.id}>
                                <td style={cell}>{t.label}</td>
                                <td style={cell}>{new Date(t.createdAt).toLocaleString()}</td>
                                <td style={{ ...cell, fontFamily: "monospace", fontSize: 12 }}>
                                    {t.id}
                                </td>
                                <td style={cell}>
                                    <button onClick={() => onRevoke(t.id)} disabled={busy}>
                                        Revoke
                                    </button>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            )}

            {error && (
                <p style={{ color: "crimson", marginTop: 16 }}>{error}</p>
            )}
        </div>
    );
}

const cell: React.CSSProperties = {
    padding: 8,
    borderBottom: "1px solid #eee",
    textAlign: "left",
};
