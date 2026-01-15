import { hydrateRoot } from "react-dom/client";
import { routes } from "./routes.generated";
import "./styles/globals.css";

function matchRoute(pathname: string): { route: typeof routes[0]; params: Record<string, string> } | null {
    for (const route of routes) {
        const routeParts = route.path.split("/");
        const pathParts = pathname.split("/");

        if (routeParts.length !== pathParts.length) continue;

        const params: Record<string, string> = {};
        let match = true;

        for (let i = 0; i < routeParts.length; i++) {
            if (routeParts[i].startsWith(":")) {
                params[routeParts[i].slice(1)] = pathParts[i];
            } else if (routeParts[i] !== pathParts[i]) {
                match = false;
                break;
            }
        }

        if (match) {
            return { route, params };
        }
    }
    return null;
}

async function hydrate() {
    const rawProps = (window as any).__FORTE_PROPS__;
    const matched = matchRoute(window.location.pathname);

    if (matched) {
        const [pageModule, schemaModule] = await Promise.all([
            matched.route.component(),
            matched.route.schema(),
        ]);
        const props = schemaModule.PropsSchema.parse(rawProps);
        const element = pageModule.default({ ...props, params: matched.params });
        hydrateRoot(document.getElementById("root")!, element);
    }
}

hydrate();
