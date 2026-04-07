import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an S3 bucket request payment configuration resource. For more information, see [Requester Pays Buckets](https://docs.aws.amazon.com/AmazonS3/latest/dev/RequesterPaysBuckets.html).
 *
 * > **NOTE:** Destroying an `aws.s3.BucketRequestPaymentConfiguration` resource resets the bucket's `payer` to the S3 default: the bucket owner.
 *
 * > This resource cannot be used with S3 directory buckets.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.BucketRequestPaymentConfiguration("example", {
 *     bucket: exampleAwsS3Bucket.id,
 *     payer: "Requester",
 * });
 * ```
 *
 * ## Import
 *
 * If the owner (account ID) of the source bucket differs from the account used to configure the AWS Provider, import using the `bucket` and `expected_bucket_owner` separated by a comma (`,`):
 *
 * __Using `pulumi import` to import__ S3 bucket request payment configuration using the `bucket` or using the `bucket` and `expected_bucket_owner` separated by a comma (`,`). For example:
 *
 * If the owner (account ID) of the source bucket is the same account used to configure the AWS Provider, import using the `bucket`:
 *
 * ```sh
 * $ pulumi import aws:s3/bucketRequestPaymentConfiguration:BucketRequestPaymentConfiguration example bucket-name
 * ```
 * If the owner (account ID) of the source bucket differs from the account used to configure the AWS Provider, import using the `bucket` and `expected_bucket_owner` separated by a comma (`,`):
 *
 * ```sh
 * $ pulumi import aws:s3/bucketRequestPaymentConfiguration:BucketRequestPaymentConfiguration example bucket-name,123456789012
 * ```
 */
export declare class BucketRequestPaymentConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing BucketRequestPaymentConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BucketRequestPaymentConfigurationState, opts?: pulumi.CustomResourceOptions): BucketRequestPaymentConfiguration;
    /**
     * Returns true if the given object is an instance of BucketRequestPaymentConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BucketRequestPaymentConfiguration;
    /**
     * Name of the bucket.
     */
    readonly bucket: pulumi.Output<string>;
    /**
     * Account ID of the expected bucket owner.
     */
    readonly expectedBucketOwner: pulumi.Output<string | undefined>;
    /**
     * Specifies who pays for the download and request fees. Valid values: `BucketOwner`, `Requester`.
     */
    readonly payer: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a BucketRequestPaymentConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BucketRequestPaymentConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BucketRequestPaymentConfiguration resources.
 */
export interface BucketRequestPaymentConfigurationState {
    /**
     * Name of the bucket.
     */
    bucket?: pulumi.Input<string>;
    /**
     * Account ID of the expected bucket owner.
     */
    expectedBucketOwner?: pulumi.Input<string>;
    /**
     * Specifies who pays for the download and request fees. Valid values: `BucketOwner`, `Requester`.
     */
    payer?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a BucketRequestPaymentConfiguration resource.
 */
export interface BucketRequestPaymentConfigurationArgs {
    /**
     * Name of the bucket.
     */
    bucket: pulumi.Input<string>;
    /**
     * Account ID of the expected bucket owner.
     */
    expectedBucketOwner?: pulumi.Input<string>;
    /**
     * Specifies who pays for the download and request fees. Valid values: `BucketOwner`, `Requester`.
     */
    payer: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
