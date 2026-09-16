import { useState } from "react";
import type { Props } from "./.props";
import { approveCliAuthorization } from "../../../../actions/.generated/approve_cli_authorization";

export default function AuthorizeCliPage(props: Props) {
    const [label, setLabel] = useState(props.defaultLabel);
    const [busy, setBusy] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [authorizationCode, setAuthorizationCode] = useState<string | null>(null);
    const [copied, setCopied] = useState(false);
    const [showCode, setShowCode] = useState(false);

    async function onApprove(e: React.FormEvent) {
        e.preventDefault();
        if (busy) return;
        const trimmed = label.trim();
        if (!trimmed) {
            setError("label cannot be empty");
            return;
        }
        setBusy(true);
        setError(null);
        const res = await approveCliAuthorization({
            responseMode: props.manualCode ? "code" : undefined,
            redirectUri: props.redirectUri ?? undefined,
            codeChallenge: props.codeChallenge,
            codeChallengeMethod: props.codeChallengeMethod,
            state: props.state ?? undefined,
            label: trimmed,
        });
        if (res.t === "Ok") {
            if (res.redirectTo) {
                window.location.replace(res.redirectTo);
                return;
            }
            setAuthorizationCode(res.code);
            setCopied(false);
            setShowCode(false);
            setBusy(false);
            return;
        }
        setBusy(false);
        if (res.t === "InvalidRequest" || res.t === "Error") {
            setError(res.message);
        } else if (res.t === "NotLoggedIn") {
            setError("Not signed in.");
        }
    }

    async function onCopyCode() {
        if (!authorizationCode) return;
        try {
            await navigator.clipboard.writeText(authorizationCode);
            setCopied(true);
            setError(null);
        } catch {
            setError("Could not copy the code. Use Show code and copy it manually.");
        }
    }

    if (authorizationCode) {
        return (
            <div style={{ maxWidth: 540, margin: "3rem auto", fontFamily: "system-ui" }}>
                <h1>Authorization approved</h1>
                <p>Copy the one-time code and paste it into the waiting CLI prompt.</p>
                <p>This code expires in five minutes and can be used only once.</p>
                {showCode && (
                    <code style={{ display: "block", wordBreak: "break-all", margin: "1rem 0" }}>
                        {authorizationCode}
                    </code>
                )}
                <div style={{ display: "flex", gap: 8 }}>
                    <button type="button" onClick={onCopyCode}>
                        {copied ? "Copied" : "Copy one-time code"}
                    </button>
                    <button type="button" onClick={() => setShowCode((visible) => !visible)}>
                        {showCode ? "Hide code" : "Show code"}
                    </button>
                </div>
                {error && <p style={{ color: "crimson", marginTop: 16 }}>{error}</p>}
            </div>
        );
    }

    return (
        <div style={{ maxWidth: 540, margin: "3rem auto", fontFamily: "system-ui" }}>
            <h1>Authorize CLI</h1>
            <p>A CLI is requesting access to your fn0 account.</p>
            <p>
                Signed in as <strong>{props.githubLogin}</strong>.
            </p>

            {!props.manualCode && props.redirectUri && (
                <div
                    style={{
                        margin: "1rem 0",
                        padding: 12,
                        border: "1px solid #ddd",
                        background: "#fafafa",
                        fontSize: 14,
                    }}
                >
                    <div style={{ marginBottom: 6 }}>
                        <strong>Will redirect back to:</strong>
                    </div>
                    <code style={{ wordBreak: "break-all" }}>{props.redirectUri}</code>
                </div>
            )}

            {props.manualCode && (
                <p>
                    After approval, this page will provide a one-time code for the CLI.
                </p>
            )}

            <form onSubmit={onApprove}>
                <label style={{ display: "block", marginBottom: 4 }}>
                    Label for this CLI token
                </label>
                <input
                    type="text"
                    value={label}
                    onChange={(e) => setLabel(e.target.value)}
                    placeholder="laptop"
                    style={{ width: "100%", padding: 8, marginBottom: 12 }}
                    disabled={busy}
                />
                <div style={{ display: "flex", gap: 8 }}>
                    <button type="submit" disabled={busy || !label.trim()}>
                        {busy ? "Authorizing…" : "Approve"}
                    </button>
                    <button type="button" onClick={() => window.location.replace("/")} disabled={busy}>
                        Cancel
                    </button>
                </div>
            </form>

            {error && <p style={{ color: "crimson", marginTop: 16 }}>{error}</p>}
        </div>
    );
}
