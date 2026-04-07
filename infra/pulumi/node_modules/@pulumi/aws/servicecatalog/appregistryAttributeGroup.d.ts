import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Service Catalog AppRegistry Attribute Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicecatalog.AppregistryAttributeGroup("example", {
 *     name: "example",
 *     description: "example description",
 *     attributes: JSON.stringify({
 *         app: "exampleapp",
 *         group: "examplegroup",
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Service Catalog AppRegistry Attribute Group using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:servicecatalog/appregistryAttributeGroup:AppregistryAttributeGroup example 1234567890abcfedhijk09876s
 * ```
 */
export declare class AppregistryAttributeGroup extends pulumi.CustomResource {
    /**
     * Get an existing AppregistryAttributeGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AppregistryAttributeGroupState, opts?: pulumi.CustomResourceOptions): AppregistryAttributeGroup;
    /**
     * Returns true if the given object is an instance of AppregistryAttributeGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AppregistryAttributeGroup;
    /**
     * ARN of the Attribute Group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A JSON string of nested key-value pairs that represents the attributes of the group.
     *
     * The following arguments are optional:
     */
    readonly attributes: pulumi.Output<string>;
    /**
     * Description of the Attribute Group.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Name of the Attribute Group.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags assigned to the Attribute Group. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a AppregistryAttributeGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AppregistryAttributeGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AppregistryAttributeGroup resources.
 */
export interface AppregistryAttributeGroupState {
    /**
     * ARN of the Attribute Group.
     */
    arn?: pulumi.Input<string>;
    /**
     * A JSON string of nested key-value pairs that represents the attributes of the group.
     *
     * The following arguments are optional:
     */
    attributes?: pulumi.Input<string>;
    /**
     * Description of the Attribute Group.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the Attribute Group.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags assigned to the Attribute Group. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a AppregistryAttributeGroup resource.
 */
export interface AppregistryAttributeGroupArgs {
    /**
     * A JSON string of nested key-value pairs that represents the attributes of the group.
     *
     * The following arguments are optional:
     */
    attributes: pulumi.Input<string>;
    /**
     * Description of the Attribute Group.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the Attribute Group.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags assigned to the Attribute Group. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
