import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a DocumentDB Cluster Parameter Group
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.docdb.ClusterParameterGroup("example", {
 *     family: "docdb3.6",
 *     name: "example",
 *     description: "docdb cluster parameter group",
 *     parameters: [{
 *         name: "tls",
 *         value: "enabled",
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DocumentDB Cluster Parameter Groups using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:docdb/clusterParameterGroup:ClusterParameterGroup cluster_pg production-pg-1
 * ```
 */
export declare class ClusterParameterGroup extends pulumi.CustomResource {
    /**
     * Get an existing ClusterParameterGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClusterParameterGroupState, opts?: pulumi.CustomResourceOptions): ClusterParameterGroup;
    /**
     * Returns true if the given object is an instance of ClusterParameterGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ClusterParameterGroup;
    /**
     * The ARN of the DocumentDB cluster parameter group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The description of the DocumentDB cluster parameter group. Defaults to "Managed by Pulumi".
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The family of the DocumentDB cluster parameter group.
     */
    readonly family: pulumi.Output<string>;
    /**
     * The name of the DocumentDB parameter.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * A list of DocumentDB parameters to apply. Setting parameters to system default values may show a difference on imported resources.
     */
    readonly parameters: pulumi.Output<outputs.docdb.ClusterParameterGroupParameter[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a ClusterParameterGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ClusterParameterGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ClusterParameterGroup resources.
 */
export interface ClusterParameterGroupState {
    /**
     * The ARN of the DocumentDB cluster parameter group.
     */
    arn?: pulumi.Input<string>;
    /**
     * The description of the DocumentDB cluster parameter group. Defaults to "Managed by Pulumi".
     */
    description?: pulumi.Input<string>;
    /**
     * The family of the DocumentDB cluster parameter group.
     */
    family?: pulumi.Input<string>;
    /**
     * The name of the DocumentDB parameter.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * A list of DocumentDB parameters to apply. Setting parameters to system default values may show a difference on imported resources.
     */
    parameters?: pulumi.Input<pulumi.Input<inputs.docdb.ClusterParameterGroupParameter>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a ClusterParameterGroup resource.
 */
export interface ClusterParameterGroupArgs {
    /**
     * The description of the DocumentDB cluster parameter group. Defaults to "Managed by Pulumi".
     */
    description?: pulumi.Input<string>;
    /**
     * The family of the DocumentDB cluster parameter group.
     */
    family: pulumi.Input<string>;
    /**
     * The name of the DocumentDB parameter.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * A list of DocumentDB parameters to apply. Setting parameters to system default values may show a difference on imported resources.
     */
    parameters?: pulumi.Input<pulumi.Input<inputs.docdb.ClusterParameterGroupParameter>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
