import { DocsLayout } from "../../components/DocsLayout";
import type { Props } from "./.props";

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
