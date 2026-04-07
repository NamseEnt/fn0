import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages Amazon S3 Metadata for a bucket.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.BucketMetadataConfiguration("example", {
 *     bucket: exampleAwsS3Bucket.bucket,
 *     metadataConfiguration: {
 *         inventoryTableConfiguration: {
 *             configurationState: "ENABLED",
 *         },
 *         journalTableConfiguration: {
 *             recordExpiration: {
 *                 days: 7,
 *                 expiration: "ENABLED",
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * If the owner (account ID) of the source bucket differs from the account used to configure the Terraform AWS Provider, import using the `bucket` and `expected_bucket_owner` separated by a comma (`,`):
 *
 * __Using `pulumi import` to import__ S3 bucket metadata configuration using the `bucket` or using the `bucket` and `expected_bucket_owner` separated by a comma (`,`). For example:
 *
 * If the owner (account ID) of the source bucket is the same account used to configure the Terraform AWS Provider, import using the `bucket`:
 *
 * ```sh
 * $ pulumi import aws:s3/bucketMetadataConfiguration:BucketMetadataConfiguration example bucket-name
 * ```
 * If the owner (account ID) of the source bucket differs from the account used to configure the Terraform AWS Provider, import using the `bucket` and `expected_bucket_owner` separated by a comma (`,`):
 *
 * ```sh
 * $ pulumi import aws:s3/bucketMetadataConfiguration:BucketMetadataConfiguration example bucket-name,123456789012
 * ```
 */
export declare class BucketMetadataConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing BucketMetadataConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BucketMetadataConfigurationState, opts?: pulumi.CustomResourceOptions): BucketMetadataConfiguration;
    /**
     * Returns true if the given object is an instance of BucketMetadataConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BucketMetadataConfiguration;
    /**
     * General purpose bucket that you want to create the metadata configuration for.
     */
    readonly bucket: pulumi.Output<string>;
    readonly expectedBucketOwner: pulumi.Output<string | undefined>;
    /**
     * Metadata configuration. See `metadataConfiguration` Block for details.
     *
     * The following arguments are optional:
     */
    readonly metadataConfiguration: pulumi.Output<outputs.s3.BucketMetadataConfigurationMetadataConfiguration | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.s3.BucketMetadataConfigurationTimeouts | undefined>;
    /**
     * Create a BucketMetadataConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BucketMetadataConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BucketMetadataConfiguration resources.
 */
export interface BucketMetadataConfigurationState {
    /**
     * General purpose bucket that you want to create the metadata configuration for.
     */
    bucket?: pulumi.Input<string>;
    expectedBucketOwner?: pulumi.Input<string>;
    /**
     * Metadata configuration. See `metadataConfiguration` Block for details.
     *
     * The following arguments are optional:
     */
    metadataConfiguration?: pulumi.Input<inputs.s3.BucketMetadataConfigurationMetadataConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.s3.BucketMetadataConfigurationTimeouts>;
}
/**
 * The set of arguments for constructing a BucketMetadataConfiguration resource.
 */
export interface BucketMetadataConfigurationArgs {
    /**
     * General purpose bucket that you want to create the metadata configuration for.
     */
    bucket: pulumi.Input<string>;
    expectedBucketOwner?: pulumi.Input<string>;
    /**
     * Metadata configuration. See `metadataConfiguration` Block for details.
     *
     * The following arguments are optional:
     */
    metadataConfiguration?: pulumi.Input<inputs.s3.BucketMetadataConfigurationMetadataConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.s3.BucketMetadataConfigurationTimeouts>;
}
