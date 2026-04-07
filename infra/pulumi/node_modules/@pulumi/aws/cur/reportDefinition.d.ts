import * as pulumi from "@pulumi/pulumi";
/**
 * Manages Cost and Usage Report Definitions.
 *
 * > *NOTE:* The AWS Cost and Usage Report service is only available in `us-east-1` currently.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleCurReportDefinition = new aws.cur.ReportDefinition("example_cur_report_definition", {
 *     reportName: "example-cur-report-definition",
 *     timeUnit: "HOURLY",
 *     format: "textORcsv",
 *     compression: "GZIP",
 *     additionalSchemaElements: [
 *         "RESOURCES",
 *         "SPLIT_COST_ALLOCATION_DATA",
 *     ],
 *     s3Bucket: "example-bucket-name",
 *     s3Prefix: "example-cur-report",
 *     s3Region: "us-east-1",
 *     additionalArtifacts: [
 *         "REDSHIFT",
 *         "QUICKSIGHT",
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Report Definitions using the `report_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:cur/reportDefinition:ReportDefinition example_cur_report_definition example-cur-report-definition
 * ```
 */
export declare class ReportDefinition extends pulumi.CustomResource {
    /**
     * Get an existing ReportDefinition resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ReportDefinitionState, opts?: pulumi.CustomResourceOptions): ReportDefinition;
    /**
     * Returns true if the given object is an instance of ReportDefinition.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ReportDefinition;
    /**
     * A list of additional artifacts. Valid values are: `REDSHIFT`, `QUICKSIGHT`, `ATHENA`. When ATHENA exists within additional_artifacts, no other artifact type can be declared and reportVersioning must be `OVERWRITE_REPORT`.
     */
    readonly additionalArtifacts: pulumi.Output<string[] | undefined>;
    /**
     * A list of schema elements. Valid values are: `RESOURCES`, `SPLIT_COST_ALLOCATION_DATA`, `MANUAL_DISCOUNT_COMPATIBILITY`.
     */
    readonly additionalSchemaElements: pulumi.Output<string[]>;
    /**
     * The Amazon Resource Name (ARN) specifying the cur report.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Compression format for report. Valid values are: `GZIP`, `ZIP`, `Parquet`. If `Parquet` is used, then format must also be `Parquet`.
     */
    readonly compression: pulumi.Output<string>;
    /**
     * Format for report. Valid values are: `textORcsv`, `Parquet`. If `Parquet` is used, then Compression must also be `Parquet`.
     */
    readonly format: pulumi.Output<string>;
    /**
     * Set to true to update your reports after they have been finalized if AWS detects charges related to previous months.
     */
    readonly refreshClosedReports: pulumi.Output<boolean | undefined>;
    /**
     * Unique name for the report. Must start with a number/letter and is case sensitive. Limited to 256 characters.
     */
    readonly reportName: pulumi.Output<string>;
    /**
     * Overwrite the previous version of each report or to deliver the report in addition to the previous versions. Valid values are: `CREATE_NEW_REPORT` and `OVERWRITE_REPORT`.
     */
    readonly reportVersioning: pulumi.Output<string | undefined>;
    /**
     * Name of the existing S3 bucket to hold generated reports.
     */
    readonly s3Bucket: pulumi.Output<string>;
    /**
     * Report path prefix. Limited to 256 characters. May be empty (`""`) but the resource can then not be modified via the AWS Console.
     */
    readonly s3Prefix: pulumi.Output<string>;
    /**
     * Region of the existing S3 bucket to hold generated reports.
     */
    readonly s3Region: pulumi.Output<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The frequency on which report data are measured and displayed.  Valid values are: `DAILY`, `HOURLY`, `MONTHLY`.
     */
    readonly timeUnit: pulumi.Output<string>;
    /**
     * Create a ReportDefinition resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ReportDefinitionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ReportDefinition resources.
 */
export interface ReportDefinitionState {
    /**
     * A list of additional artifacts. Valid values are: `REDSHIFT`, `QUICKSIGHT`, `ATHENA`. When ATHENA exists within additional_artifacts, no other artifact type can be declared and reportVersioning must be `OVERWRITE_REPORT`.
     */
    additionalArtifacts?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A list of schema elements. Valid values are: `RESOURCES`, `SPLIT_COST_ALLOCATION_DATA`, `MANUAL_DISCOUNT_COMPATIBILITY`.
     */
    additionalSchemaElements?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The Amazon Resource Name (ARN) specifying the cur report.
     */
    arn?: pulumi.Input<string>;
    /**
     * Compression format for report. Valid values are: `GZIP`, `ZIP`, `Parquet`. If `Parquet` is used, then format must also be `Parquet`.
     */
    compression?: pulumi.Input<string>;
    /**
     * Format for report. Valid values are: `textORcsv`, `Parquet`. If `Parquet` is used, then Compression must also be `Parquet`.
     */
    format?: pulumi.Input<string>;
    /**
     * Set to true to update your reports after they have been finalized if AWS detects charges related to previous months.
     */
    refreshClosedReports?: pulumi.Input<boolean>;
    /**
     * Unique name for the report. Must start with a number/letter and is case sensitive. Limited to 256 characters.
     */
    reportName?: pulumi.Input<string>;
    /**
     * Overwrite the previous version of each report or to deliver the report in addition to the previous versions. Valid values are: `CREATE_NEW_REPORT` and `OVERWRITE_REPORT`.
     */
    reportVersioning?: pulumi.Input<string>;
    /**
     * Name of the existing S3 bucket to hold generated reports.
     */
    s3Bucket?: pulumi.Input<string>;
    /**
     * Report path prefix. Limited to 256 characters. May be empty (`""`) but the resource can then not be modified via the AWS Console.
     */
    s3Prefix?: pulumi.Input<string>;
    /**
     * Region of the existing S3 bucket to hold generated reports.
     */
    s3Region?: pulumi.Input<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * The frequency on which report data are measured and displayed.  Valid values are: `DAILY`, `HOURLY`, `MONTHLY`.
     */
    timeUnit?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ReportDefinition resource.
 */
export interface ReportDefinitionArgs {
    /**
     * A list of additional artifacts. Valid values are: `REDSHIFT`, `QUICKSIGHT`, `ATHENA`. When ATHENA exists within additional_artifacts, no other artifact type can be declared and reportVersioning must be `OVERWRITE_REPORT`.
     */
    additionalArtifacts?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A list of schema elements. Valid values are: `RESOURCES`, `SPLIT_COST_ALLOCATION_DATA`, `MANUAL_DISCOUNT_COMPATIBILITY`.
     */
    additionalSchemaElements: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Compression format for report. Valid values are: `GZIP`, `ZIP`, `Parquet`. If `Parquet` is used, then format must also be `Parquet`.
     */
    compression: pulumi.Input<string>;
    /**
     * Format for report. Valid values are: `textORcsv`, `Parquet`. If `Parquet` is used, then Compression must also be `Parquet`.
     */
    format: pulumi.Input<string>;
    /**
     * Set to true to update your reports after they have been finalized if AWS detects charges related to previous months.
     */
    refreshClosedReports?: pulumi.Input<boolean>;
    /**
     * Unique name for the report. Must start with a number/letter and is case sensitive. Limited to 256 characters.
     */
    reportName: pulumi.Input<string>;
    /**
     * Overwrite the previous version of each report or to deliver the report in addition to the previous versions. Valid values are: `CREATE_NEW_REPORT` and `OVERWRITE_REPORT`.
     */
    reportVersioning?: pulumi.Input<string>;
    /**
     * Name of the existing S3 bucket to hold generated reports.
     */
    s3Bucket: pulumi.Input<string>;
    /**
     * Report path prefix. Limited to 256 characters. May be empty (`""`) but the resource can then not be modified via the AWS Console.
     */
    s3Prefix: pulumi.Input<string>;
    /**
     * Region of the existing S3 bucket to hold generated reports.
     */
    s3Region: pulumi.Input<string>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Location. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The frequency on which report data are measured and displayed.  Valid values are: `DAILY`, `HOURLY`, `MONTHLY`.
     */
    timeUnit: pulumi.Input<string>;
}
