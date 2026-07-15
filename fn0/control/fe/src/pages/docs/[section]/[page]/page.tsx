import { DocNotFound, DocsLayout } from "../../../../components/DocsLayout";
import type { Props } from "./.props";

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
