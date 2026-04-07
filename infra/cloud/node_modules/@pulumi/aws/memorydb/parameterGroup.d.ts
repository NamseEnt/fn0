import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a MemoryDB Parameter Group.
 *
 * More information about parameter groups can be found in the [MemoryDB User Guide](https://docs.aws.amazon.com/memorydb/latest/devguide/parametergroups.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.memorydb.ParameterGroup("example", {
 *     name: "my-parameter-group",
 *     family: "memorydb_redis6",
 *     parameters: [{
 *         name: "activedefrag",
 *         value: "yes",
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a parameter group using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:memorydb/parameterGroup:ParameterGroup example my-parameter-group
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
     * The ARN of the parameter group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Description for the parameter group. Defaults to `"Managed by Pulumi"`.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The engine version that the parameter group can be used with.
     *
     * The following arguments are optional:
     */
    readonly family: pulumi.Output<string>;
    /**
     * Name of the parameter group. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * Set of MemoryDB parameters to apply. Any parameters not specified will fall back to their family defaults. Detailed below.
     */
    readonly parameters: pulumi.Output<outputs.memorydb.ParameterGroupParameter[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * The ARN of the parameter group.
     */
    arn?: pulumi.Input<string>;
    /**
     * Description for the parameter group. Defaults to `"Managed by Pulumi"`.
     */
    description?: pulumi.Input<string>;
    /**
     * The engine version that the parameter group can be used with.
     *
     * The following arguments are optional:
     */
    family?: pulumi.Input<string>;
    /**
     * Name of the parameter group. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Set of MemoryDB parameters to apply. Any parameters not specified will fall back to their family defaults. Detailed below.
     */
    parameters?: pulumi.Input<pulumi.Input<inputs.memorydb.ParameterGroupParameter>[]>;
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
     * Description for the parameter group. Defaults to `"Managed by Pulumi"`.
     */
    description?: pulumi.Input<string>;
    /**
     * The engine version that the parameter group can be used with.
     *
     * The following arguments are optional:
     */
    family: pulumi.Input<string>;
    /**
     * Name of the parameter group. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Set of MemoryDB parameters to apply. Any parameters not specified will fall back to their family defaults. Detailed below.
     */
    parameters?: pulumi.Input<pulumi.Input<inputs.memorydb.ParameterGroupParameter>[]>;
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
}
