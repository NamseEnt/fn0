import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS BCM Data Exports Export.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const currentGetPartition = aws.getPartition({});
 * const test = new aws.bcmdata.Export("test", {"export": {
 *     name: "testexample",
 *     dataQueries: [{
 *         queryStatement: "SELECT identity_line_item_id, identity_time_interval, line_item_product_code,line_item_unblended_cost FROM COST_AND_USAGE_REPORT",
 *         tableConfigurations: {
 *             COST_AND_USAGE_REPORT: {
 *                 BILLING_VIEW_ARN: Promise.all([currentGetPartition, current]).then(([currentGetPartition, current]) => `arn:${currentGetPartition.partition}:billing::${current.accountId}:billingview/primary`),
 *                 TIME_GRANULARITY: "HOURLY",
 *                 INCLUDE_RESOURCES: "FALSE",
 *                 INCLUDE_MANUAL_DISCOUNT_COMPATIBILITY: "FALSE",
 *                 INCLUDE_SPLIT_COST_ALLOCATION_DATA: "FALSE",
 *             },
 *         },
 *     }],
 *     destinationConfigurations: [{
 *         s3Destinations: [{
 *             s3Bucket: testAwsS3Bucket.bucket,
 *             s3Prefix: testAwsS3Bucket.bucketPrefix,
 *             s3Region: testAwsS3Bucket.region,
 *             s3OutputConfigurations: [{
 *                 overwrite: "OVERWRITE_REPORT",
 *                 format: "TEXT_OR_CSV",
 *                 compression: "GZIP",
 *                 outputType: "CUSTOM",
 *             }],
 *         }],
 *     }],
 *     refreshCadences: [{
 *         frequency: "SYNCHRONOUS",
 *     }],
 * }});
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the BCM Data Exports export.
 *
 * Using `pulumi import`, import BCM Data Exports Export using the export ARN. For example:
 *
 * console
 *
 * % pulumi import aws_bcmdataexports_export.example arn:aws:bcm-data-exports:us-east-1:123456789012:export/CostUsageReport-9f1c75f3-f982-4d9a-b936-1e7ecab814b7
 */
export declare class Export extends pulumi.CustomResource {
    /**
     * Get an existing Export resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ExportState, opts?: pulumi.CustomResourceOptions): Export;
    /**
     * Returns true if the given object is an instance of Export.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Export;
    /**
     * Amazon Resource Name (ARN) for this export.
     * * `export[0].export_arn` - Amazon Resource Name (ARN) for this export.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The details of the export, including data query, name, description, and destination configuration.  See the `export` argument reference below.
     */
    readonly export: pulumi.Output<outputs.bcmdata.ExportExport | undefined>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.bcmdata.ExportTimeouts | undefined>;
    /**
     * Create a Export resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ExportArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Export resources.
 */
export interface ExportState {
    /**
     * Amazon Resource Name (ARN) for this export.
     * * `export[0].export_arn` - Amazon Resource Name (ARN) for this export.
     */
    arn?: pulumi.Input<string>;
    /**
     * The details of the export, including data query, name, description, and destination configuration.  See the `export` argument reference below.
     */
    export?: pulumi.Input<inputs.bcmdata.ExportExport>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bcmdata.ExportTimeouts>;
}
/**
 * The set of arguments for constructing a Export resource.
 */
export interface ExportArgs {
    /**
     * The details of the export, including data query, name, description, and destination configuration.  See the `export` argument reference below.
     */
    export?: pulumi.Input<inputs.bcmdata.ExportExport>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bcmdata.ExportTimeouts>;
}
