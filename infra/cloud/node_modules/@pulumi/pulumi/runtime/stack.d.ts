import { Inputs, Output } from "../output";
import { ComponentResource, ResourceTransform, ResourceTransformation } from "../resource";
import { InvokeTransform } from "../invoke";
/**
 * The type name that should be used to construct the root component in the tree
 * of Pulumi resources allocated by a deployment. This must be kept up to date
 * with `github.com/pulumi/pulumi/sdk/v3/go/common/resource/stack.RootStackType`.
 */
export declare const rootPulumiStackTypeName = "pulumi:pulumi:Stack";
/**
 * Creates a new Pulumi stack resource and executes the callback inside of it.
 * Any outputs returned by the callback will be stored as output properties on
 * this resulting Stack object.
 */
export declare function runInPulumiStack(init: () => Promise<any>): Promise<Inputs | undefined>;
/**
 * {@link Stack} is the root resource for a Pulumi stack. Before invoking the
 * `init` callback, it registers itself as the root resource with the Pulumi
 * engine.
 */
export declare class Stack extends ComponentResource<Inputs> {
    /**
     * The outputs of this stack, if the `init` callback exited normally.
     */
    readonly outputs: Output<Inputs>;
    constructor(init: () => Promise<Inputs>);
    /**
     * Invokes the given `init` callback with this resource set as the root
     * resource. The return value of init is used as the stack's output
     * properties.
     *
     * @param args.init
     *  The callback to run in the context of this Pulumi stack
     */
    initialize(args: {
        init: () => Promise<Inputs>;
    }): Promise<Inputs>;
}
/**
 * Add a transformation to all future resources constructed in this Pulumi
 * stack.
 */
export declare function registerStackTransformation(t: ResourceTransformation): void;
/**
 * Add a transformation to all future resources constructed in this Pulumi
 * stack.
 */
export declare function registerResourceTransform(t: ResourceTransform): void;
/**
 * Add a transformation to all future resources constructed in this Pulumi
 * stack.
 *
 * @deprecated
 *  Use `registerResourceTransform` instead.
 */
export declare function registerStackTransform(t: ResourceTransform): void;
/**
 * Add a transformation to all future invoke calls in this Pulumi stack.
 */
export declare function registerInvokeTransform(t: InvokeTransform): void;
export declare function getStackResource(): Stack | undefined;
