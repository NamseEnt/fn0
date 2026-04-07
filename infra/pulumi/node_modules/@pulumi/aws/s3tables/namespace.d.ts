import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an Amazon S3 Tables Namespace.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleTableBucket = new aws.s3tables.TableBucket("example", {name: "example-bucket"});
 * const example = new aws.s3tables.Namespace("example", {
 *     namespace: "example_namespace",
 *     tableBucketArn: exampleTableBucket.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import S3 Tables Namespace using the `table_bucket_arn` and the value of `namespace`, separated by a semicolon (`;`). For example:
 *
 * ```sh
 * $ pulumi import aws:s3tables/namespace:Namespace example 'arn:aws:s3tables:us-west-2:123456789012:bucket/example-bucket;example-namespace'
 * ```
 */
export declare class Namespace extends pulumi.CustomResource {
    /**
     * Get an existing Namespace resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NamespaceState, opts?: pulumi.CustomResourceOptions): Namespace;
    /**
     * Returns true if the given object is an instance of Namespace.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Namespace;
    /**
     * Date and time when the namespace was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Account ID of the account that created the namespace.
     */
    readonly createdBy: pulumi.Output<string>;
    /**
     * Name of the namespace.
     * Must be between 1 and 255 characters in length.
     * Can consist of lowercase letters, numbers, and underscores, and must begin and end with a lowercase letter or number.
     */
    readonly namespace: pulumi.Output<string>;
    /**
     * Account ID of the account that owns the namespace.
     */
    readonly ownerAccountId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN referencing the Table Bucket that contains this Namespace.
     */
    readonly tableBucketArn: pulumi.Output<string>;
    /**
     * Create a Namespace resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NamespaceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Namespace resources.
 */
export interface NamespaceState {
    /**
     * Date and time when the namespace was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Account ID of the account that created the namespace.
     */
    createdBy?: pulumi.Input<string>;
    /**
     * Name of the namespace.
     * Must be between 1 and 255 characters in length.
     * Can consist of lowercase letters, numbers, and underscores, and must begin and end with a lowercase letter or number.
     */
    namespace?: pulumi.Input<string>;
    /**
     * Account ID of the account that owns the namespace.
     */
    ownerAccountId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN referencing the Table Bucket that contains this Namespace.
     */
    tableBucketArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Namespace resource.
 */
export interface NamespaceArgs {
    /**
     * Name of the namespace.
     * Must be between 1 and 255 characters in length.
     * Can consist of lowercase letters, numbers, and underscores, and must begin and end with a lowercase letter or number.
     */
    namespace: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN referencing the Table Bucket that contains this Namespace.
     */
    tableBucketArn: pulumi.Input<string>;
}
