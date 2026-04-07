import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
/**
 * Attaches a policy to an S3 bucket resource.
 *
 * > Policies can be attached to both S3 general purpose buckets and S3 directory buckets.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.Bucket("example", {bucket: "my-tf-test-bucket"});
 * const allowAccessFromAnotherAccount = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         principals: [{
 *             type: "AWS",
 *             identifiers: ["123456789012"],
 *         }],
 *         actions: [
 *             "s3:GetObject",
 *             "s3:ListBucket",
 *         ],
 *         resources: [
 *             example.arn,
 *             pulumi.interpolate`${example.arn}/*`,
 *         ],
 *     }],
 * });
 * const allowAccessFromAnotherAccountBucketPolicy = new aws.s3.BucketPolicy("allow_access_from_another_account", {
 *     bucket: example.id,
 *     policy: allowAccessFromAnotherAccount.apply(allowAccessFromAnotherAccount => allowAccessFromAnotherAccount.json),
 * });
 * ```
 *
 * > Only one `aws.s3.BucketPolicy` resource should be defined per S3 bucket. Defining multiple `aws.s3.BucketPolicy` resources with different Pulumi names but the same `bucket` value may result in unexpected policy overwrites. Each resource uses the `PutBucketPolicy` API, which replaces the entire existing policy without error or warning. Because Pulumi treats each resource independently, the policy applied last will silently override any previously applied policy.
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `bucket` (String) Name of the S3 bucket.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * * `region` (String) Region where this resource is managed.
 *
 * Using `pulumi import`, import S3 bucket policies using the bucket name. For example:
 *
 * console
 *
 * % pulumi import aws_s3_bucket_policy.example my-tf-test-bucket
 */
export declare class BucketPolicy extends pulumi.CustomResource {
    /**
     * Get an existing BucketPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BucketPolicyState, opts?: pulumi.CustomResourceOptions): BucketPolicy;
    /**
     * Returns true if the given object is an instance of BucketPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BucketPolicy;
    /**
     * Name of the bucket to which to apply the policy.
     */
    readonly bucket: pulumi.Output<string>;
    /**
     * Text of the policy. Although this is a bucket policy rather than an IAM policy, the `aws.iam.getPolicyDocument` data source may be used, so long as it specifies a principal. For more information about building AWS IAM policy documents, see the AWS IAM Policy Document Guide. Note: Bucket policies are limited to 20 KB in size.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a BucketPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BucketPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BucketPolicy resources.
 */
export interface BucketPolicyState {
    /**
     * Name of the bucket to which to apply the policy.
     */
    bucket?: pulumi.Input<string>;
    /**
     * Text of the policy. Although this is a bucket policy rather than an IAM policy, the `aws.iam.getPolicyDocument` data source may be used, so long as it specifies a principal. For more information about building AWS IAM policy documents, see the AWS IAM Policy Document Guide. Note: Bucket policies are limited to 20 KB in size.
     */
    policy?: pulumi.Input<string | inputs.s3.PolicyDocument>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a BucketPolicy resource.
 */
export interface BucketPolicyArgs {
    /**
     * Name of the bucket to which to apply the policy.
     */
    bucket: pulumi.Input<string>;
    /**
     * Text of the policy. Although this is a bucket policy rather than an IAM policy, the `aws.iam.getPolicyDocument` data source may be used, so long as it specifies a principal. For more information about building AWS IAM policy documents, see the AWS IAM Policy Document Guide. Note: Bucket policies are limited to 20 KB in size.
     */
    policy: pulumi.Input<string | inputs.s3.PolicyDocument>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
