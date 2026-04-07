import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage an S3 Control Bucket Lifecycle Configuration.
 *
 * > **NOTE:** Each S3 Control Bucket can only have one Lifecycle Configuration. Using multiple of this resource against the same S3 Control Bucket will result in perpetual differences each provider run.
 *
 * > This functionality is for managing [S3 on Outposts](https://docs.aws.amazon.com/AmazonS3/latest/dev/S3onOutposts.html). To manage S3 Bucket Lifecycle Configurations in an AWS Partition, see the `aws.s3.Bucket` resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3control.BucketLifecycleConfiguration("example", {
 *     bucket: exampleAwsS3controlBucket.arn,
 *     rules: [
 *         {
 *             expiration: {
 *                 days: 365,
 *             },
 *             filter: {
 *                 prefix: "logs/",
 *             },
 *             id: "logs",
 *         },
 *         {
 *             expiration: {
 *                 days: 7,
 *             },
 *             filter: {
 *                 prefix: "temp/",
 *             },
 *             id: "temp",
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import S3 Control Bucket Lifecycle Configurations using the Amazon Resource Name (ARN). For example:
 *
 * ```sh
 * $ pulumi import aws:s3control/bucketLifecycleConfiguration:BucketLifecycleConfiguration example arn:aws:s3-outposts:us-east-1:123456789012:outpost/op-12345678/bucket/example
 * ```
 */
export declare class BucketLifecycleConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing BucketLifecycleConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BucketLifecycleConfigurationState, opts?: pulumi.CustomResourceOptions): BucketLifecycleConfiguration;
    /**
     * Returns true if the given object is an instance of BucketLifecycleConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BucketLifecycleConfiguration;
    /**
     * Amazon Resource Name (ARN) of the bucket.
     */
    readonly bucket: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration block(s) containing lifecycle rules for the bucket.
     */
    readonly rules: pulumi.Output<outputs.s3control.BucketLifecycleConfigurationRule[]>;
    /**
     * Create a BucketLifecycleConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BucketLifecycleConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BucketLifecycleConfiguration resources.
 */
export interface BucketLifecycleConfigurationState {
    /**
     * Amazon Resource Name (ARN) of the bucket.
     */
    bucket?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block(s) containing lifecycle rules for the bucket.
     */
    rules?: pulumi.Input<pulumi.Input<inputs.s3control.BucketLifecycleConfigurationRule>[]>;
}
/**
 * The set of arguments for constructing a BucketLifecycleConfiguration resource.
 */
export interface BucketLifecycleConfigurationArgs {
    /**
     * Amazon Resource Name (ARN) of the bucket.
     */
    bucket: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block(s) containing lifecycle rules for the bucket.
     */
    rules: pulumi.Input<pulumi.Input<inputs.s3control.BucketLifecycleConfigurationRule>[]>;
}
