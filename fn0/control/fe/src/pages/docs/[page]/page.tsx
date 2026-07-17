import type { HeadDescriptor } from "@forte/react";
import { DocNotFound, DocsLayout } from "../../../components/DocsLayout";
import type { Props } from "./.props";

export function head(
    props: Props & { params: { page: string } },
): HeadDescriptor[] {
    if (props.t !== "Ok") return [{ title: "Not found — fn0 docs" }];
    const title = `${props.title} — fn0 docs`;
    return [
        { title },
        { property: "og:title", content: title },
        { property: "og:url", content: `https://fn0.dev/docs/${props.params.page}` },
    ];
}

export default function DocsPage(props: Props & { params: { page: string } }) {
    if (props.t !== "Ok") {
        return (
            <DocsLayout nav={props.nav} route={props.params.page} title={null}>
                <DocNotFound />
            </DocsLayout>
        );
    }
    return (
        <DocsLayout nav={props.nav} route={props.params.page} title={props.title}>
            <article
                className="doc-prose"
                dangerouslySetInnerHTML={{ __html: props.html }}
            />
        </DocsLayout>
    );
}
