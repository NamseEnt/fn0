import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Audit Manager Assessment Report.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.auditmanager.AssessmentReport("test", {
 *     name: "example",
 *     assessmentId: testAwsAuditmanagerAssessment.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Audit Manager Assessment Reports using the assessment report `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:auditmanager/assessmentReport:AssessmentReport example abc123-de45
 * ```
 */
export declare class AssessmentReport extends pulumi.CustomResource {
    /**
     * Get an existing AssessmentReport resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AssessmentReportState, opts?: pulumi.CustomResourceOptions): AssessmentReport;
    /**
     * Returns true if the given object is an instance of AssessmentReport.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AssessmentReport;
    /**
     * Unique identifier of the assessment to create the report from.
     *
     * The following arguments are optional:
     */
    readonly assessmentId: pulumi.Output<string>;
    /**
     * Name of the user who created the assessment report.
     */
    readonly author: pulumi.Output<string>;
    /**
     * Description of the assessment report.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Name of the assessment report.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Current status of the specified assessment report. Valid values are `COMPLETE`, `IN_PROGRESS`, and `FAILED`.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Create a AssessmentReport resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AssessmentReportArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AssessmentReport resources.
 */
export interface AssessmentReportState {
    /**
     * Unique identifier of the assessment to create the report from.
     *
     * The following arguments are optional:
     */
    assessmentId?: pulumi.Input<string>;
    /**
     * Name of the user who created the assessment report.
     */
    author?: pulumi.Input<string>;
    /**
     * Description of the assessment report.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the assessment report.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Current status of the specified assessment report. Valid values are `COMPLETE`, `IN_PROGRESS`, and `FAILED`.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AssessmentReport resource.
 */
export interface AssessmentReportArgs {
    /**
     * Unique identifier of the assessment to create the report from.
     *
     * The following arguments are optional:
     */
    assessmentId: pulumi.Input<string>;
    /**
     * Description of the assessment report.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the assessment report.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
