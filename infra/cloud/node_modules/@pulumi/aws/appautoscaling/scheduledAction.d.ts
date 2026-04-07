import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Application AutoScaling ScheduledAction resource.
 *
 * ## Example Usage
 *
 * ### DynamoDB Table Autoscaling
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const dynamodb = new aws.appautoscaling.Target("dynamodb", {
 *     maxCapacity: 100,
 *     minCapacity: 5,
 *     resourceId: "table/tableName",
 *     scalableDimension: "dynamodb:table:ReadCapacityUnits",
 *     serviceNamespace: "dynamodb",
 * });
 * const dynamodbScheduledAction = new aws.appautoscaling.ScheduledAction("dynamodb", {
 *     name: "dynamodb",
 *     serviceNamespace: dynamodb.serviceNamespace,
 *     resourceId: dynamodb.resourceId,
 *     scalableDimension: dynamodb.scalableDimension,
 *     schedule: "at(2006-01-02T15:04:05)",
 *     scalableTargetAction: {
 *         minCapacity: 1,
 *         maxCapacity: 200,
 *     },
 * });
 * ```
 *
 * ### ECS Service Autoscaling
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const ecs = new aws.appautoscaling.Target("ecs", {
 *     maxCapacity: 4,
 *     minCapacity: 1,
 *     resourceId: "service/clusterName/serviceName",
 *     scalableDimension: "ecs:service:DesiredCount",
 *     serviceNamespace: "ecs",
 * });
 * const ecsScheduledAction = new aws.appautoscaling.ScheduledAction("ecs", {
 *     name: "ecs",
 *     serviceNamespace: ecs.serviceNamespace,
 *     resourceId: ecs.resourceId,
 *     scalableDimension: ecs.scalableDimension,
 *     schedule: "at(2006-01-02T15:04:05)",
 *     scalableTargetAction: {
 *         minCapacity: 1,
 *         maxCapacity: 10,
 *     },
 * });
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
     * ARN of the scheduled action.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Date and time for the scheduled action to end in RFC 3339 format. The timezone is not affected by the setting of `timezone`.
     */
    readonly endTime: pulumi.Output<string | undefined>;
    /**
     * Name of the scheduled action.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Identifier of the resource associated with the scheduled action. Documentation can be found in the `ResourceId` parameter at: [AWS Application Auto Scaling API Reference](https://docs.aws.amazon.com/autoscaling/application/APIReference/API_PutScheduledAction.html)
     */
    readonly resourceId: pulumi.Output<string>;
    /**
     * Scalable dimension. Documentation can be found in the `ScalableDimension` parameter at: [AWS Application Auto Scaling API Reference](https://docs.aws.amazon.com/autoscaling/application/APIReference/API_PutScheduledAction.html) Example: ecs:service:DesiredCount
     */
    readonly scalableDimension: pulumi.Output<string>;
    /**
     * New minimum and maximum capacity. You can set both values or just one. See below
     */
    readonly scalableTargetAction: pulumi.Output<outputs.appautoscaling.ScheduledActionScalableTargetAction>;
    /**
     * Schedule for this action. The following formats are supported: At expressions - at(yyyy-mm-ddThh:mm:ss), Rate expressions - rate(valueunit), Cron expressions - cron(fields). Times for at expressions and cron expressions are evaluated using the time zone configured in `timezone`. Documentation can be found in the `Timezone` parameter at: [AWS Application Auto Scaling API Reference](https://docs.aws.amazon.com/autoscaling/application/APIReference/API_PutScheduledAction.html)
     */
    readonly schedule: pulumi.Output<string>;
    /**
     * Namespace of the AWS service. Documentation can be found in the `ServiceNamespace` parameter at: [AWS Application Auto Scaling API Reference](https://docs.aws.amazon.com/autoscaling/application/APIReference/API_PutScheduledAction.html) Example: ecs
     */
    readonly serviceNamespace: pulumi.Output<string>;
    /**
     * Date and time for the scheduled action to start in RFC 3339 format. The timezone is not affected by the setting of `timezone`.
     */
    readonly startTime: pulumi.Output<string | undefined>;
    /**
     * Time zone used when setting a scheduled action by using an at or cron expression. Does not affect timezone for `startTime` and `endTime`. Valid values are the [canonical names of the IANA time zones supported by Joda-Time](https://www.joda.org/joda-time/timezones.html), such as `Etc/GMT+9` or `Pacific/Tahiti`. Default is `UTC`.
     */
    readonly timezone: pulumi.Output<string | undefined>;
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
     * ARN of the scheduled action.
     */
    arn?: pulumi.Input<string>;
    /**
     * Date and time for the scheduled action to end in RFC 3339 format. The timezone is not affected by the setting of `timezone`.
     */
    endTime?: pulumi.Input<string>;
    /**
     * Name of the scheduled action.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of the resource associated with the scheduled action. Documentation can be found in the `ResourceId` parameter at: [AWS Application Auto Scaling API Reference](https://docs.aws.amazon.com/autoscaling/application/APIReference/API_PutScheduledAction.html)
     */
    resourceId?: pulumi.Input<string>;
    /**
     * Scalable dimension. Documentation can be found in the `ScalableDimension` parameter at: [AWS Application Auto Scaling API Reference](https://docs.aws.amazon.com/autoscaling/application/APIReference/API_PutScheduledAction.html) Example: ecs:service:DesiredCount
     */
    scalableDimension?: pulumi.Input<string>;
    /**
     * New minimum and maximum capacity. You can set both values or just one. See below
     */
    scalableTargetAction?: pulumi.Input<inputs.appautoscaling.ScheduledActionScalableTargetAction>;
    /**
     * Schedule for this action. The following formats are supported: At expressions - at(yyyy-mm-ddThh:mm:ss), Rate expressions - rate(valueunit), Cron expressions - cron(fields). Times for at expressions and cron expressions are evaluated using the time zone configured in `timezone`. Documentation can be found in the `Timezone` parameter at: [AWS Application Auto Scaling API Reference](https://docs.aws.amazon.com/autoscaling/application/APIReference/API_PutScheduledAction.html)
     */
    schedule?: pulumi.Input<string>;
    /**
     * Namespace of the AWS service. Documentation can be found in the `ServiceNamespace` parameter at: [AWS Application Auto Scaling API Reference](https://docs.aws.amazon.com/autoscaling/application/APIReference/API_PutScheduledAction.html) Example: ecs
     */
    serviceNamespace?: pulumi.Input<string>;
    /**
     * Date and time for the scheduled action to start in RFC 3339 format. The timezone is not affected by the setting of `timezone`.
     */
    startTime?: pulumi.Input<string>;
    /**
     * Time zone used when setting a scheduled action by using an at or cron expression. Does not affect timezone for `startTime` and `endTime`. Valid values are the [canonical names of the IANA time zones supported by Joda-Time](https://www.joda.org/joda-time/timezones.html), such as `Etc/GMT+9` or `Pacific/Tahiti`. Default is `UTC`.
     */
    timezone?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ScheduledAction resource.
 */
export interface ScheduledActionArgs {
    /**
     * Date and time for the scheduled action to end in RFC 3339 format. The timezone is not affected by the setting of `timezone`.
     */
    endTime?: pulumi.Input<string>;
    /**
     * Name of the scheduled action.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of the resource associated with the scheduled action. Documentation can be found in the `ResourceId` parameter at: [AWS Application Auto Scaling API Reference](https://docs.aws.amazon.com/autoscaling/application/APIReference/API_PutScheduledAction.html)
     */
    resourceId: pulumi.Input<string>;
    /**
     * Scalable dimension. Documentation can be found in the `ScalableDimension` parameter at: [AWS Application Auto Scaling API Reference](https://docs.aws.amazon.com/autoscaling/application/APIReference/API_PutScheduledAction.html) Example: ecs:service:DesiredCount
     */
    scalableDimension: pulumi.Input<string>;
    /**
     * New minimum and maximum capacity. You can set both values or just one. See below
     */
    scalableTargetAction: pulumi.Input<inputs.appautoscaling.ScheduledActionScalableTargetAction>;
    /**
     * Schedule for this action. The following formats are supported: At expressions - at(yyyy-mm-ddThh:mm:ss), Rate expressions - rate(valueunit), Cron expressions - cron(fields). Times for at expressions and cron expressions are evaluated using the time zone configured in `timezone`. Documentation can be found in the `Timezone` parameter at: [AWS Application Auto Scaling API Reference](https://docs.aws.amazon.com/autoscaling/application/APIReference/API_PutScheduledAction.html)
     */
    schedule: pulumi.Input<string>;
    /**
     * Namespace of the AWS service. Documentation can be found in the `ServiceNamespace` parameter at: [AWS Application Auto Scaling API Reference](https://docs.aws.amazon.com/autoscaling/application/APIReference/API_PutScheduledAction.html) Example: ecs
     */
    serviceNamespace: pulumi.Input<string>;
    /**
     * Date and time for the scheduled action to start in RFC 3339 format. The timezone is not affected by the setting of `timezone`.
     */
    startTime?: pulumi.Input<string>;
    /**
     * Time zone used when setting a scheduled action by using an at or cron expression. Does not affect timezone for `startTime` and `endTime`. Valid values are the [canonical names of the IANA time zones supported by Joda-Time](https://www.joda.org/joda-time/timezones.html), such as `Etc/GMT+9` or `Pacific/Tahiti`. Default is `UTC`.
     */
    timezone?: pulumi.Input<string>;
}
