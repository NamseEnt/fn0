import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Inspector Filter.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.inspector2.Filter("example", {
 *     name: "test",
 *     action: "NONE",
 *     filterCriterias: [{
 *         awsAccountIds: [{
 *             comparison: "EQUALS",
 *             value: "111222333444",
 *         }],
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Inspector Filter using the `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:inspector2/filter:Filter example "arn:aws:inspector2:us-east-1:111222333444:owner/111222333444/filter/abcdefgh12345678"
 * ```
 */
export declare class Filter extends pulumi.CustomResource {
    /**
     * Get an existing Filter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FilterState, opts?: pulumi.CustomResourceOptions): Filter;
    /**
     * Returns true if the given object is an instance of Filter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Filter;
    /**
     * Action to be applied to the findings that maatch the filter. Possible values are `NONE` and `SUPPRESS`
     */
    readonly action: pulumi.Output<string>;
    /**
     * ARN of the Filter.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Description
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Details on the filter criteria. Documented below.
     *
     * The following arguments are optional:
     */
    readonly filterCriterias: pulumi.Output<outputs.inspector2.FilterFilterCriteria[] | undefined>;
    /**
     * Name of the filter.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Reason for creating the filter
     */
    readonly reason: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a Filter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FilterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Filter resources.
 */
export interface FilterState {
    /**
     * Action to be applied to the findings that maatch the filter. Possible values are `NONE` and `SUPPRESS`
     */
    action?: pulumi.Input<string>;
    /**
     * ARN of the Filter.
     */
    arn?: pulumi.Input<string>;
    /**
     * Description
     */
    description?: pulumi.Input<string>;
    /**
     * Details on the filter criteria. Documented below.
     *
     * The following arguments are optional:
     */
    filterCriterias?: pulumi.Input<pulumi.Input<inputs.inspector2.FilterFilterCriteria>[]>;
    /**
     * Name of the filter.
     */
    name?: pulumi.Input<string>;
    /**
     * Reason for creating the filter
     */
    reason?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Filter resource.
 */
export interface FilterArgs {
    /**
     * Action to be applied to the findings that maatch the filter. Possible values are `NONE` and `SUPPRESS`
     */
    action: pulumi.Input<string>;
    /**
     * Description
     */
    description?: pulumi.Input<string>;
    /**
     * Details on the filter criteria. Documented below.
     *
     * The following arguments are optional:
     */
    filterCriterias?: pulumi.Input<pulumi.Input<inputs.inspector2.FilterFilterCriteria>[]>;
    /**
     * Name of the filter.
     */
    name?: pulumi.Input<string>;
    /**
     * Reason for creating the filter
     */
    reason?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
