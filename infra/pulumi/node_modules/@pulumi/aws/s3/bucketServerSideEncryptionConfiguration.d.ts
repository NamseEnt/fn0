import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a S3 bucket server-side encryption configuration resource.
 *
 * > **NOTE:** Destroying an `aws.s3.BucketServerSideEncryptionConfiguration` resource resets the bucket to [Amazon S3 bucket default encryption](https://docs.aws.amazon.com/AmazonS3/latest/userguide/default-encryption-faq.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const mykey = new aws.kms.Key("mykey", {
 *     description: "This key is used to encrypt bucket objects",
 *     deletionWindowInDays: 10,
 * });
 * const mybucket = new aws.s3.Bucket("mybucket", {bucket: "mybucket"});
 * const example = new aws.s3.BucketServerSideEncryptionConfiguration("example", {
 *     bucket: mybucket.id,
 *     rules: [{
 *         applyServerSideEncryptionByDefault: {
 *             kmsMasterKeyId: mykey.arn,
 *             sseAlgorithm: "aws:kms",
 *         },
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `bucket` (String) S3 bucket name.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * * `expected_bucket_owner` (String) Account ID of the expected bucket owner.
 *
 * * `region` (String) Region where this resource is managed.
 *
 * If the owner (account ID) of the source bucket differs from the account used to configure the AWS Provider, import using the `bucket` and `expected_bucket_owner` separated by a comma (`,`):
 *
 * terraform
 *
 * import {
 *
 *   to = aws_s3_bucket_server_side_encryption_configuration.example
 *
 *   id = "bucket-name,123456789012"
 *
 * }
 *
 * **Using `pulumi import` to import** S3 bucket server-side encryption configuration using the `bucket` or using the `bucket` and `expected_bucket_owner` separated by a comma (`,`). For example:
 *
 * If the owner (account ID) of the source bucket is the same account used to configure the AWS Provider, import using the `bucket`:
 *
 * console
 *
 * % pulumi import aws_s3_bucket_server_side_encryption_configuration.example bucket-name
 *
 * If the owner (account ID) of the source bucket differs from the account used to configure the AWS Provider, import using the `bucket` and `expected_bucket_owner` separated by a comma (`,`):
 *
 * console
 *
 * % pulumi import aws_s3_bucket_server_side_encryption_configuration.example bucket-name,123456789012
 */
export declare class BucketServerSideEncryptionConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing BucketServerSideEncryptionConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BucketServerSideEncryptionConfigurationState, opts?: pulumi.CustomResourceOptions): BucketServerSideEncryptionConfiguration;
    /**
     * Returns true if the given object is an instance of BucketServerSideEncryptionConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BucketServerSideEncryptionConfiguration;
    /**
     * ID (name) of the bucket.
     */
    readonly bucket: pulumi.Output<string>;
    /**
     * Account ID of the expected bucket owner.
     */
    readonly expectedBucketOwner: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Set of server-side encryption configuration rules. See below. Currently, only a single rule is supported.
     */
    readonly rules: pulumi.Output<outputs.s3.BucketServerSideEncryptionConfigurationRule[]>;
    /**
     * Create a BucketServerSideEncryptionConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BucketServerSideEncryptionConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BucketServerSideEncryptionConfiguration resources.
 */
export interface BucketServerSideEncryptionConfigurationState {
    /**
     * ID (name) of the bucket.
     */
    bucket?: pulumi.Input<string>;
    /**
     * Account ID of the expected bucket owner.
     */
    expectedBucketOwner?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Set of server-side encryption configuration rules. See below. Currently, only a single rule is supported.
     */
    rules?: pulumi.Input<pulumi.Input<inputs.s3.BucketServerSideEncryptionConfigurationRule>[]>;
}
/**
 * The set of arguments for constructing a BucketServerSideEncryptionConfiguration resource.
 */
export interface BucketServerSideEncryptionConfigurationArgs {
    /**
     * ID (name) of the bucket.
     */
    bucket: pulumi.Input<string>;
    /**
     * Account ID of the expected bucket owner.
     */
    expectedBucketOwner?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Set of server-side encryption configuration rules. See below. Currently, only a single rule is supported.
     */
    rules: pulumi.Input<pulumi.Input<inputs.s3.BucketServerSideEncryptionConfigurationRule>[]>;
}
