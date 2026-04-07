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
 *
 * @deprecated aws.s3/bucketserversideencryptionconfigurationv2.BucketServerSideEncryptionConfigurationV2 has been deprecated in favor of aws.s3/bucketserversideencryptionconfiguration.BucketServerSideEncryptionConfiguration
 */
export declare class BucketServerSideEncryptionConfigurationV2 extends pulumi.CustomResource {
    /**
     * Get an existing BucketServerSideEncryptionConfigurationV2 resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BucketServerSideEncryptionConfigurationV2State, opts?: pulumi.CustomResourceOptions): BucketServerSideEncryptionConfigurationV2;
    /**
     * Returns true if the given object is an instance of BucketServerSideEncryptionConfigurationV2.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BucketServerSideEncryptionConfigurationV2;
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
    readonly rules: pulumi.Output<outputs.s3.BucketServerSideEncryptionConfigurationV2Rule[]>;
    /**
     * Create a BucketServerSideEncryptionConfigurationV2 resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    /** @deprecated aws.s3/bucketserversideencryptionconfigurationv2.BucketServerSideEncryptionConfigurationV2 has been deprecated in favor of aws.s3/bucketserversideencryptionconfiguration.BucketServerSideEncryptionConfiguration */
    constructor(name: string, args: BucketServerSideEncryptionConfigurationV2Args, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BucketServerSideEncryptionConfigurationV2 resources.
 */
export interface BucketServerSideEncryptionConfigurationV2State {
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
    rules?: pulumi.Input<pulumi.Input<inputs.s3.BucketServerSideEncryptionConfigurationV2Rule>[]>;
}
/**
 * The set of arguments for constructing a BucketServerSideEncryptionConfigurationV2 resource.
 */
export interface BucketServerSideEncryptionConfigurationV2Args {
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
    rules: pulumi.Input<pulumi.Input<inputs.s3.BucketServerSideEncryptionConfigurationV2Rule>[]>;
}
