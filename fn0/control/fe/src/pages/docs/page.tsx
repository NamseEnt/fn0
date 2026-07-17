import type { HeadDescriptor } from "@forte/react";
import { DocsLayout } from "../../components/DocsLayout";
import type { Props } from "./.props";

export function head(props: Props): HeadDescriptor[] {
    return [
        { title: props.title },
        { property: "og:title", content: props.title },
        { property: "og:url", content: "https://fn0.dev/docs" },
    ];
}

export default function DocsIndexPage(props: Props) {
    return (
        <DocsLayout nav={props.nav} route="" title={props.title}>
            <article
                className="doc-prose"
                dangerouslySetInnerHTML={{ __html: props.html }}
            />
        </DocsLayout>
    );
}
