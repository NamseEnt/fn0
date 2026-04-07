import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS Backup Report Plan resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.backup.ReportPlan("example", {
 *     name: "example_name",
 *     description: "example description",
 *     reportDeliveryChannel: {
 *         formats: [
 *             "CSV",
 *             "JSON",
 *         ],
 *         s3BucketName: "example-bucket-name",
 *     },
 *     reportSetting: {
 *         reportTemplate: "RESTORE_JOB_REPORT",
 *     },
 *     tags: {
 *         Name: "Example Report Plan",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Backup Report Plan using the `id` which corresponds to the name of the Backup Report Plan. For example:
 *
 * ```sh
 * $ pulumi import aws:backup/reportPlan:ReportPlan test <id>
 * ```
 */
export declare class ReportPlan extends pulumi.CustomResource {
    /**
     * Get an existing ReportPlan resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ReportPlanState, opts?: pulumi.CustomResourceOptions): ReportPlan;
    /**
     * Returns true if the given object is an instance of ReportPlan.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ReportPlan;
    /**
     * The ARN of the backup report plan.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The date and time that a report plan is created, in Unix format and Coordinated Universal Time (UTC).
     */
    readonly creationTime: pulumi.Output<string>;
    /**
     * The deployment status of a report plan. The statuses are: `CREATE_IN_PROGRESS` | `UPDATE_IN_PROGRESS` | `DELETE_IN_PROGRESS` | `COMPLETED`.
     */
    readonly deploymentStatus: pulumi.Output<string>;
    /**
     * The description of the report plan with a maximum of 1,024 characters
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The unique name of the report plan. The name must be between 1 and 256 characters, starting with a letter, and consisting of letters, numbers, and underscores.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * An object that contains information about where and how to deliver your reports, specifically your Amazon S3 bucket name, S3 key prefix, and the formats of your reports. Detailed below.
     */
    readonly reportDeliveryChannel: pulumi.Output<outputs.backup.ReportPlanReportDeliveryChannel>;
    /**
     * An object that identifies the report template for the report. Reports are built using a report template. Detailed below.
     */
    readonly reportSetting: pulumi.Output<outputs.backup.ReportPlanReportSetting>;
    /**
     * Metadata that you can assign to help organize the report plans you create. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a ReportPlan resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ReportPlanArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ReportPlan resources.
 */
export interface ReportPlanState {
    /**
     * The ARN of the backup report plan.
     */
    arn?: pulumi.Input<string>;
    /**
     * The date and time that a report plan is created, in Unix format and Coordinated Universal Time (UTC).
     */
    creationTime?: pulumi.Input<string>;
    /**
     * The deployment status of a report plan. The statuses are: `CREATE_IN_PROGRESS` | `UPDATE_IN_PROGRESS` | `DELETE_IN_PROGRESS` | `COMPLETED`.
     */
    deploymentStatus?: pulumi.Input<string>;
    /**
     * The description of the report plan with a maximum of 1,024 characters
     */
    description?: pulumi.Input<string>;
    /**
     * The unique name of the report plan. The name must be between 1 and 256 characters, starting with a letter, and consisting of letters, numbers, and underscores.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * An object that contains information about where and how to deliver your reports, specifically your Amazon S3 bucket name, S3 key prefix, and the formats of your reports. Detailed below.
     */
    reportDeliveryChannel?: pulumi.Input<inputs.backup.ReportPlanReportDeliveryChannel>;
    /**
     * An object that identifies the report template for the report. Reports are built using a report template. Detailed below.
     */
    reportSetting?: pulumi.Input<inputs.backup.ReportPlanReportSetting>;
    /**
     * Metadata that you can assign to help organize the report plans you create. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a ReportPlan resource.
 */
export interface ReportPlanArgs {
    /**
     * The description of the report plan with a maximum of 1,024 characters
     */
    description?: pulumi.Input<string>;
    /**
     * The unique name of the report plan. The name must be between 1 and 256 characters, starting with a letter, and consisting of letters, numbers, and underscores.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * An object that contains information about where and how to deliver your reports, specifically your Amazon S3 bucket name, S3 key prefix, and the formats of your reports. Detailed below.
     */
    reportDeliveryChannel: pulumi.Input<inputs.backup.ReportPlanReportDeliveryChannel>;
    /**
     * An object that identifies the report template for the report. Reports are built using a report template. Detailed below.
     */
    reportSetting: pulumi.Input<inputs.backup.ReportPlanReportSetting>;
    /**
     * Metadata that you can assign to help organize the report plans you create. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
