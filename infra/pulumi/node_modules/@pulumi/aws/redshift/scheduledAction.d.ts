import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ### Pause Cluster Action
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const assumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["scheduler.redshift.amazonaws.com"],
 *         }],
 *         actions: ["sts:AssumeRole"],
 *     }],
 * });
 * const exampleRole = new aws.iam.Role("example", {
 *     name: "redshift_scheduled_action",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const example = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         actions: [
 *             "redshift:PauseCluster",
 *             "redshift:ResumeCluster",
 *             "redshift:ResizeCluster",
 *         ],
 *         resources: ["*"],
 *     }],
 * });
 * const examplePolicy = new aws.iam.Policy("example", {
 *     name: "redshift_scheduled_action",
 *     policy: example.then(example => example.json),
 * });
 * const exampleRolePolicyAttachment = new aws.iam.RolePolicyAttachment("example", {
 *     policyArn: examplePolicy.arn,
 *     role: exampleRole.name,
 * });
 * const exampleScheduledAction = new aws.redshift.ScheduledAction("example", {
 *     name: "tf-redshift-scheduled-action",
 *     schedule: "cron(00 23 * * ? *)",
 *     iamRole: exampleRole.arn,
 *     targetAction: {
 *         pauseCluster: {
 *             clusterIdentifier: "tf-redshift001",
 *         },
 *     },
 * });
 * ```
 *
 * ### Resize Cluster Action
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshift.ScheduledAction("example", {
 *     name: "tf-redshift-scheduled-action",
 *     schedule: "cron(00 23 * * ? *)",
 *     iamRole: exampleAwsIamRole.arn,
 *     targetAction: {
 *         resizeCluster: {
 *             clusterIdentifier: "tf-redshift001",
 *             clusterType: "multi-node",
 *             nodeType: "dc1.large",
 *             numberOfNodes: 2,
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Scheduled Action using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshift/scheduledAction:ScheduledAction example tf-redshift-scheduled-action
 * ```
 */
export declare class ScheduledAction extends pulumi.CustomResource {
    /**
     * Get an existing ScheduledAction resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ScheduledActionState, opts?: pulumi.CustomResourceOptions): ScheduledAction;
    /**
     * Returns true if the given object is an instance of ScheduledAction.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ScheduledAction;
    /**
     * The description of the scheduled action.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Whether to enable the scheduled action. Default is `true` .
     */
    readonly enable: pulumi.Output<boolean | undefined>;
    /**
     * The end time in UTC when the schedule is active, in UTC RFC3339 format(for example, YYYY-MM-DDTHH:MM:SSZ).
     */
    readonly endTime: pulumi.Output<string | undefined>;
    /**
     * The IAM role to assume to run the scheduled action.
     */
    readonly iamRole: pulumi.Output<string>;
    /**
     * The scheduled action name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The schedule of action. The schedule is defined format of "at expression" or "cron expression", for example `at(2016-03-04T17:27:00)` or `cron(0 10 ? * MON *)`. See [Scheduled Action](https://docs.aws.amazon.com/redshift/latest/APIReference/API_ScheduledAction.html) for more information.
     */
    readonly schedule: pulumi.Output<string>;
    /**
     * The start time in UTC when the schedule is active, in UTC RFC3339 format(for example, YYYY-MM-DDTHH:MM:SSZ).
     */
    readonly startTime: pulumi.Output<string | undefined>;
    /**
     * Target action. Documented below.
     */
    readonly targetAction: pulumi.Output<outputs.redshift.ScheduledActionTargetAction>;
    /**
     * Create a ScheduledAction resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ScheduledActionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ScheduledAction resources.
 */
export interface ScheduledActionState {
    /**
     * The description of the scheduled action.
     */
    description?: pulumi.Input<string>;
    /**
     * Whether to enable the scheduled action. Default is `true` .
     */
    enable?: pulumi.Input<boolean>;
    /**
     * The end time in UTC when the schedule is active, in UTC RFC3339 format(for example, YYYY-MM-DDTHH:MM:SSZ).
     */
    endTime?: pulumi.Input<string>;
    /**
     * The IAM role to assume to run the scheduled action.
     */
    iamRole?: pulumi.Input<string>;
    /**
     * The scheduled action name.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The schedule of action. The schedule is defined format of "at expression" or "cron expression", for example `at(2016-03-04T17:27:00)` or `cron(0 10 ? * MON *)`. See [Scheduled Action](https://docs.aws.amazon.com/redshift/latest/APIReference/API_ScheduledAction.html) for more information.
     */
    schedule?: pulumi.Input<string>;
    /**
     * The start time in UTC when the schedule is active, in UTC RFC3339 format(for example, YYYY-MM-DDTHH:MM:SSZ).
     */
    startTime?: pulumi.Input<string>;
    /**
     * Target action. Documented below.
     */
    targetAction?: pulumi.Input<inputs.redshift.ScheduledActionTargetAction>;
}
/**
 * The set of arguments for constructing a ScheduledAction resource.
 */
export interface ScheduledActionArgs {
    /**
     * The description of the scheduled action.
     */
    description?: pulumi.Input<string>;
    /**
     * Whether to enable the scheduled action. Default is `true` .
     */
    enable?: pulumi.Input<boolean>;
    /**
     * The end time in UTC when the schedule is active, in UTC RFC3339 format(for example, YYYY-MM-DDTHH:MM:SSZ).
     */
    endTime?: pulumi.Input<string>;
    /**
     * The IAM role to assume to run the scheduled action.
     */
    iamRole: pulumi.Input<string>;
    /**
     * The scheduled action name.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The schedule of action. The schedule is defined format of "at expression" or "cron expression", for example `at(2016-03-04T17:27:00)` or `cron(0 10 ? * MON *)`. See [Scheduled Action](https://docs.aws.amazon.com/redshift/latest/APIReference/API_ScheduledAction.html) for more information.
     */
    schedule: pulumi.Input<string>;
    /**
     * The start time in UTC when the schedule is active, in UTC RFC3339 format(for example, YYYY-MM-DDTHH:MM:SSZ).
     */
    startTime?: pulumi.Input<string>;
    /**
     * Target action. Documented below.
     */
    targetAction: pulumi.Input<inputs.redshift.ScheduledActionTargetAction>;
}
