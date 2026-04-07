import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an ElastiCache parameter group resource.
 *
 * > **NOTE:** Attempting to remove the `reserved-memory` parameter when `family` is set to `redis2.6` or `redis2.8` may show a perpetual difference in this provider due to an ElastiCache API limitation. Leave that parameter configured with any value to workaround the issue.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.elasticache.ParameterGroup("default", {
 *     name: "cache-params",
 *     family: "redis2.8",
 *     parameters: [
 *         {
 *             name: "activerehashing",
 *             value: "yes",
 *         },
 *         {
 *             name: "min-slaves-to-write",
 *             value: "2",
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import ElastiCache Parameter Groups using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:elasticache/parameterGroup:ParameterGroup default redis-params
 * ```
 */
export declare class ParameterGroup extends pulumi.CustomResource {
    /**
     * Get an existing ParameterGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ParameterGroupState, opts?: pulumi.CustomResourceOptions): ParameterGroup;
    /**
     * Returns true if the given object is an instance of ParameterGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ParameterGroup;
    /**
     * The AWS ARN associated with the parameter group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The description of the ElastiCache parameter group. Defaults to "Managed by Pulumi".
     */
    readonly description: pulumi.Output<string>;
    /**
     * The family of the ElastiCache parameter group.
     */
    readonly family: pulumi.Output<string>;
    /**
     * The name of the ElastiCache parameter.
     */
    readonly name: pulumi.Output<string>;
    /**
     * A list of ElastiCache parameters to apply.
     */
    readonly parameters: pulumi.Output<outputs.elasticache.ParameterGroupParameter[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
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
     * Create a ParameterGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ParameterGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ParameterGroup resources.
 */
export interface ParameterGroupState {
    /**
     * The AWS ARN associated with the parameter group.
     */
    arn?: pulumi.Input<string>;
    /**
     * The description of the ElastiCache parameter group. Defaults to "Managed by Pulumi".
     */
    description?: pulumi.Input<string>;
    /**
     * The family of the ElastiCache parameter group.
     */
    family?: pulumi.Input<string>;
    /**
     * The name of the ElastiCache parameter.
     */
    name?: pulumi.Input<string>;
    /**
     * A list of ElastiCache parameters to apply.
     */
    parameters?: pulumi.Input<pulumi.Input<inputs.elasticache.ParameterGroupParameter>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
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
 * The set of arguments for constructing a ParameterGroup resource.
 */
export interface ParameterGroupArgs {
    /**
     * The description of the ElastiCache parameter group. Defaults to "Managed by Pulumi".
     */
    description?: pulumi.Input<string>;
    /**
     * The family of the ElastiCache parameter group.
     */
    family: pulumi.Input<string>;
    /**
     * The name of the ElastiCache parameter.
     */
    name?: pulumi.Input<string>;
    /**
     * A list of ElastiCache parameters to apply.
     */
    parameters?: pulumi.Input<pulumi.Input<inputs.elasticache.ParameterGroupParameter>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
