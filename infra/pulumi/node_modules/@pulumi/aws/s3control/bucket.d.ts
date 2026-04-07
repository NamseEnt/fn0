import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage an S3 Control Bucket.
 *
 * > This functionality is for managing [S3 on Outposts](https://docs.aws.amazon.com/AmazonS3/latest/dev/S3onOutposts.html). To manage S3 Buckets in an AWS Partition, see the `aws.s3.Bucket` resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3control.Bucket("example", {
 *     bucket: "example",
 *     outpostId: exampleAwsOutpostsOutpost.id,
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) ARN of the bucket.
 *
 * Using `pulumi import`, import S3 Control Buckets using Amazon Resource Name (ARN). For example:
 *
 * console
 *
 * % pulumi import aws_s3control_bucket.example arn:aws:s3-outposts:us-east-1:123456789012:outpost/op-12345678/bucket/example
 */
export declare class Bucket extends pulumi.CustomResource {
    /**
     * Get an existing Bucket resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BucketState, opts?: pulumi.CustomResourceOptions): Bucket;
    /**
     * Returns true if the given object is an instance of Bucket.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Bucket;
    /**
     * Amazon Resource Name (ARN) of the bucket.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Name of the bucket.
     */
    readonly bucket: pulumi.Output<string>;
    /**
     * UTC creation date in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8).
     */
    readonly creationDate: pulumi.Output<string>;
    /**
     * Identifier of the Outpost to contain this bucket.
     */
    readonly outpostId: pulumi.Output<string>;
    /**
     * Boolean whether Public Access Block is enabled.
     */
    readonly publicAccessBlockEnabled: pulumi.Output<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a Bucket resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BucketArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Bucket resources.
 */
export interface BucketState {
    /**
     * Amazon Resource Name (ARN) of the bucket.
     */
    arn?: pulumi.Input<string>;
    /**
     * Name of the bucket.
     */
    bucket?: pulumi.Input<string>;
    /**
     * UTC creation date in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8).
     */
    creationDate?: pulumi.Input<string>;
    /**
     * Identifier of the Outpost to contain this bucket.
     */
    outpostId?: pulumi.Input<string>;
    /**
     * Boolean whether Public Access Block is enabled.
     */
    publicAccessBlockEnabled?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Bucket resource.
 */
export interface BucketArgs {
    /**
     * Name of the bucket.
     */
    bucket: pulumi.Input<string>;
    /**
     * Identifier of the Outpost to contain this bucket.
     */
    outpostId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
