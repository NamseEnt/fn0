import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an FIS Experiment Template, which can be used to run an experiment.
 * An experiment template contains one or more actions to run on specified targets during an experiment.
 * It also contains the stop conditions that prevent the experiment from going out of bounds.
 * See [Amazon Fault Injection Simulator](https://docs.aws.amazon.com/fis/index.html)
 * for more information.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.fis.ExperimentTemplate("example", {
 *     description: "example",
 *     roleArn: exampleAwsIamRole.arn,
 *     stopConditions: [{
 *         source: "none",
 *     }],
 *     actions: [{
 *         name: "example-action",
 *         actionId: "aws:ec2:terminate-instances",
 *         target: {
 *             key: "Instances",
 *             value: "example-target",
 *         },
 *     }],
 *     targets: [{
 *         name: "example-target",
 *         resourceType: "aws:ec2:instance",
 *         selectionMode: "COUNT(1)",
 *         resourceTags: [{
 *             key: "env",
 *             value: "example",
 *         }],
 *     }],
 * });
 * ```
 *
 * ### With Report Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getPartition({});
 * const example = new aws.iam.Role("example", {
 *     name: "example",
 *     assumeRolePolicy: JSON.stringify({
 *         Statement: [{
 *             Action: "sts:AssumeRole",
 *             Effect: "Allow",
 *             Principal: {
 *                 Service: [current.then(current => `fis.${current.dnsSuffix}`)],
 *             },
 *         }],
 *         Version: "2012-10-17",
 *     }),
 * });
 * const reportAccess = aws.iam.getPolicyDocument({
 *     version: "2012-10-17",
 *     statements: [
 *         {
 *             sid: "logsDelivery",
 *             effect: "Allow",
 *             actions: ["logs:CreateLogDelivery"],
 *             resources: ["*"],
 *         },
 *         {
 *             sid: "ReportsBucket",
 *             effect: "Allow",
 *             actions: [
 *                 "s3:PutObject",
 *                 "s3:GetObject",
 *             ],
 *             resources: ["*"],
 *         },
 *         {
 *             sid: "GetDashboard",
 *             effect: "Allow",
 *             actions: ["cloudwatch:GetDashboard"],
 *             resources: ["*"],
 *         },
 *         {
 *             sid: "GetDashboardData",
 *             effect: "Allow",
 *             actions: ["cloudwatch:getMetricWidgetImage"],
 *             resources: ["*"],
 *         },
 *     ],
 * });
 * const reportAccessPolicy = new aws.iam.Policy("report_access", {
 *     name: "report_access",
 *     policy: reportAccess.then(reportAccess => reportAccess.json),
 * });
 * const reportAccessRolePolicyAttachment = new aws.iam.RolePolicyAttachment("report_access", {
 *     role: test.name,
 *     policyArn: reportAccessPolicy.arn,
 * });
 * const exampleExperimentTemplate = new aws.fis.ExperimentTemplate("example", {
 *     description: "example",
 *     roleArn: example.arn,
 *     stopConditions: [{
 *         source: "none",
 *     }],
 *     actions: [{
 *         name: "example-action",
 *         actionId: "aws:ec2:terminate-instances",
 *         target: {
 *             key: "Instances",
 *             value: "example-target",
 *         },
 *     }],
 *     targets: [{
 *         name: "example-target",
 *         resourceType: "aws:ec2:instance",
 *         selectionMode: "COUNT(1)",
 *         resourceTags: [{
 *             key: "env",
 *             value: "example",
 *         }],
 *     }],
 *     experimentReportConfiguration: {
 *         dataSources: {
 *             cloudwatchDashboards: [{
 *                 dashboardArn: exampleAwsCloudwatchDashboard.dashboardArn,
 *             }],
 *         },
 *         outputs: {
 *             s3Configuration: {
 *                 bucketName: exampleAwsS3Bucket.bucket,
 *                 prefix: "fis-example-reports",
 *             },
 *         },
 *         postExperimentDuration: "PT10M",
 *         preExperimentDuration: "PT10M",
 *     },
 *     tags: {
 *         Name: "example",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import FIS Experiment Templates using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:fis/experimentTemplate:ExperimentTemplate template EXT123AbCdEfGhIjK
 * ```
 */
export declare class ExperimentTemplate extends pulumi.CustomResource {
    /**
     * Get an existing ExperimentTemplate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ExperimentTemplateState, opts?: pulumi.CustomResourceOptions): ExperimentTemplate;
    /**
     * Returns true if the given object is an instance of ExperimentTemplate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ExperimentTemplate;
    /**
     * Action to be performed during an experiment. See below.
     */
    readonly actions: pulumi.Output<outputs.fis.ExperimentTemplateAction[]>;
    /**
     * Description for the experiment template.
     */
    readonly description: pulumi.Output<string>;
    /**
     * The experiment options for the experiment template. See experimentOptions below for more details!
     */
    readonly experimentOptions: pulumi.Output<outputs.fis.ExperimentTemplateExperimentOptions>;
    /**
     * The configuration for [experiment reporting](https://docs.aws.amazon.com/fis/latest/userguide/experiment-report-configuration.html). See below.
     */
    readonly experimentReportConfiguration: pulumi.Output<outputs.fis.ExperimentTemplateExperimentReportConfiguration | undefined>;
    /**
     * The configuration for experiment logging. See below.
     */
    readonly logConfiguration: pulumi.Output<outputs.fis.ExperimentTemplateLogConfiguration | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of an IAM role that grants the AWS FIS service permission to perform service actions on your behalf.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * When an ongoing experiment should be stopped. See below.
     *
     * The following arguments are optional:
     */
    readonly stopConditions: pulumi.Output<outputs.fis.ExperimentTemplateStopCondition[]>;
    /**
     * Key-value mapping of tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Target of an action. See below.
     */
    readonly targets: pulumi.Output<outputs.fis.ExperimentTemplateTarget[] | undefined>;
    /**
     * Create a ExperimentTemplate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ExperimentTemplateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ExperimentTemplate resources.
 */
export interface ExperimentTemplateState {
    /**
     * Action to be performed during an experiment. See below.
     */
    actions?: pulumi.Input<pulumi.Input<inputs.fis.ExperimentTemplateAction>[]>;
    /**
     * Description for the experiment template.
     */
    description?: pulumi.Input<string>;
    /**
     * The experiment options for the experiment template. See experimentOptions below for more details!
     */
    experimentOptions?: pulumi.Input<inputs.fis.ExperimentTemplateExperimentOptions>;
    /**
     * The configuration for [experiment reporting](https://docs.aws.amazon.com/fis/latest/userguide/experiment-report-configuration.html). See below.
     */
    experimentReportConfiguration?: pulumi.Input<inputs.fis.ExperimentTemplateExperimentReportConfiguration>;
    /**
     * The configuration for experiment logging. See below.
     */
    logConfiguration?: pulumi.Input<inputs.fis.ExperimentTemplateLogConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of an IAM role that grants the AWS FIS service permission to perform service actions on your behalf.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * When an ongoing experiment should be stopped. See below.
     *
     * The following arguments are optional:
     */
    stopConditions?: pulumi.Input<pulumi.Input<inputs.fis.ExperimentTemplateStopCondition>[]>;
    /**
     * Key-value mapping of tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Target of an action. See below.
     */
    targets?: pulumi.Input<pulumi.Input<inputs.fis.ExperimentTemplateTarget>[]>;
}
/**
 * The set of arguments for constructing a ExperimentTemplate resource.
 */
export interface ExperimentTemplateArgs {
    /**
     * Action to be performed during an experiment. See below.
     */
    actions: pulumi.Input<pulumi.Input<inputs.fis.ExperimentTemplateAction>[]>;
    /**
     * Description for the experiment template.
     */
    description: pulumi.Input<string>;
    /**
     * The experiment options for the experiment template. See experimentOptions below for more details!
     */
    experimentOptions?: pulumi.Input<inputs.fis.ExperimentTemplateExperimentOptions>;
    /**
     * The configuration for [experiment reporting](https://docs.aws.amazon.com/fis/latest/userguide/experiment-report-configuration.html). See below.
     */
    experimentReportConfiguration?: pulumi.Input<inputs.fis.ExperimentTemplateExperimentReportConfiguration>;
    /**
     * The configuration for experiment logging. See below.
     */
    logConfiguration?: pulumi.Input<inputs.fis.ExperimentTemplateLogConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of an IAM role that grants the AWS FIS service permission to perform service actions on your behalf.
     */
    roleArn: pulumi.Input<string>;
    /**
     * When an ongoing experiment should be stopped. See below.
     *
     * The following arguments are optional:
     */
    stopConditions: pulumi.Input<pulumi.Input<inputs.fis.ExperimentTemplateStopCondition>[]>;
    /**
     * Key-value mapping of tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Target of an action. See below.
     */
    targets?: pulumi.Input<pulumi.Input<inputs.fis.ExperimentTemplateTarget>[]>;
}
