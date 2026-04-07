import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an EventBridge Scheduler Schedule resource.
 *
 * You can find out more about EventBridge Scheduler in the [User Guide](https://docs.aws.amazon.com/scheduler/latest/UserGuide/what-is-scheduler.html).
 *
 * > **Note:** EventBridge was formerly known as CloudWatch Events. The functionality is identical.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.scheduler.Schedule("example", {
 *     name: "my-schedule",
 *     groupName: "default",
 *     flexibleTimeWindow: {
 *         mode: "OFF",
 *     },
 *     scheduleExpression: "rate(1 hours)",
 *     target: {
 *         arn: exampleAwsSqsQueue.arn,
 *         roleArn: exampleAwsIamRole.arn,
 *     },
 * });
 * ```
 *
 * ### Universal Target
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sqs.Queue("example", {});
 * const exampleSchedule = new aws.scheduler.Schedule("example", {
 *     name: "my-schedule",
 *     flexibleTimeWindow: {
 *         mode: "OFF",
 *     },
 *     scheduleExpression: "rate(1 hours)",
 *     target: {
 *         arn: "arn:aws:scheduler:::aws-sdk:sqs:sendMessage",
 *         roleArn: exampleAwsIamRole.arn,
 *         input: pulumi.jsonStringify({
 *             MessageBody: "Greetings, programs!",
 *             QueueUrl: example.url,
 *         }),
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import schedules using the combination `group_name/name`. For example:
 *
 * ```sh
 * $ pulumi import aws:scheduler/schedule:Schedule example my-schedule-group/my-schedule
 * ```
 */
export declare class Schedule extends pulumi.CustomResource {
    /**
     * Get an existing Schedule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ScheduleState, opts?: pulumi.CustomResourceOptions): Schedule;
    /**
     * Returns true if the given object is an instance of Schedule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Schedule;
    /**
     * Action that applies to the schedule after completing invocation of the target. Valid values are `NONE` and `DELETE`. Defaults to `NONE`.
     */
    readonly actionAfterCompletion: pulumi.Output<string>;
    /**
     * ARN of the schedule.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Brief description of the schedule.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The date, in UTC, before which the schedule can invoke its target. Depending on the schedule's recurrence expression, invocations might stop on, or before, the end date you specify. EventBridge Scheduler ignores the end date for one-time schedules. Example: `2030-01-01T01:00:00Z`.
     */
    readonly endDate: pulumi.Output<string | undefined>;
    /**
     * Configures a time window during which EventBridge Scheduler invokes the schedule. Detailed below.
     */
    readonly flexibleTimeWindow: pulumi.Output<outputs.scheduler.ScheduleFlexibleTimeWindow>;
    /**
     * Name of the schedule group to associate with this schedule. When omitted, the `default` schedule group is used.
     */
    readonly groupName: pulumi.Output<string>;
    /**
     * ARN for the customer managed KMS key that EventBridge Scheduler will use to encrypt and decrypt your data.
     */
    readonly kmsKeyArn: pulumi.Output<string | undefined>;
    /**
     * Name of the schedule. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Defines when the schedule runs. Read more in [Schedule types on EventBridge Scheduler](https://docs.aws.amazon.com/scheduler/latest/UserGuide/schedule-types.html).
     */
    readonly scheduleExpression: pulumi.Output<string>;
    /**
     * Timezone in which the scheduling expression is evaluated. Defaults to `UTC`. Example: `Australia/Sydney`.
     */
    readonly scheduleExpressionTimezone: pulumi.Output<string | undefined>;
    /**
     * The date, in UTC, after which the schedule can begin invoking its target. Depending on the schedule's recurrence expression, invocations might occur on, or after, the start date you specify. EventBridge Scheduler ignores the start date for one-time schedules. Example: `2030-01-01T01:00:00Z`.
     */
    readonly startDate: pulumi.Output<string | undefined>;
    /**
     * Specifies whether the schedule is enabled or disabled. One of: `ENABLED` (default), `DISABLED`.
     */
    readonly state: pulumi.Output<string | undefined>;
    /**
     * Configures the target of the schedule. Detailed below.
     *
     * The following arguments are optional:
     */
    readonly target: pulumi.Output<outputs.scheduler.ScheduleTarget>;
    /**
     * Create a Schedule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ScheduleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Schedule resources.
 */
export interface ScheduleState {
    /**
     * Action that applies to the schedule after completing invocation of the target. Valid values are `NONE` and `DELETE`. Defaults to `NONE`.
     */
    actionAfterCompletion?: pulumi.Input<string>;
    /**
     * ARN of the schedule.
     */
    arn?: pulumi.Input<string>;
    /**
     * Brief description of the schedule.
     */
    description?: pulumi.Input<string>;
    /**
     * The date, in UTC, before which the schedule can invoke its target. Depending on the schedule's recurrence expression, invocations might stop on, or before, the end date you specify. EventBridge Scheduler ignores the end date for one-time schedules. Example: `2030-01-01T01:00:00Z`.
     */
    endDate?: pulumi.Input<string>;
    /**
     * Configures a time window during which EventBridge Scheduler invokes the schedule. Detailed below.
     */
    flexibleTimeWindow?: pulumi.Input<inputs.scheduler.ScheduleFlexibleTimeWindow>;
    /**
     * Name of the schedule group to associate with this schedule. When omitted, the `default` schedule group is used.
     */
    groupName?: pulumi.Input<string>;
    /**
     * ARN for the customer managed KMS key that EventBridge Scheduler will use to encrypt and decrypt your data.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Name of the schedule. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Defines when the schedule runs. Read more in [Schedule types on EventBridge Scheduler](https://docs.aws.amazon.com/scheduler/latest/UserGuide/schedule-types.html).
     */
    scheduleExpression?: pulumi.Input<string>;
    /**
     * Timezone in which the scheduling expression is evaluated. Defaults to `UTC`. Example: `Australia/Sydney`.
     */
    scheduleExpressionTimezone?: pulumi.Input<string>;
    /**
     * The date, in UTC, after which the schedule can begin invoking its target. Depending on the schedule's recurrence expression, invocations might occur on, or after, the start date you specify. EventBridge Scheduler ignores the start date for one-time schedules. Example: `2030-01-01T01:00:00Z`.
     */
    startDate?: pulumi.Input<string>;
    /**
     * Specifies whether the schedule is enabled or disabled. One of: `ENABLED` (default), `DISABLED`.
     */
    state?: pulumi.Input<string>;
    /**
     * Configures the target of the schedule. Detailed below.
     *
     * The following arguments are optional:
     */
    target?: pulumi.Input<inputs.scheduler.ScheduleTarget>;
}
/**
 * The set of arguments for constructing a Schedule resource.
 */
export interface ScheduleArgs {
    /**
     * Action that applies to the schedule after completing invocation of the target. Valid values are `NONE` and `DELETE`. Defaults to `NONE`.
     */
    actionAfterCompletion?: pulumi.Input<string>;
    /**
     * Brief description of the schedule.
     */
    description?: pulumi.Input<string>;
    /**
     * The date, in UTC, before which the schedule can invoke its target. Depending on the schedule's recurrence expression, invocations might stop on, or before, the end date you specify. EventBridge Scheduler ignores the end date for one-time schedules. Example: `2030-01-01T01:00:00Z`.
     */
    endDate?: pulumi.Input<string>;
    /**
     * Configures a time window during which EventBridge Scheduler invokes the schedule. Detailed below.
     */
    flexibleTimeWindow: pulumi.Input<inputs.scheduler.ScheduleFlexibleTimeWindow>;
    /**
     * Name of the schedule group to associate with this schedule. When omitted, the `default` schedule group is used.
     */
    groupName?: pulumi.Input<string>;
    /**
     * ARN for the customer managed KMS key that EventBridge Scheduler will use to encrypt and decrypt your data.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Name of the schedule. If omitted, the provider will assign a random, unique name. Conflicts with `namePrefix`.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Defines when the schedule runs. Read more in [Schedule types on EventBridge Scheduler](https://docs.aws.amazon.com/scheduler/latest/UserGuide/schedule-types.html).
     */
    scheduleExpression: pulumi.Input<string>;
    /**
     * Timezone in which the scheduling expression is evaluated. Defaults to `UTC`. Example: `Australia/Sydney`.
     */
    scheduleExpressionTimezone?: pulumi.Input<string>;
    /**
     * The date, in UTC, after which the schedule can begin invoking its target. Depending on the schedule's recurrence expression, invocations might occur on, or after, the start date you specify. EventBridge Scheduler ignores the start date for one-time schedules. Example: `2030-01-01T01:00:00Z`.
     */
    startDate?: pulumi.Input<string>;
    /**
     * Specifies whether the schedule is enabled or disabled. One of: `ENABLED` (default), `DISABLED`.
     */
    state?: pulumi.Input<string>;
    /**
     * Configures the target of the schedule. Detailed below.
     *
     * The following arguments are optional:
     */
    target: pulumi.Input<inputs.scheduler.ScheduleTarget>;
}
