import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage an S3 Storage Lens configuration.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const example = new aws.s3control.StorageLensConfiguration("example", {
 *     configId: "example-1",
 *     storageLensConfiguration: {
 *         enabled: true,
 *         accountLevel: {
 *             activityMetrics: {
 *                 enabled: true,
 *             },
 *             bucketLevel: {
 *                 activityMetrics: {
 *                     enabled: true,
 *                 },
 *             },
 *         },
 *         dataExport: {
 *             cloudWatchMetrics: {
 *                 enabled: true,
 *             },
 *             s3BucketDestination: {
 *                 accountId: current.then(current => current.accountId),
 *                 arn: target.arn,
 *                 format: "CSV",
 *                 outputSchemaVersion: "V_1",
 *                 encryption: {
 *                     sseS3s: [{}],
 *                 },
 *             },
 *         },
 *         exclude: {
 *             buckets: [
 *                 b1.arn,
 *                 b2.arn,
 *             ],
 *             regions: ["us-east-2"],
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import S3 Storage Lens configurations using the `account_id` and `config_id`, separated by a colon (`:`). For example:
 *
 * ```sh
 * $ pulumi import aws:s3control/storageLensConfiguration:StorageLensConfiguration example 123456789012:example-1
 * ```
 */
export declare class StorageLensConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing StorageLensConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: StorageLensConfigurationState, opts?: pulumi.CustomResourceOptions): StorageLensConfiguration;
    /**
     * Returns true if the given object is an instance of StorageLensConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is StorageLensConfiguration;
    /**
     * The AWS account ID for the S3 Storage Lens configuration. Defaults to automatically determined account ID of the AWS provider.
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the S3 Storage Lens configuration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The ID of the S3 Storage Lens configuration.
     */
    readonly configId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The S3 Storage Lens configuration. See Storage Lens Configuration below for more details.
     */
    readonly storageLensConfiguration: pulumi.Output<outputs.s3control.StorageLensConfigurationStorageLensConfiguration>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a StorageLensConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: StorageLensConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering StorageLensConfiguration resources.
 */
export interface StorageLensConfigurationState {
    /**
     * The AWS account ID for the S3 Storage Lens configuration. Defaults to automatically determined account ID of the AWS provider.
     */
    accountId?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the S3 Storage Lens configuration.
     */
    arn?: pulumi.Input<string>;
    /**
     * The ID of the S3 Storage Lens configuration.
     */
    configId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The S3 Storage Lens configuration. See Storage Lens Configuration below for more details.
     */
    storageLensConfiguration?: pulumi.Input<inputs.s3control.StorageLensConfigurationStorageLensConfiguration>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a StorageLensConfiguration resource.
 */
export interface StorageLensConfigurationArgs {
    /**
     * The AWS account ID for the S3 Storage Lens configuration. Defaults to automatically determined account ID of the AWS provider.
     */
    accountId?: pulumi.Input<string>;
    /**
     * The ID of the S3 Storage Lens configuration.
     */
    configId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The S3 Storage Lens configuration. See Storage Lens Configuration below for more details.
     */
    storageLensConfiguration: pulumi.Input<inputs.s3control.StorageLensConfigurationStorageLensConfiguration>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
