import "./styles/globals.css";
import globalsCss from "./styles/globals.css?inline";

const favicon =
    "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'%3E%3Crect width='64' height='64' rx='14' fill='%23654FF0'/%3E%3Ctext x='32' y='44' font-family='Menlo,monospace' font-size='30' font-weight='700' text-anchor='middle' fill='white'%3Ef0%3C/text%3E%3C/svg%3E";

export function Head() {
    return (
        <>
            <meta charSet="utf-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            <title>fn0 — the batteries-included FaaS platform</title>
            <meta
                name="description"
                content="fn0 is an open-source FaaS platform powered by WebAssembly. Build with Rust or TypeScript, deploy in one command — a reasonable alternative to Cloudflare Workers."
            />
            <link rel="icon" href={favicon} />
            <link rel="preconnect" href="https://fonts.googleapis.com" />
            <link
                rel="preconnect"
                href="https://fonts.gstatic.com"
                crossOrigin="anonymous"
            />
            <link
                href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600&family=IBM+Plex+Sans:wght@400;500;600;700&display=swap"
                rel="stylesheet"
            />
            <style dangerouslySetInnerHTML={{ __html: globalsCss }} />
        </>
    );
}
