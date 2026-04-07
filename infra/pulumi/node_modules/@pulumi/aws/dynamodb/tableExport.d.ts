import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.Bucket("example", {
 *     bucketPrefix: "example",
 *     forceDestroy: true,
 * });
 * const exampleTable = new aws.dynamodb.Table("example", {
 *     name: "example-table-1",
 *     billingMode: "PAY_PER_REQUEST",
 *     hashKey: "user_id",
 *     attributes: [{
 *         name: "user_id",
 *         type: "S",
 *     }],
 *     pointInTimeRecovery: {
 *         enabled: true,
 *     },
 * });
 * const exampleTableExport = new aws.dynamodb.TableExport("example", {
 *     tableArn: exampleTable.arn,
 *     s3Bucket: example.id,
 * });
 * ```
 *
 * ### Example with export time
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.dynamodb.TableExport("example", {
 *     exportTime: "2023-04-02T11:30:13+01:00",
 *     s3Bucket: exampleAwsS3Bucket.id,
 *     tableArn: exampleAwsDynamodbTable.arn,
 * });
 * ```
 *
 * ### Incremental export
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.dynamodb.TableExport("example", {
 *     exportType: "INCREMENTAL_EXPORT",
 *     s3Bucket: exampleAwsS3Bucket.id,
 *     tableArn: exampleAwsDynamodbTable.arn,
 *     incrementalExportSpecification: {
 *         exportFromTime: "2025-02-09T12:00:00+01:00",
 *         exportToTime: "2025-02-09T13:00:00+01:00",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the DynamoDB table export.
 *
 * Using `pulumi import`, import DynamoDB table exports using the `arn`. For example:
 *
 * console
 *
 * % pulumi import aws_dynamodb_table_export.example arn:aws:dynamodb:us-west-2:12345678911:table/my-table-1/export/01580735656614-2c2f422e
 */
export declare class TableExport extends pulumi.CustomResource {
    /**
     * Get an existing TableExport resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TableExportState, opts?: pulumi.CustomResourceOptions): TableExport;
    /**
     * Returns true if the given object is an instance of TableExport.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TableExport;
    /**
     * ARN of the Table Export.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Billable size of the table export.
     */
    readonly billedSizeInBytes: pulumi.Output<number>;
    /**
     * Time at which the export task completed.
     */
    readonly endTime: pulumi.Output<string>;
    /**
     * Format for the exported data. Valid values are: `DYNAMODB_JSON`, `ION`. See the [AWS Documentation](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/S3DataExport.Output.html#S3DataExport.Output_Data) for more information on these export formats. Default is `DYNAMODB_JSON`.
     */
    readonly exportFormat: pulumi.Output<string | undefined>;
    /**
     * Status of the export - export can be in one of the following states `IN_PROGRESS`, `COMPLETED`, or `FAILED`.
     */
    readonly exportStatus: pulumi.Output<string>;
    /**
     * Time in RFC3339 format from which to export table data. The table export will be a snapshot of the table's state at this point in time. Omitting this value will result in a snapshot from the current time.
     */
    readonly exportTime: pulumi.Output<string>;
    /**
     * Whether to execute as a full export or incremental export. Valid values are: `FULL_EXPORT`, `INCREMENTAL_EXPORT`. Defaults to `FULL_EXPORT`. If `INCREMENTAL_EXPORT` is provided, the `incrementalExportSpecification` argument must also be provided.
     * `incrementalExportSpecification` - (Optional, Forces new resource) Parameters specific to an incremental export. See `incrementalExportSpecification` Block for details.
     */
    readonly exportType: pulumi.Output<string>;
    readonly incrementalExportSpecification: pulumi.Output<outputs.dynamodb.TableExportIncrementalExportSpecification | undefined>;
    /**
     * Number of items exported.
     */
    readonly itemCount: pulumi.Output<number>;
    /**
     * Name of the manifest file for the export task. See the [AWS Documentation](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/S3DataExport.Output.html#S3DataExport.Output_Manifest) for more information on this manifest file.
     */
    readonly manifestFilesS3Key: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the Amazon S3 bucket to export the snapshot to. See the [AWS Documentation](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/S3DataExport_Requesting.html#S3DataExport_Requesting_Permissions) for information on how configure this S3 bucket.
     */
    readonly s3Bucket: pulumi.Output<string>;
    /**
     * ID of the AWS account that owns the bucket the export will be stored in.
     */
    readonly s3BucketOwner: pulumi.Output<string>;
    /**
     * Amazon S3 bucket prefix to use as the file name and path of the exported snapshot.
     */
    readonly s3Prefix: pulumi.Output<string>;
    /**
     * Type of encryption used on the bucket where export data will be stored. Valid values are: `AES256`, `KMS`.
     */
    readonly s3SseAlgorithm: pulumi.Output<string>;
    /**
     * ID of the AWS KMS managed key used to encrypt the S3 bucket where export data will be stored (if applicable).
     */
    readonly s3SseKmsKeyId: pulumi.Output<string | undefined>;
    /**
     * Time at which the export task began.
     */
    readonly startTime: pulumi.Output<string>;
    /**
     * ARN associated with the table to export.
     *
     * The following arguments are optional:
     */
    readonly tableArn: pulumi.Output<string>;
    /**
     * Create a TableExport resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TableExportArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TableExport resources.
 */
export interface TableExportState {
    /**
     * ARN of the Table Export.
     */
    arn?: pulumi.Input<string>;
    /**
     * Billable size of the table export.
     */
    billedSizeInBytes?: pulumi.Input<number>;
    /**
     * Time at which the export task completed.
     */
    endTime?: pulumi.Input<string>;
    /**
     * Format for the exported data. Valid values are: `DYNAMODB_JSON`, `ION`. See the [AWS Documentation](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/S3DataExport.Output.html#S3DataExport.Output_Data) for more information on these export formats. Default is `DYNAMODB_JSON`.
     */
    exportFormat?: pulumi.Input<string>;
    /**
     * Status of the export - export can be in one of the following states `IN_PROGRESS`, `COMPLETED`, or `FAILED`.
     */
    exportStatus?: pulumi.Input<string>;
    /**
     * Time in RFC3339 format from which to export table data. The table export will be a snapshot of the table's state at this point in time. Omitting this value will result in a snapshot from the current time.
     */
    exportTime?: pulumi.Input<string>;
    /**
     * Whether to execute as a full export or incremental export. Valid values are: `FULL_EXPORT`, `INCREMENTAL_EXPORT`. Defaults to `FULL_EXPORT`. If `INCREMENTAL_EXPORT` is provided, the `incrementalExportSpecification` argument must also be provided.
     * `incrementalExportSpecification` - (Optional, Forces new resource) Parameters specific to an incremental export. See `incrementalExportSpecification` Block for details.
     */
    exportType?: pulumi.Input<string>;
    incrementalExportSpecification?: pulumi.Input<inputs.dynamodb.TableExportIncrementalExportSpecification>;
    /**
     * Number of items exported.
     */
    itemCount?: pulumi.Input<number>;
    /**
     * Name of the manifest file for the export task. See the [AWS Documentation](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/S3DataExport.Output.html#S3DataExport.Output_Manifest) for more information on this manifest file.
     */
    manifestFilesS3Key?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the Amazon S3 bucket to export the snapshot to. See the [AWS Documentation](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/S3DataExport_Requesting.html#S3DataExport_Requesting_Permissions) for information on how configure this S3 bucket.
     */
    s3Bucket?: pulumi.Input<string>;
    /**
     * ID of the AWS account that owns the bucket the export will be stored in.
     */
    s3BucketOwner?: pulumi.Input<string>;
    /**
     * Amazon S3 bucket prefix to use as the file name and path of the exported snapshot.
     */
    s3Prefix?: pulumi.Input<string>;
    /**
     * Type of encryption used on the bucket where export data will be stored. Valid values are: `AES256`, `KMS`.
     */
    s3SseAlgorithm?: pulumi.Input<string>;
    /**
     * ID of the AWS KMS managed key used to encrypt the S3 bucket where export data will be stored (if applicable).
     */
    s3SseKmsKeyId?: pulumi.Input<string>;
    /**
     * Time at which the export task began.
     */
    startTime?: pulumi.Input<string>;
    /**
     * ARN associated with the table to export.
     *
     * The following arguments are optional:
     */
    tableArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TableExport resource.
 */
export interface TableExportArgs {
    /**
     * Format for the exported data. Valid values are: `DYNAMODB_JSON`, `ION`. See the [AWS Documentation](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/S3DataExport.Output.html#S3DataExport.Output_Data) for more information on these export formats. Default is `DYNAMODB_JSON`.
     */
    exportFormat?: pulumi.Input<string>;
    /**
     * Time in RFC3339 format from which to export table data. The table export will be a snapshot of the table's state at this point in time. Omitting this value will result in a snapshot from the current time.
     */
    exportTime?: pulumi.Input<string>;
    /**
     * Whether to execute as a full export or incremental export. Valid values are: `FULL_EXPORT`, `INCREMENTAL_EXPORT`. Defaults to `FULL_EXPORT`. If `INCREMENTAL_EXPORT` is provided, the `incrementalExportSpecification` argument must also be provided.
     * `incrementalExportSpecification` - (Optional, Forces new resource) Parameters specific to an incremental export. See `incrementalExportSpecification` Block for details.
     */
    exportType?: pulumi.Input<string>;
    incrementalExportSpecification?: pulumi.Input<inputs.dynamodb.TableExportIncrementalExportSpecification>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the Amazon S3 bucket to export the snapshot to. See the [AWS Documentation](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/S3DataExport_Requesting.html#S3DataExport_Requesting_Permissions) for information on how configure this S3 bucket.
     */
    s3Bucket: pulumi.Input<string>;
    /**
     * ID of the AWS account that owns the bucket the export will be stored in.
     */
    s3BucketOwner?: pulumi.Input<string>;
    /**
     * Amazon S3 bucket prefix to use as the file name and path of the exported snapshot.
     */
    s3Prefix?: pulumi.Input<string>;
    /**
     * Type of encryption used on the bucket where export data will be stored. Valid values are: `AES256`, `KMS`.
     */
    s3SseAlgorithm?: pulumi.Input<string>;
    /**
     * ID of the AWS KMS managed key used to encrypt the S3 bucket where export data will be stored (if applicable).
     */
    s3SseKmsKeyId?: pulumi.Input<string>;
    /**
     * ARN associated with the table to export.
     *
     * The following arguments are optional:
     */
    tableArn: pulumi.Input<string>;
}
