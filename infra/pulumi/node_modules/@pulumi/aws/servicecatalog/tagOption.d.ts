import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Service Catalog Tag Option.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicecatalog.TagOption("example", {
 *     key: "nyckel",
 *     value: "värde",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_servicecatalog_tag_option` using the tag option ID. For example:
 *
 * ```sh
 * $ pulumi import aws:servicecatalog/tagOption:TagOption example tag-pjtvagohlyo3m
 * ```
 */
export declare class TagOption extends pulumi.CustomResource {
    /**
     * Get an existing TagOption resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TagOptionState, opts?: pulumi.CustomResourceOptions): TagOption;
    /**
     * Returns true if the given object is an instance of TagOption.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TagOption;
    /**
     * Whether tag option is active. Default is `true`.
     */
    readonly active: pulumi.Output<boolean | undefined>;
    /**
     * Tag option key.
     */
    readonly key: pulumi.Output<string>;
    readonly owner: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Tag option value.
     *
     * The following arguments are optional:
     */
    readonly value: pulumi.Output<string>;
    /**
     * Create a TagOption resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TagOptionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TagOption resources.
 */
export interface TagOptionState {
    /**
     * Whether tag option is active. Default is `true`.
     */
    active?: pulumi.Input<boolean>;
    /**
     * Tag option key.
     */
    key?: pulumi.Input<string>;
    owner?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tag option value.
     *
     * The following arguments are optional:
     */
    value?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TagOption resource.
 */
export interface TagOptionArgs {
    /**
     * Whether tag option is active. Default is `true`.
     */
    active?: pulumi.Input<boolean>;
    /**
     * Tag option key.
     */
    key: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tag option value.
     *
     * The following arguments are optional:
     */
    value: pulumi.Input<string>;
}
