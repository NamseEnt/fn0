import * as pulumi from "@pulumi/pulumi";
/**
 * Creates an LF-Tag with the specified name and values. Each key must have at least one value. The maximum number of values permitted is 1000.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lakeformation.LfTag("example", {
 *     key: "module",
 *     values: [
 *         "Orders",
 *         "Sales",
 *         "Customers",
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Lake Formation LF-Tags using the `catalog_id:key`. If you have not set a Catalog ID specify the AWS Account ID that the database is in. For example:
 *
 * ```sh
 * $ pulumi import aws:lakeformation/lfTag:LfTag example 123456789012:some_key
 * ```
 */
export declare class LfTag extends pulumi.CustomResource {
    /**
     * Get an existing LfTag resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LfTagState, opts?: pulumi.CustomResourceOptions): LfTag;
    /**
     * Returns true if the given object is an instance of LfTag.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LfTag;
    /**
     * ID of the Data Catalog to create the tag in. If omitted, this defaults to the AWS Account ID.
     */
    readonly catalogId: pulumi.Output<string>;
    /**
     * Key-name for the tag.
     */
    readonly key: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * List of possible values an attribute can take.
     */
    readonly values: pulumi.Output<string[]>;
    /**
     * Create a LfTag resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LfTagArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LfTag resources.
 */
export interface LfTagState {
    /**
     * ID of the Data Catalog to create the tag in. If omitted, this defaults to the AWS Account ID.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * Key-name for the tag.
     */
    key?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of possible values an attribute can take.
     */
    values?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a LfTag resource.
 */
export interface LfTagArgs {
    /**
     * ID of the Data Catalog to create the tag in. If omitted, this defaults to the AWS Account ID.
     */
    catalogId?: pulumi.Input<string>;
    /**
     * Key-name for the tag.
     */
    key: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of possible values an attribute can take.
     */
    values: pulumi.Input<pulumi.Input<string>[]>;
}
