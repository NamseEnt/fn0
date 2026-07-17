import type { HeadDescriptor } from "@forte/react";
import { DocNotFound, DocsLayout } from "../../../../components/DocsLayout";
import type { Props } from "./.props";

export function head(
    props: Props & { params: { section: string; page: string } },
): HeadDescriptor[] {
    if (props.t !== "Ok") return [{ title: "Not found — fn0 docs" }];
    const title = `${props.title} — fn0 docs`;
    return [
        { title },
        { property: "og:title", content: title },
        {
            property: "og:url",
            content: `https://fn0.dev/docs/${props.params.section}/${props.params.page}`,
        },
    ];
}

export default function DocsSectionPage(
    props: Props & { params: { section: string; page: string } },
) {
    const route = `${props.params.section}/${props.params.page}`;
    if (props.t !== "Ok") {
        return (
            <DocsLayout nav={props.nav} route={route} title={null}>
                <DocNotFound />
            </DocsLayout>
        );
    }
    return (
        <DocsLayout nav={props.nav} route={route} title={props.title}>
            <article
                className="doc-prose"
                dangerouslySetInnerHTML={{ __html: props.html }}
            />
        </DocsLayout>
    );
}
