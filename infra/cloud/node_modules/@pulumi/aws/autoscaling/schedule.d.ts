import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AutoScaling Schedule resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const foobar = new aws.autoscaling.Group("foobar", {
 *     availabilityZones: ["us-west-2a"],
 *     name: "test-foobar5",
 *     maxSize: 1,
 *     minSize: 1,
 *     healthCheckGracePeriod: 300,
 *     healthCheckType: "ELB",
 *     forceDelete: true,
 *     terminationPolicies: ["OldestInstance"],
 * });
 * const foobarSchedule = new aws.autoscaling.Schedule("foobar", {
 *     scheduledActionName: "foobar",
 *     minSize: 0,
 *     maxSize: 1,
 *     desiredCapacity: 0,
 *     startTime: "2016-12-11T18:00:00Z",
 *     endTime: "2016-12-12T06:00:00Z",
 *     autoscalingGroupName: foobar.name,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AutoScaling ScheduledAction using the `auto-scaling-group-name` and `scheduled-action-name`. For example:
 *
 * ```sh
 * $ pulumi import aws:autoscaling/schedule:Schedule resource-name auto-scaling-group-name/scheduled-action-name
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
     * ARN assigned by AWS to the autoscaling schedule.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The name of the Auto Scaling group.
     */
    readonly autoscalingGroupName: pulumi.Output<string>;
    /**
     * The initial capacity of the Auto Scaling group after the scheduled action runs and the capacity it attempts to maintain. Set to `-1` if you don't want to change the desired capacity at the scheduled time. Defaults to `0`.
     */
    readonly desiredCapacity: pulumi.Output<number>;
    /**
     * The date and time for the recurring schedule to end, in UTC with the format `"YYYY-MM-DDThh:mm:ssZ"` (e.g. `"2021-06-01T00:00:00Z"`).
     */
    readonly endTime: pulumi.Output<string>;
    /**
     * The maximum size of the Auto Scaling group. Set to `-1` if you don't want to change the maximum size at the scheduled time. Defaults to `0`.
     */
    readonly maxSize: pulumi.Output<number>;
    /**
     * The minimum size of the Auto Scaling group. Set to `-1` if you don't want to change the minimum size at the scheduled time. Defaults to `0`.
     */
    readonly minSize: pulumi.Output<number>;
    /**
     * The recurring schedule for this action specified using the Unix cron syntax format.
     */
    readonly recurrence: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name of this scaling action.
     *
     * The following arguments are optional:
     */
    readonly scheduledActionName: pulumi.Output<string>;
    /**
     * The date and time for the recurring schedule to start, in UTC with the format `"YYYY-MM-DDThh:mm:ssZ"` (e.g. `"2021-06-01T00:00:00Z"`).
     */
    readonly startTime: pulumi.Output<string>;
    /**
     * Specifies the time zone for a cron expression. Valid values are the canonical names of the IANA time zones (such as `Etc/GMT+9` or `Pacific/Tahiti`).
     *
     * > **NOTE:** When `startTime` and `endTime` are specified with `recurrence` , they form the boundaries of when the recurring action will start and stop.
     */
    readonly timeZone: pulumi.Output<string>;
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
     * ARN assigned by AWS to the autoscaling schedule.
     */
    arn?: pulumi.Input<string>;
    /**
     * The name of the Auto Scaling group.
     */
    autoscalingGroupName?: pulumi.Input<string>;
    /**
     * The initial capacity of the Auto Scaling group after the scheduled action runs and the capacity it attempts to maintain. Set to `-1` if you don't want to change the desired capacity at the scheduled time. Defaults to `0`.
     */
    desiredCapacity?: pulumi.Input<number>;
    /**
     * The date and time for the recurring schedule to end, in UTC with the format `"YYYY-MM-DDThh:mm:ssZ"` (e.g. `"2021-06-01T00:00:00Z"`).
     */
    endTime?: pulumi.Input<string>;
    /**
     * The maximum size of the Auto Scaling group. Set to `-1` if you don't want to change the maximum size at the scheduled time. Defaults to `0`.
     */
    maxSize?: pulumi.Input<number>;
    /**
     * The minimum size of the Auto Scaling group. Set to `-1` if you don't want to change the minimum size at the scheduled time. Defaults to `0`.
     */
    minSize?: pulumi.Input<number>;
    /**
     * The recurring schedule for this action specified using the Unix cron syntax format.
     */
    recurrence?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of this scaling action.
     *
     * The following arguments are optional:
     */
    scheduledActionName?: pulumi.Input<string>;
    /**
     * The date and time for the recurring schedule to start, in UTC with the format `"YYYY-MM-DDThh:mm:ssZ"` (e.g. `"2021-06-01T00:00:00Z"`).
     */
    startTime?: pulumi.Input<string>;
    /**
     * Specifies the time zone for a cron expression. Valid values are the canonical names of the IANA time zones (such as `Etc/GMT+9` or `Pacific/Tahiti`).
     *
     * > **NOTE:** When `startTime` and `endTime` are specified with `recurrence` , they form the boundaries of when the recurring action will start and stop.
     */
    timeZone?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Schedule resource.
 */
export interface ScheduleArgs {
    /**
     * The name of the Auto Scaling group.
     */
    autoscalingGroupName: pulumi.Input<string>;
    /**
     * The initial capacity of the Auto Scaling group after the scheduled action runs and the capacity it attempts to maintain. Set to `-1` if you don't want to change the desired capacity at the scheduled time. Defaults to `0`.
     */
    desiredCapacity?: pulumi.Input<number>;
    /**
     * The date and time for the recurring schedule to end, in UTC with the format `"YYYY-MM-DDThh:mm:ssZ"` (e.g. `"2021-06-01T00:00:00Z"`).
     */
    endTime?: pulumi.Input<string>;
    /**
     * The maximum size of the Auto Scaling group. Set to `-1` if you don't want to change the maximum size at the scheduled time. Defaults to `0`.
     */
    maxSize?: pulumi.Input<number>;
    /**
     * The minimum size of the Auto Scaling group. Set to `-1` if you don't want to change the minimum size at the scheduled time. Defaults to `0`.
     */
    minSize?: pulumi.Input<number>;
    /**
     * The recurring schedule for this action specified using the Unix cron syntax format.
     */
    recurrence?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of this scaling action.
     *
     * The following arguments are optional:
     */
    scheduledActionName: pulumi.Input<string>;
    /**
     * The date and time for the recurring schedule to start, in UTC with the format `"YYYY-MM-DDThh:mm:ssZ"` (e.g. `"2021-06-01T00:00:00Z"`).
     */
    startTime?: pulumi.Input<string>;
    /**
     * Specifies the time zone for a cron expression. Valid values are the canonical names of the IANA time zones (such as `Etc/GMT+9` or `Pacific/Tahiti`).
     *
     * > **NOTE:** When `startTime` and `endTime` are specified with `recurrence` , they form the boundaries of when the recurring action will start and stop.
     */
    timeZone?: pulumi.Input<string>;
}
