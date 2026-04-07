import * as pulumi from "@pulumi/pulumi";
/**
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.Bucket("example", {bucket: "example"});
 * const exampleBucketPublicAccessBlock = new aws.s3.BucketPublicAccessBlock("example", {
 *     bucket: example.id,
 *     blockPublicAcls: true,
 *     blockPublicPolicy: true,
 *     ignorePublicAcls: true,
 *     restrictPublicBuckets: true,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_s3_bucket_public_access_block` using the bucket name. For example:
 *
 * ```sh
 * $ pulumi import aws:s3/bucketPublicAccessBlock:BucketPublicAccessBlock example my-bucket
 * ```
 */
export declare class BucketPublicAccessBlock extends pulumi.CustomResource {
    /**
     * Get an existing BucketPublicAccessBlock resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BucketPublicAccessBlockState, opts?: pulumi.CustomResourceOptions): BucketPublicAccessBlock;
    /**
     * Returns true if the given object is an instance of BucketPublicAccessBlock.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BucketPublicAccessBlock;
    /**
     * Whether Amazon S3 should block public ACLs for this bucket. Defaults to `false`. Enabling this setting does not affect existing policies or ACLs. When set to `true` causes the following behavior:
     * * PUT Bucket ACL and PUT Object ACL calls will fail if the specified ACL allows public access.
     * * PUT Object calls will fail if the request includes an object ACL.
     */
    readonly blockPublicAcls: pulumi.Output<boolean | undefined>;
    /**
     * Whether Amazon S3 should block public bucket policies for this bucket. Defaults to `false`. Enabling this setting does not affect the existing bucket policy. When set to `true` causes Amazon S3 to:
     * * Reject calls to PUT Bucket policy if the specified bucket policy allows public access.
     */
    readonly blockPublicPolicy: pulumi.Output<boolean | undefined>;
    /**
     * S3 Bucket to which this Public Access Block configuration should be applied.
     */
    readonly bucket: pulumi.Output<string>;
    /**
     * Whether Amazon S3 should ignore public ACLs for this bucket. Defaults to `false`. Enabling this setting does not affect the persistence of any existing ACLs and doesn't prevent new public ACLs from being set. When set to `true` causes Amazon S3 to:
     * * Ignore public ACLs on this bucket and any objects that it contains.
     */
    readonly ignorePublicAcls: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Whether Amazon S3 should restrict public bucket policies for this bucket. Defaults to `false`. Enabling this setting does not affect the previously stored bucket policy, except that public and cross-account access within the public bucket policy, including non-public delegation to specific accounts, is blocked. When set to `true`:
     * * Only the bucket owner and AWS Services can access this buckets if it has a public policy.
     */
    readonly restrictPublicBuckets: pulumi.Output<boolean | undefined>;
    /**
     * Whether to retain the public access block upon destruction. If set to `true`, the resource is simply removed from state instead. This may be desirable in certain scenarios to prevent the removal of a public access block before deletion of the associated bucket.
     */
    readonly skipDestroy: pulumi.Output<boolean | undefined>;
    /**
     * Create a BucketPublicAccessBlock resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BucketPublicAccessBlockArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BucketPublicAccessBlock resources.
 */
export interface BucketPublicAccessBlockState {
    /**
     * Whether Amazon S3 should block public ACLs for this bucket. Defaults to `false`. Enabling this setting does not affect existing policies or ACLs. When set to `true` causes the following behavior:
     * * PUT Bucket ACL and PUT Object ACL calls will fail if the specified ACL allows public access.
     * * PUT Object calls will fail if the request includes an object ACL.
     */
    blockPublicAcls?: pulumi.Input<boolean>;
    /**
     * Whether Amazon S3 should block public bucket policies for this bucket. Defaults to `false`. Enabling this setting does not affect the existing bucket policy. When set to `true` causes Amazon S3 to:
     * * Reject calls to PUT Bucket policy if the specified bucket policy allows public access.
     */
    blockPublicPolicy?: pulumi.Input<boolean>;
    /**
     * S3 Bucket to which this Public Access Block configuration should be applied.
     */
    bucket?: pulumi.Input<string>;
    /**
     * Whether Amazon S3 should ignore public ACLs for this bucket. Defaults to `false`. Enabling this setting does not affect the persistence of any existing ACLs and doesn't prevent new public ACLs from being set. When set to `true` causes Amazon S3 to:
     * * Ignore public ACLs on this bucket and any objects that it contains.
     */
    ignorePublicAcls?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether Amazon S3 should restrict public bucket policies for this bucket. Defaults to `false`. Enabling this setting does not affect the previously stored bucket policy, except that public and cross-account access within the public bucket policy, including non-public delegation to specific accounts, is blocked. When set to `true`:
     * * Only the bucket owner and AWS Services can access this buckets if it has a public policy.
     */
    restrictPublicBuckets?: pulumi.Input<boolean>;
    /**
     * Whether to retain the public access block upon destruction. If set to `true`, the resource is simply removed from state instead. This may be desirable in certain scenarios to prevent the removal of a public access block before deletion of the associated bucket.
     */
    skipDestroy?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a BucketPublicAccessBlock resource.
 */
export interface BucketPublicAccessBlockArgs {
    /**
     * Whether Amazon S3 should block public ACLs for this bucket. Defaults to `false`. Enabling this setting does not affect existing policies or ACLs. When set to `true` causes the following behavior:
     * * PUT Bucket ACL and PUT Object ACL calls will fail if the specified ACL allows public access.
     * * PUT Object calls will fail if the request includes an object ACL.
     */
    blockPublicAcls?: pulumi.Input<boolean>;
    /**
     * Whether Amazon S3 should block public bucket policies for this bucket. Defaults to `false`. Enabling this setting does not affect the existing bucket policy. When set to `true` causes Amazon S3 to:
     * * Reject calls to PUT Bucket policy if the specified bucket policy allows public access.
     */
    blockPublicPolicy?: pulumi.Input<boolean>;
    /**
     * S3 Bucket to which this Public Access Block configuration should be applied.
     */
    bucket: pulumi.Input<string>;
    /**
     * Whether Amazon S3 should ignore public ACLs for this bucket. Defaults to `false`. Enabling this setting does not affect the persistence of any existing ACLs and doesn't prevent new public ACLs from being set. When set to `true` causes Amazon S3 to:
     * * Ignore public ACLs on this bucket and any objects that it contains.
     */
    ignorePublicAcls?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether Amazon S3 should restrict public bucket policies for this bucket. Defaults to `false`. Enabling this setting does not affect the previously stored bucket policy, except that public and cross-account access within the public bucket policy, including non-public delegation to specific accounts, is blocked. When set to `true`:
     * * Only the bucket owner and AWS Services can access this buckets if it has a public policy.
     */
    restrictPublicBuckets?: pulumi.Input<boolean>;
    /**
     * Whether to retain the public access block upon destruction. If set to `true`, the resource is simply removed from state instead. This may be desirable in certain scenarios to prevent the removal of a public access block before deletion of the associated bucket.
     */
    skipDestroy?: pulumi.Input<boolean>;
}
