import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an S3 bucket website configuration resource. For more information, see [Hosting Websites on S3](https://docs.aws.amazon.com/AmazonS3/latest/dev/WebsiteHosting.html).
 *
 * > This resource cannot be used with S3 directory buckets.
 *
 * ## Example Usage
 *
 * ### With `routingRule` configured
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.BucketWebsiteConfiguration("example", {
 *     bucket: exampleAwsS3Bucket.id,
 *     indexDocument: {
 *         suffix: "index.html",
 *     },
 *     errorDocument: {
 *         key: "error.html",
 *     },
 *     routingRules: [{
 *         condition: {
 *             keyPrefixEquals: "docs/",
 *         },
 *         redirect: {
 *             replaceKeyPrefixWith: "documents/",
 *         },
 *     }],
 * });
 * ```
 *
 * ### With `routingRules` configured
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.BucketWebsiteConfiguration("example", {
 *     bucket: exampleAwsS3Bucket.id,
 *     indexDocument: {
 *         suffix: "index.html",
 *     },
 *     errorDocument: {
 *         key: "error.html",
 *     },
 *     routingRuleDetails: `[{
 *     \\"Condition\\": {
 *         \\"KeyPrefixEquals\\": \\"docs/\\"
 *     },
 *     \\"Redirect\\": {
 *         \\"ReplaceKeyPrefixWith\\": \\"\\"
 *     }
 * }]
 * `,
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
 *   to = aws_s3_bucket_website_configuration.example
 *
 *   id = "bucket-name,123456789012"
 *
 * }
 *
 * **Using `pulumi import` to import** S3 bucket website configuration using the `bucket` or using the `bucket` and `expected_bucket_owner` separated by a comma (`,`). For example:
 *
 * If the owner (account ID) of the source bucket is the same account used to configure the AWS Provider, import using the `bucket`:
 *
 * console
 *
 * % pulumi import aws_s3_bucket_website_configuration.example bucket-name
 *
 * If the owner (account ID) of the source bucket differs from the account used to configure the AWS Provider, import using the `bucket` and `expected_bucket_owner` separated by a comma (`,`):
 *
 * console
 *
 * % pulumi import aws_s3_bucket_website_configuration.example bucket-name,123456789012
 *
 * @deprecated aws.s3/bucketwebsiteconfigurationv2.BucketWebsiteConfigurationV2 has been deprecated in favor of aws.s3/bucketwebsiteconfiguration.BucketWebsiteConfiguration
 */
export declare class BucketWebsiteConfigurationV2 extends pulumi.CustomResource {
    /**
     * Get an existing BucketWebsiteConfigurationV2 resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BucketWebsiteConfigurationV2State, opts?: pulumi.CustomResourceOptions): BucketWebsiteConfigurationV2;
    /**
     * Returns true if the given object is an instance of BucketWebsiteConfigurationV2.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BucketWebsiteConfigurationV2;
    /**
     * Name of the bucket.
     */
    readonly bucket: pulumi.Output<string>;
    /**
     * Name of the error document for the website. See below.
     */
    readonly errorDocument: pulumi.Output<outputs.s3.BucketWebsiteConfigurationV2ErrorDocument | undefined>;
    /**
     * Account ID of the expected bucket owner.
     */
    readonly expectedBucketOwner: pulumi.Output<string | undefined>;
    /**
     * Name of the index document for the website. See below.
     */
    readonly indexDocument: pulumi.Output<outputs.s3.BucketWebsiteConfigurationV2IndexDocument | undefined>;
    /**
     * Redirect behavior for every request to this bucket's website endpoint. See below. Conflicts with `errorDocument`, `indexDocument`, and `routingRule`.
     */
    readonly redirectAllRequestsTo: pulumi.Output<outputs.s3.BucketWebsiteConfigurationV2RedirectAllRequestsTo | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * JSON array containing [routing rules](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-s3-websiteconfiguration-routingrules.html)
     * describing redirect behavior and when redirects are applied. Use this parameter when your routing rules contain empty String values (`""`) as seen in the example above.
     */
    readonly routingRuleDetails: pulumi.Output<string>;
    /**
     * List of rules that define when a redirect is applied and the redirect behavior. See below.
     */
    readonly routingRules: pulumi.Output<outputs.s3.BucketWebsiteConfigurationV2RoutingRule[]>;
    /**
     * Domain of the website endpoint. This is used to create Route 53 alias records.
     */
    readonly websiteDomain: pulumi.Output<string>;
    /**
     * Website endpoint.
     */
    readonly websiteEndpoint: pulumi.Output<string>;
    /**
     * Create a BucketWebsiteConfigurationV2 resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    /** @deprecated aws.s3/bucketwebsiteconfigurationv2.BucketWebsiteConfigurationV2 has been deprecated in favor of aws.s3/bucketwebsiteconfiguration.BucketWebsiteConfiguration */
    constructor(name: string, args: BucketWebsiteConfigurationV2Args, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BucketWebsiteConfigurationV2 resources.
 */
export interface BucketWebsiteConfigurationV2State {
    /**
     * Name of the bucket.
     */
    bucket?: pulumi.Input<string>;
    /**
     * Name of the error document for the website. See below.
     */
    errorDocument?: pulumi.Input<inputs.s3.BucketWebsiteConfigurationV2ErrorDocument>;
    /**
     * Account ID of the expected bucket owner.
     */
    expectedBucketOwner?: pulumi.Input<string>;
    /**
     * Name of the index document for the website. See below.
     */
    indexDocument?: pulumi.Input<inputs.s3.BucketWebsiteConfigurationV2IndexDocument>;
    /**
     * Redirect behavior for every request to this bucket's website endpoint. See below. Conflicts with `errorDocument`, `indexDocument`, and `routingRule`.
     */
    redirectAllRequestsTo?: pulumi.Input<inputs.s3.BucketWebsiteConfigurationV2RedirectAllRequestsTo>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * JSON array containing [routing rules](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-s3-websiteconfiguration-routingrules.html)
     * describing redirect behavior and when redirects are applied. Use this parameter when your routing rules contain empty String values (`""`) as seen in the example above.
     */
    routingRuleDetails?: pulumi.Input<string>;
    /**
     * List of rules that define when a redirect is applied and the redirect behavior. See below.
     */
    routingRules?: pulumi.Input<pulumi.Input<inputs.s3.BucketWebsiteConfigurationV2RoutingRule>[]>;
    /**
     * Domain of the website endpoint. This is used to create Route 53 alias records.
     */
    websiteDomain?: pulumi.Input<string>;
    /**
     * Website endpoint.
     */
    websiteEndpoint?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a BucketWebsiteConfigurationV2 resource.
 */
export interface BucketWebsiteConfigurationV2Args {
    /**
     * Name of the bucket.
     */
    bucket: pulumi.Input<string>;
    /**
     * Name of the error document for the website. See below.
     */
    errorDocument?: pulumi.Input<inputs.s3.BucketWebsiteConfigurationV2ErrorDocument>;
    /**
     * Account ID of the expected bucket owner.
     */
    expectedBucketOwner?: pulumi.Input<string>;
    /**
     * Name of the index document for the website. See below.
     */
    indexDocument?: pulumi.Input<inputs.s3.BucketWebsiteConfigurationV2IndexDocument>;
    /**
     * Redirect behavior for every request to this bucket's website endpoint. See below. Conflicts with `errorDocument`, `indexDocument`, and `routingRule`.
     */
    redirectAllRequestsTo?: pulumi.Input<inputs.s3.BucketWebsiteConfigurationV2RedirectAllRequestsTo>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * JSON array containing [routing rules](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-s3-websiteconfiguration-routingrules.html)
     * describing redirect behavior and when redirects are applied. Use this parameter when your routing rules contain empty String values (`""`) as seen in the example above.
     */
    routingRuleDetails?: pulumi.Input<string>;
    /**
     * List of rules that define when a redirect is applied and the redirect behavior. See below.
     */
    routingRules?: pulumi.Input<pulumi.Input<inputs.s3.BucketWebsiteConfigurationV2RoutingRule>[]>;
}
