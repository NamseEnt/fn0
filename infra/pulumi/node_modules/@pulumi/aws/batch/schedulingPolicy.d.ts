import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Batch Scheduling Policy resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.batch.SchedulingPolicy("example", {
 *     name: "example",
 *     fairSharePolicy: {
 *         computeReservation: 1,
 *         shareDecaySeconds: 3600,
 *         shareDistributions: [
 *             {
 *                 shareIdentifier: "A1*",
 *                 weightFactor: 0.1,
 *             },
 *             {
 *                 shareIdentifier: "A2",
 *                 weightFactor: 0.2,
 *             },
 *         ],
 *     },
 *     tags: {
 *         Name: "Example Batch Scheduling Policy",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Batch Scheduling Policy using the `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:batch/schedulingPolicy:SchedulingPolicy test_policy arn:aws:batch:us-east-1:123456789012:scheduling-policy/sample
 * ```
 */
export declare class SchedulingPolicy extends pulumi.CustomResource {
    /**
     * Get an existing SchedulingPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SchedulingPolicyState, opts?: pulumi.CustomResourceOptions): SchedulingPolicy;
    /**
     * Returns true if the given object is an instance of SchedulingPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SchedulingPolicy;
    /**
     * The Amazon Resource Name of the scheduling policy.
     */
    readonly arn: pulumi.Output<string>;
    readonly fairSharePolicy: pulumi.Output<outputs.batch.SchedulingPolicyFairSharePolicy | undefined>;
    /**
     * Specifies the name of the scheduling policy.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a SchedulingPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: SchedulingPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SchedulingPolicy resources.
 */
export interface SchedulingPolicyState {
    /**
     * The Amazon Resource Name of the scheduling policy.
     */
    arn?: pulumi.Input<string>;
    fairSharePolicy?: pulumi.Input<inputs.batch.SchedulingPolicyFairSharePolicy>;
    /**
     * Specifies the name of the scheduling policy.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a SchedulingPolicy resource.
 */
export interface SchedulingPolicyArgs {
    fairSharePolicy?: pulumi.Input<inputs.batch.SchedulingPolicyFairSharePolicy>;
    /**
     * Specifies the name of the scheduling policy.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
