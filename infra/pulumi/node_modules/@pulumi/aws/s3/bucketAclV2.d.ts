import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an S3 bucket ACL resource.
 *
 * > **Note:** destroy does not delete the S3 Bucket ACL but does remove the resource from state.
 *
 * > This resource cannot be used with S3 directory buckets.
 *
 * ## Example Usage
 *
 * ### With `private` ACL
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.Bucket("example", {bucket: "my-tf-example-bucket"});
 * const exampleBucketOwnershipControls = new aws.s3.BucketOwnershipControls("example", {
 *     bucket: example.id,
 *     rule: {
 *         objectOwnership: "BucketOwnerPreferred",
 *     },
 * });
 * const exampleBucketAcl = new aws.s3.BucketAcl("example", {
 *     bucket: example.id,
 *     acl: "private",
 * }, {
 *     dependsOn: [exampleBucketOwnershipControls],
 * });
 * ```
 *
 * ### With `public-read` ACL
 *
 * > This example explicitly disables the default S3 bucket security settings. This
 * should be done with caution, as all bucket objects become publicly exposed.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.Bucket("example", {bucket: "my-tf-example-bucket"});
 * const exampleBucketOwnershipControls = new aws.s3.BucketOwnershipControls("example", {
 *     bucket: example.id,
 *     rule: {
 *         objectOwnership: "BucketOwnerPreferred",
 *     },
 * });
 * const exampleBucketPublicAccessBlock = new aws.s3.BucketPublicAccessBlock("example", {
 *     bucket: example.id,
 *     blockPublicAcls: false,
 *     blockPublicPolicy: false,
 *     ignorePublicAcls: false,
 *     restrictPublicBuckets: false,
 * });
 * const exampleBucketAcl = new aws.s3.BucketAcl("example", {
 *     bucket: example.id,
 *     acl: "public-read",
 * }, {
 *     dependsOn: [
 *         exampleBucketOwnershipControls,
 *         exampleBucketPublicAccessBlock,
 *     ],
 * });
 * ```
 *
 * ### With Grants
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.s3.getCanonicalUserId({});
 * const example = new aws.s3.Bucket("example", {bucket: "my-tf-example-bucket"});
 * const exampleBucketOwnershipControls = new aws.s3.BucketOwnershipControls("example", {
 *     bucket: example.id,
 *     rule: {
 *         objectOwnership: "BucketOwnerPreferred",
 *     },
 * });
 * const exampleBucketAcl = new aws.s3.BucketAcl("example", {
 *     bucket: example.id,
 *     accessControlPolicy: {
 *         grants: [
 *             {
 *                 grantee: {
 *                     id: current.then(current => current.id),
 *                     type: "CanonicalUser",
 *                 },
 *                 permission: "READ",
 *             },
 *             {
 *                 grantee: {
 *                     type: "Group",
 *                     uri: "http://acs.amazonaws.com/groups/s3/LogDelivery",
 *                 },
 *                 permission: "READ_ACP",
 *             },
 *         ],
 *         owner: {
 *             id: current.then(current => current.id),
 *         },
 *     },
 * }, {
 *     dependsOn: [exampleBucketOwnershipControls],
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
 * * `acl` (String) Canned ACL to apply to the bucket.
 *
 * * `expected_bucket_owner` (String) Account ID of the expected bucket owner.
 *
 * * `region` (String) Region where this resource is managed.
 *
 * If the owner (account ID) of the source bucket is the _same_ account used to configure the AWS Provider, and the source bucket is **configured** with a
 *
 * [canned ACL][1] (i.e. predefined grant), import using the `bucket` and `acl` separated by a comma (`,`):
 *
 * terraform
 *
 * import {
 *
 *   to = aws_s3_bucket_acl.example
 *
 *   id = "bucket-name,private"
 *
 * }
 *
 * If the owner (account ID) of the source bucket _differs_ from the account used to configure the AWS Provider, and the source bucket is **not configured** with a [canned ACL][1] (i.e. predefined grant), imported using the `bucket` and `expected_bucket_owner` separated by a comma (`,`):
 *
 * terraform
 *
 * import {
 *
 *   to = aws_s3_bucket_acl.example
 *
 *   id = "bucket-name,123456789012"
 *
 * }
 *
 * If the owner (account ID) of the source bucket _differs_ from the account used to configure the AWS Provider, and the source bucket is **configured** with a
 *
 * [canned ACL][1] (i.e. predefined grant), imported using the `bucket`, `expected_bucket_owner`, and `acl` separated by commas (`,`):
 *
 * terraform
 *
 * import {
 *
 *   to = aws_s3_bucket_acl.example
 *
 *   id = "bucket-name,123456789012,private"
 *
 * }
 *
 * **Using `pulumi import` to import** using `bucket`, `expected_bucket_owner`, and/or `acl`, depending on your situation. For example:
 *
 * If the owner (account ID) of the source bucket is the _same_ account used to configure the AWS Provider, and the source bucket is **not configured** with a
 *
 * [canned ACL][1] (i.e. predefined grant), import using the `bucket`:
 *
 * console
 *
 * % pulumi import aws_s3_bucket_acl.example bucket-name
 *
 * If the owner (account ID) of the source bucket is the _same_ account used to configure the AWS Provider, and the source bucket is **configured** with a [canned ACL][1] (i.e. predefined grant), import using the `bucket` and `acl` separated by a comma (`,`):
 *
 * console
 *
 * % pulumi import aws_s3_bucket_acl.example bucket-name,private
 *
 * If the owner (account ID) of the source bucket _differs_ from the account used to configure the AWS Provider, and the source bucket is **not configured** with a [canned ACL][1] (i.e. predefined grant), imported using the `bucket` and `expected_bucket_owner` separated by a comma (`,`):
 *
 * console
 *
 * % pulumi import aws_s3_bucket_acl.example bucket-name,123456789012
 *
 * If the owner (account ID) of the source bucket _differs_ from the account used to configure the AWS Provider, and the source bucket is **configured** with a [canned ACL][1] (i.e. predefined grant), imported using the `bucket`, `expected_bucket_owner`, and `acl` separated by commas (`,`):
 *
 * console
 *
 * % pulumi import aws_s3_bucket_acl.example bucket-name,123456789012,private
 *
 * [1]: https://docs.aws.amazon.com/AmazonS3/latest/userguide/acl-overview.html#canned-acl
 *
 * @deprecated aws.s3/bucketaclv2.BucketAclV2 has been deprecated in favor of aws.s3/bucketacl.BucketAcl
 */
export declare class BucketAclV2 extends pulumi.CustomResource {
    /**
     * Get an existing BucketAclV2 resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BucketAclV2State, opts?: pulumi.CustomResourceOptions): BucketAclV2;
    /**
     * Returns true if the given object is an instance of BucketAclV2.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BucketAclV2;
    /**
     * Configuration block that sets the ACL permissions for an object per grantee. See below.
     */
    readonly accessControlPolicy: pulumi.Output<outputs.s3.BucketAclV2AccessControlPolicy>;
    /**
     * Specifies the Canned ACL to apply to the bucket. Valid values: `private`, `public-read`, `public-read-write`, `aws-exec-read`, `authenticated-read`, `bucket-owner-read`, `bucket-owner-full-control`, `log-delivery-write`. Full details are available on the [AWS documentation](https://docs.aws.amazon.com/AmazonS3/latest/userguide/acl-overview.html#canned-acl).
     */
    readonly acl: pulumi.Output<string | undefined>;
    /**
     * Bucket to which to apply the ACL.
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
     * Create a BucketAclV2 resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    /** @deprecated aws.s3/bucketaclv2.BucketAclV2 has been deprecated in favor of aws.s3/bucketacl.BucketAcl */
    constructor(name: string, args: BucketAclV2Args, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BucketAclV2 resources.
 */
export interface BucketAclV2State {
    /**
     * Configuration block that sets the ACL permissions for an object per grantee. See below.
     */
    accessControlPolicy?: pulumi.Input<inputs.s3.BucketAclV2AccessControlPolicy>;
    /**
     * Specifies the Canned ACL to apply to the bucket. Valid values: `private`, `public-read`, `public-read-write`, `aws-exec-read`, `authenticated-read`, `bucket-owner-read`, `bucket-owner-full-control`, `log-delivery-write`. Full details are available on the [AWS documentation](https://docs.aws.amazon.com/AmazonS3/latest/userguide/acl-overview.html#canned-acl).
     */
    acl?: pulumi.Input<string>;
    /**
     * Bucket to which to apply the ACL.
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
}
/**
 * The set of arguments for constructing a BucketAclV2 resource.
 */
export interface BucketAclV2Args {
    /**
     * Configuration block that sets the ACL permissions for an object per grantee. See below.
     */
    accessControlPolicy?: pulumi.Input<inputs.s3.BucketAclV2AccessControlPolicy>;
    /**
     * Specifies the Canned ACL to apply to the bucket. Valid values: `private`, `public-read`, `public-read-write`, `aws-exec-read`, `authenticated-read`, `bucket-owner-read`, `bucket-owner-full-control`, `log-delivery-write`. Full details are available on the [AWS documentation](https://docs.aws.amazon.com/AmazonS3/latest/userguide/acl-overview.html#canned-acl).
     */
    acl?: pulumi.Input<string>;
    /**
     * Bucket to which to apply the ACL.
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
}
