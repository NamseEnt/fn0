import { Input, Output } from "./output";
import { CustomResource, CustomResourceOptions } from "./resource";
/**
 * Stash stores an arbitrary value in the state.
 */
export declare class Stash extends CustomResource {
    /**
     * The value saved in the state for the stash.
     */
    readonly output: Output<any>;
    /**
     * The most recent value passed to the stash resource.
     */
    readonly input: Output<any>;
    /**
     * Create a {@link Stash} resource with the given arguments, and options.
     *
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: StashArgs, opts?: CustomResourceOptions);
}
/**
 * The set of arguments for constructing a {@link Stash} resource.
 */
export interface StashArgs {
    /**
     * The value to store in the stash resource.
     */
    readonly input: Input<any>;
}
