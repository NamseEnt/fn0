import type { Props } from "./.props";
import { useGreet } from "../../hooks/.generated/useGreet";

export default function IndexPage(props: Props) {
    if (props.t !== "Ok") {
        return <div>Error loading page</div>;
    }

    const greet = useGreet({ name: "Workers" });

    return (
        <div>
            <h1>Welcome to Forte</h1>
            <p>{props.message}</p>
            <p data-testid="hook">{greet.greeting} ({greet.random})</p>
        </div>
    );
}
