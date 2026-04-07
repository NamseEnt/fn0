import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a S3 bucket [analytics configuration](https://docs.aws.amazon.com/AmazonS3/latest/dev/analytics-storage-class.html) resource.
 *
 * > This resource cannot be used with S3 directory buckets.
 *
 * ## Example Usage
 *
 * ### Add analytics configuration for entire S3 bucket and export results to a second S3 bucket
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.Bucket("example", {bucket: "example"});
 * const analytics = new aws.s3.Bucket("analytics", {bucket: "analytics-destination"});
 * const example_entire_bucket = new aws.s3.AnalyticsConfiguration("example-entire-bucket", {
 *     bucket: example.id,
 *     name: "EntireBucket",
 *     storageClassAnalysis: {
 *         dataExport: {
 *             destination: {
 *                 s3BucketDestination: {
 *                     bucketArn: analytics.arn,
 *                 },
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ### Add analytics configuration with S3 object filter
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.Bucket("example", {bucket: "example"});
 * const example_filtered = new aws.s3.AnalyticsConfiguration("example-filtered", {
 *     bucket: example.id,
 *     name: "ImportantBlueDocuments",
 *     filter: {
 *         prefix: "documents/",
 *         tags: {
 *             priority: "high",
 *             "class": "blue",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import S3 bucket analytics configurations using `bucket:analytics`. For example:
 *
 * ```sh
 * $ pulumi import aws:s3/analyticsConfiguration:AnalyticsConfiguration my-bucket-entire-bucket my-bucket:EntireBucket
 * ```
 */
export declare class AnalyticsConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing AnalyticsConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AnalyticsConfigurationState, opts?: pulumi.CustomResourceOptions): AnalyticsConfiguration;
    /**
     * Returns true if the given object is an instance of AnalyticsConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AnalyticsConfiguration;
    /**
     * Name of the bucket this analytics configuration is associated with.
     */
    readonly bucket: pulumi.Output<string>;
    /**
     * Object filtering that accepts a prefix, tags, or a logical AND of prefix and tags (documented below).
     */
    readonly filter: pulumi.Output<outputs.s3.AnalyticsConfigurationFilter | undefined>;
    /**
     * Unique identifier of the analytics configuration for the bucket.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration for the analytics data export (documented below).
     */
    readonly storageClassAnalysis: pulumi.Output<outputs.s3.AnalyticsConfigurationStorageClassAnalysis | undefined>;
    /**
     * Create a AnalyticsConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AnalyticsConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AnalyticsConfiguration resources.
 */
export interface AnalyticsConfigurationState {
    /**
     * Name of the bucket this analytics configuration is associated with.
     */
    bucket?: pulumi.Input<string>;
    /**
     * Object filtering that accepts a prefix, tags, or a logical AND of prefix and tags (documented below).
     */
    filter?: pulumi.Input<inputs.s3.AnalyticsConfigurationFilter>;
    /**
     * Unique identifier of the analytics configuration for the bucket.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration for the analytics data export (documented below).
     */
    storageClassAnalysis?: pulumi.Input<inputs.s3.AnalyticsConfigurationStorageClassAnalysis>;
}
/**
 * The set of arguments for constructing a AnalyticsConfiguration resource.
 */
export interface AnalyticsConfigurationArgs {
    /**
     * Name of the bucket this analytics configuration is associated with.
     */
    bucket: pulumi.Input<string>;
    /**
     * Object filtering that accepts a prefix, tags, or a logical AND of prefix and tags (documented below).
     */
    filter?: pulumi.Input<inputs.s3.AnalyticsConfigurationFilter>;
    /**
     * Unique identifier of the analytics configuration for the bucket.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration for the analytics data export (documented below).
     */
    storageClassAnalysis?: pulumi.Input<inputs.s3.AnalyticsConfigurationStorageClassAnalysis>;
}
