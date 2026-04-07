import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a S3 bucket [inventory configuration](https://docs.aws.amazon.com/AmazonS3/latest/dev/storage-inventory.html) resource.
 *
 * > This resource cannot be used with S3 directory buckets.
 *
 * ## Example Usage
 *
 * ### Add inventory configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.s3.Bucket("test", {bucket: "my-tf-test-bucket"});
 * const inventory = new aws.s3.Bucket("inventory", {bucket: "my-tf-inventory-bucket"});
 * const testInventory = new aws.s3.Inventory("test", {
 *     bucket: test.id,
 *     name: "EntireBucketDaily",
 *     includedObjectVersions: "All",
 *     schedule: {
 *         frequency: "Daily",
 *     },
 *     destination: {
 *         bucket: {
 *             format: "ORC",
 *             bucketArn: inventory.arn,
 *         },
 *     },
 * });
 * ```
 *
 * ### Add inventory configuration with S3 object prefix
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.s3.Bucket("test", {bucket: "my-tf-test-bucket"});
 * const inventory = new aws.s3.Bucket("inventory", {bucket: "my-tf-inventory-bucket"});
 * const test_prefix = new aws.s3.Inventory("test-prefix", {
 *     bucket: test.id,
 *     name: "DocumentsWeekly",
 *     includedObjectVersions: "All",
 *     schedule: {
 *         frequency: "Daily",
 *     },
 *     filter: {
 *         prefix: "documents/",
 *     },
 *     destination: {
 *         bucket: {
 *             format: "ORC",
 *             bucketArn: inventory.arn,
 *             prefix: "inventory",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import S3 bucket inventory configurations using `bucket:inventory`. For example:
 *
 * ```sh
 * $ pulumi import aws:s3/inventory:Inventory my-bucket-entire-bucket my-bucket:EntireBucket
 * ```
 */
export declare class Inventory extends pulumi.CustomResource {
    /**
     * Get an existing Inventory resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InventoryState, opts?: pulumi.CustomResourceOptions): Inventory;
    /**
     * Returns true if the given object is an instance of Inventory.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Inventory;
    /**
     * Name of the source bucket that inventory lists the objects for.
     */
    readonly bucket: pulumi.Output<string>;
    /**
     * Contains information about where to publish the inventory results (documented below).
     */
    readonly destination: pulumi.Output<outputs.s3.InventoryDestination>;
    /**
     * Specifies whether the inventory is enabled or disabled.
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * Specifies an inventory filter. The inventory only includes objects that meet the filter's criteria (documented below).
     */
    readonly filter: pulumi.Output<outputs.s3.InventoryFilter | undefined>;
    /**
     * Object versions to include in the inventory list. Valid values: `All`, `Current`.
     */
    readonly includedObjectVersions: pulumi.Output<string>;
    /**
     * Unique identifier of the inventory configuration for the bucket.
     */
    readonly name: pulumi.Output<string>;
    /**
     * List of optional fields that are included in the inventory results. Please refer to the S3 [documentation](https://docs.aws.amazon.com/AmazonS3/latest/API/API_InventoryConfiguration.html#AmazonS3-Type-InventoryConfiguration-OptionalFields) for more details.
     */
    readonly optionalFields: pulumi.Output<string[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies the schedule for generating inventory results (documented below).
     */
    readonly schedule: pulumi.Output<outputs.s3.InventorySchedule>;
    /**
     * Create a Inventory resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InventoryArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Inventory resources.
 */
export interface InventoryState {
    /**
     * Name of the source bucket that inventory lists the objects for.
     */
    bucket?: pulumi.Input<string>;
    /**
     * Contains information about where to publish the inventory results (documented below).
     */
    destination?: pulumi.Input<inputs.s3.InventoryDestination>;
    /**
     * Specifies whether the inventory is enabled or disabled.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Specifies an inventory filter. The inventory only includes objects that meet the filter's criteria (documented below).
     */
    filter?: pulumi.Input<inputs.s3.InventoryFilter>;
    /**
     * Object versions to include in the inventory list. Valid values: `All`, `Current`.
     */
    includedObjectVersions?: pulumi.Input<string>;
    /**
     * Unique identifier of the inventory configuration for the bucket.
     */
    name?: pulumi.Input<string>;
    /**
     * List of optional fields that are included in the inventory results. Please refer to the S3 [documentation](https://docs.aws.amazon.com/AmazonS3/latest/API/API_InventoryConfiguration.html#AmazonS3-Type-InventoryConfiguration-OptionalFields) for more details.
     */
    optionalFields?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the schedule for generating inventory results (documented below).
     */
    schedule?: pulumi.Input<inputs.s3.InventorySchedule>;
}
/**
 * The set of arguments for constructing a Inventory resource.
 */
export interface InventoryArgs {
    /**
     * Name of the source bucket that inventory lists the objects for.
     */
    bucket: pulumi.Input<string>;
    /**
     * Contains information about where to publish the inventory results (documented below).
     */
    destination: pulumi.Input<inputs.s3.InventoryDestination>;
    /**
     * Specifies whether the inventory is enabled or disabled.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Specifies an inventory filter. The inventory only includes objects that meet the filter's criteria (documented below).
     */
    filter?: pulumi.Input<inputs.s3.InventoryFilter>;
    /**
     * Object versions to include in the inventory list. Valid values: `All`, `Current`.
     */
    includedObjectVersions: pulumi.Input<string>;
    /**
     * Unique identifier of the inventory configuration for the bucket.
     */
    name?: pulumi.Input<string>;
    /**
     * List of optional fields that are included in the inventory results. Please refer to the S3 [documentation](https://docs.aws.amazon.com/AmazonS3/latest/API/API_InventoryConfiguration.html#AmazonS3-Type-InventoryConfiguration-OptionalFields) for more details.
     */
    optionalFields?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the schedule for generating inventory results (documented below).
     */
    schedule: pulumi.Input<inputs.s3.InventorySchedule>;
}
