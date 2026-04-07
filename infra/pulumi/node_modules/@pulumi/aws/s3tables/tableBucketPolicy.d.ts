import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an Amazon S3 Tables Table Bucket Policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.iam.getPolicyDocument({
 *     statements: [{}],
 * });
 * const exampleTableBucketPolicy = new aws.s3tables.TableBucketPolicy("example", {
 *     resourcePolicy: example.then(example => example.json),
 *     tableBucketArn: exampleAwsS3tablesTableBucket.arn,
 * });
 * const test = new aws.s3tables.TableBucket("test", {name: "example-bucket"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import S3 Tables Table Bucket Policy using the `table_bucket_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:s3tables/tableBucketPolicy:TableBucketPolicy example 'arn:aws:s3tables:us-west-2:123456789012:bucket/example-bucket;example-namespace'
 * ```
 */
export declare class TableBucketPolicy extends pulumi.CustomResource {
    /**
     * Get an existing TableBucketPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TableBucketPolicyState, opts?: pulumi.CustomResourceOptions): TableBucketPolicy;
    /**
     * Returns true if the given object is an instance of TableBucketPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TableBucketPolicy;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Amazon Web Services resource-based policy document in JSON format.
     */
    readonly resourcePolicy: pulumi.Output<string>;
    /**
     * ARN referencing the Table Bucket that owns this policy.
     */
    readonly tableBucketArn: pulumi.Output<string>;
    /**
     * Create a TableBucketPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TableBucketPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TableBucketPolicy resources.
 */
export interface TableBucketPolicyState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Web Services resource-based policy document in JSON format.
     */
    resourcePolicy?: pulumi.Input<string>;
    /**
     * ARN referencing the Table Bucket that owns this policy.
     */
    tableBucketArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TableBucketPolicy resource.
 */
export interface TableBucketPolicyArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Web Services resource-based policy document in JSON format.
     */
    resourcePolicy: pulumi.Input<string>;
    /**
     * ARN referencing the Table Bucket that owns this policy.
     */
    tableBucketArn: pulumi.Input<string>;
}
