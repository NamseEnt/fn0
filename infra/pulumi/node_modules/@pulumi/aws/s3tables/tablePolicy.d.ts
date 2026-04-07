import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an Amazon S3 Tables Table Policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ## Import
 *
 * Using `pulumi import`, import S3 Tables Table Policy using the `table_bucket_arn`, the value of `namespace`, and the value of `name`, separated by a semicolon (`;`). For example:
 *
 * ```sh
 * $ pulumi import aws:s3tables/tablePolicy:TablePolicy example 'arn:aws:s3tables:us-west-2:123456789012:bucket/example-bucket;example-namespace;example-table'
 * ```
 */
export declare class TablePolicy extends pulumi.CustomResource {
    /**
     * Get an existing TablePolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TablePolicyState, opts?: pulumi.CustomResourceOptions): TablePolicy;
    /**
     * Returns true if the given object is an instance of TablePolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TablePolicy;
    /**
     * Name of the table.
     * Must be between 1 and 255 characters in length.
     * Can consist of lowercase letters, numbers, and underscores, and must begin and end with a lowercase letter or number.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Name of the namespace for this table.
     * Must be between 1 and 255 characters in length.
     * Can consist of lowercase letters, numbers, and underscores, and must begin and end with a lowercase letter or number.
     */
    readonly namespace: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Amazon Web Services resource-based policy document in JSON format.
     */
    readonly resourcePolicy: pulumi.Output<string>;
    /**
     * ARN referencing the Table Bucket that contains this Namespace.
     */
    readonly tableBucketArn: pulumi.Output<string>;
    /**
     * Create a TablePolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TablePolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TablePolicy resources.
 */
export interface TablePolicyState {
    /**
     * Name of the table.
     * Must be between 1 and 255 characters in length.
     * Can consist of lowercase letters, numbers, and underscores, and must begin and end with a lowercase letter or number.
     */
    name?: pulumi.Input<string>;
    /**
     * Name of the namespace for this table.
     * Must be between 1 and 255 characters in length.
     * Can consist of lowercase letters, numbers, and underscores, and must begin and end with a lowercase letter or number.
     */
    namespace?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Web Services resource-based policy document in JSON format.
     */
    resourcePolicy?: pulumi.Input<string>;
    /**
     * ARN referencing the Table Bucket that contains this Namespace.
     */
    tableBucketArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TablePolicy resource.
 */
export interface TablePolicyArgs {
    /**
     * Name of the table.
     * Must be between 1 and 255 characters in length.
     * Can consist of lowercase letters, numbers, and underscores, and must begin and end with a lowercase letter or number.
     */
    name?: pulumi.Input<string>;
    /**
     * Name of the namespace for this table.
     * Must be between 1 and 255 characters in length.
     * Can consist of lowercase letters, numbers, and underscores, and must begin and end with a lowercase letter or number.
     */
    namespace: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Web Services resource-based policy document in JSON format.
     */
    resourcePolicy: pulumi.Input<string>;
    /**
     * ARN referencing the Table Bucket that contains this Namespace.
     */
    tableBucketArn: pulumi.Input<string>;
}
