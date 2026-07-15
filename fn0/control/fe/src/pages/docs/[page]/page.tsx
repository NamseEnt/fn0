import { DocNotFound, DocsLayout } from "../../../components/DocsLayout";
import type { Props } from "./.props";

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
