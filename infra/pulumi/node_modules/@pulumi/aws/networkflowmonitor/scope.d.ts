import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Network Flow Monitor Scope.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const example = new aws.networkflowmonitor.Scope("example", {
 *     targets: [{
 *         region: "us-east-1",
 *         targetIdentifier: {
 *             targetType: "ACCOUNT",
 *             targetId: {
 *                 accountId: current.then(current => current.accountId),
 *             },
 *         },
 *     }],
 *     tags: {
 *         Name: "example",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Network Flow Monitor Scope using the scope ID. For example:
 *
 * ```sh
 * $ pulumi import aws:networkflowmonitor/scope:Scope example example-scope-id
 * ```
 */
export declare class Scope extends pulumi.CustomResource {
    /**
     * Get an existing Scope resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ScopeState, opts?: pulumi.CustomResourceOptions): Scope;
    /**
     * Returns true if the given object is an instance of Scope.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Scope;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the scope.
     */
    readonly scopeArn: pulumi.Output<string>;
    /**
     * The identifier for the scope that includes the resources you want to get data results for.
     */
    readonly scopeId: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The targets to define the scope to be monitored. A target is an array of target resources, which are currently Region-account pairs.
     *
     * The following arguments are optional:
     */
    readonly targets: pulumi.Output<outputs.networkflowmonitor.ScopeTarget[] | undefined>;
    readonly timeouts: pulumi.Output<outputs.networkflowmonitor.ScopeTimeouts | undefined>;
    /**
     * Create a Scope resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ScopeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Scope resources.
 */
export interface ScopeState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the scope.
     */
    scopeArn?: pulumi.Input<string>;
    /**
     * The identifier for the scope that includes the resources you want to get data results for.
     */
    scopeId?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The targets to define the scope to be monitored. A target is an array of target resources, which are currently Region-account pairs.
     *
     * The following arguments are optional:
     */
    targets?: pulumi.Input<pulumi.Input<inputs.networkflowmonitor.ScopeTarget>[]>;
    timeouts?: pulumi.Input<inputs.networkflowmonitor.ScopeTimeouts>;
}
/**
 * The set of arguments for constructing a Scope resource.
 */
export interface ScopeArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The targets to define the scope to be monitored. A target is an array of target resources, which are currently Region-account pairs.
     *
     * The following arguments are optional:
     */
    targets?: pulumi.Input<pulumi.Input<inputs.networkflowmonitor.ScopeTarget>[]>;
    timeouts?: pulumi.Input<inputs.networkflowmonitor.ScopeTimeouts>;
}
