import type { Props } from "./.props";

export default function IndexPage(props: Props) {
    if (props.t !== "Ok") {
        return <div>Error loading page</div>;
    }

    return (
        <div>
            <h1>fn0 operations canary</h1>
            <p>{props.message}</p>
        </div>
    );
}
