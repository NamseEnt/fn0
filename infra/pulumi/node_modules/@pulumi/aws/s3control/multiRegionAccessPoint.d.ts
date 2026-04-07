import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage an S3 Multi-Region Access Point associated with specified buckets.
 *
 * > This resource cannot be used with S3 directory buckets.
 *
 * ## Example Usage
 *
 * ### Multiple AWS Buckets in Different Regions
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const fooBucket = new aws.s3.Bucket("foo_bucket", {bucket: "example-bucket-foo"});
 * const barBucket = new aws.s3.Bucket("bar_bucket", {bucket: "example-bucket-bar"});
 * const example = new aws.s3control.MultiRegionAccessPoint("example", {details: {
 *     name: "example",
 *     regions: [
 *         {
 *             bucket: fooBucket.id,
 *         },
 *         {
 *             bucket: barBucket.id,
 *         },
 *     ],
 * }});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Multi-Region Access Points using the `account_id` and `name` of the Multi-Region Access Point separated by a colon (`:`). For example:
 *
 * ```sh
 * $ pulumi import aws:s3control/multiRegionAccessPoint:MultiRegionAccessPoint example 123456789012:example
 * ```
 */
export declare class MultiRegionAccessPoint extends pulumi.CustomResource {
    /**
     * Get an existing MultiRegionAccessPoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MultiRegionAccessPointState, opts?: pulumi.CustomResourceOptions): MultiRegionAccessPoint;
    /**
     * Returns true if the given object is an instance of MultiRegionAccessPoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is MultiRegionAccessPoint;
    /**
     * The AWS account ID for the owner of the buckets for which you want to create a Multi-Region Access Point. Defaults to automatically determined account ID of the AWS provider.
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * The alias for the Multi-Region Access Point.
     */
    readonly alias: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the Multi-Region Access Point.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A configuration block containing details about the Multi-Region Access Point. See Details Configuration Block below for more details
     */
    readonly details: pulumi.Output<outputs.s3control.MultiRegionAccessPointDetails>;
    /**
     * The DNS domain name of the S3 Multi-Region Access Point in the format _`alias`_.accesspoint.s3-global.amazonaws.com. For more information, see the documentation on [Multi-Region Access Point Requests](https://docs.aws.amazon.com/AmazonS3/latest/userguide/MultiRegionAccessPointRequests.html).
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The current status of the Multi-Region Access Point. One of: `READY`, `INCONSISTENT_ACROSS_REGIONS`, `CREATING`, `PARTIALLY_CREATED`, `PARTIALLY_DELETED`, `DELETING`.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Create a MultiRegionAccessPoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MultiRegionAccessPointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering MultiRegionAccessPoint resources.
 */
export interface MultiRegionAccessPointState {
    /**
     * The AWS account ID for the owner of the buckets for which you want to create a Multi-Region Access Point. Defaults to automatically determined account ID of the AWS provider.
     */
    accountId?: pulumi.Input<string>;
    /**
     * The alias for the Multi-Region Access Point.
     */
    alias?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the Multi-Region Access Point.
     */
    arn?: pulumi.Input<string>;
    /**
     * A configuration block containing details about the Multi-Region Access Point. See Details Configuration Block below for more details
     */
    details?: pulumi.Input<inputs.s3control.MultiRegionAccessPointDetails>;
    /**
     * The DNS domain name of the S3 Multi-Region Access Point in the format _`alias`_.accesspoint.s3-global.amazonaws.com. For more information, see the documentation on [Multi-Region Access Point Requests](https://docs.aws.amazon.com/AmazonS3/latest/userguide/MultiRegionAccessPointRequests.html).
     */
    domainName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The current status of the Multi-Region Access Point. One of: `READY`, `INCONSISTENT_ACROSS_REGIONS`, `CREATING`, `PARTIALLY_CREATED`, `PARTIALLY_DELETED`, `DELETING`.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a MultiRegionAccessPoint resource.
 */
export interface MultiRegionAccessPointArgs {
    /**
     * The AWS account ID for the owner of the buckets for which you want to create a Multi-Region Access Point. Defaults to automatically determined account ID of the AWS provider.
     */
    accountId?: pulumi.Input<string>;
    /**
     * A configuration block containing details about the Multi-Region Access Point. See Details Configuration Block below for more details
     */
    details: pulumi.Input<inputs.s3control.MultiRegionAccessPointDetails>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
