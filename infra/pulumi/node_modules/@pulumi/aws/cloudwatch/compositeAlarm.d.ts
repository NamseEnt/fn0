import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a CloudWatch Composite Alarm resource.
 *
 * > **NOTE:** An alarm (composite or metric) cannot be destroyed when there are other composite alarms depending on it. This can lead to a cyclical dependency on update, as the provider will unsuccessfully attempt to destroy alarms before updating the rule. Consider using `dependsOn`, references to alarm names, and two-stage updates.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudwatch.CompositeAlarm("example", {
 *     alarmDescription: "This is a composite alarm!",
 *     alarmName: "example-composite-alarm",
 *     alarmActions: exampleAwsSnsTopic.arn,
 *     okActions: exampleAwsSnsTopic.arn,
 *     alarmRule: `ALARM(${alpha.alarmName}) OR
 * ALARM(${bravo.alarmName})
 * `,
 *     actionsSuppressor: {
 *         alarm: "suppressor-alarm",
 *         extensionPeriod: 10,
 *         waitPeriod: 20,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a CloudWatch Composite Alarm using the `alarm_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/compositeAlarm:CompositeAlarm test my-alarm
 * ```
 */
export declare class CompositeAlarm extends pulumi.CustomResource {
    /**
     * Get an existing CompositeAlarm resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CompositeAlarmState, opts?: pulumi.CustomResourceOptions): CompositeAlarm;
    /**
     * Returns true if the given object is an instance of CompositeAlarm.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CompositeAlarm;
    /**
     * Indicates whether actions should be executed during any changes to the alarm state of the composite alarm. Defaults to `true`.
     */
    readonly actionsEnabled: pulumi.Output<boolean | undefined>;
    /**
     * Actions will be suppressed if the suppressor alarm is in the ALARM state.
     */
    readonly actionsSuppressor: pulumi.Output<outputs.cloudwatch.CompositeAlarmActionsSuppressor | undefined>;
    /**
     * The set of actions to execute when this alarm transitions to the `ALARM` state from any other state. Each action is specified as an ARN. Up to 5 actions are allowed.
     */
    readonly alarmActions: pulumi.Output<string[] | undefined>;
    /**
     * The description for the composite alarm.
     */
    readonly alarmDescription: pulumi.Output<string | undefined>;
    /**
     * The name for the composite alarm. This name must be unique within the region.
     */
    readonly alarmName: pulumi.Output<string>;
    /**
     * An expression that specifies which other alarms are to be evaluated to determine this composite alarm's state. For syntax, see [Creating a Composite Alarm](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/Create_Composite_Alarm.html). The maximum length is 10240 characters.
     */
    readonly alarmRule: pulumi.Output<string>;
    /**
     * The ARN of the composite alarm.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The set of actions to execute when this alarm transitions to the `INSUFFICIENT_DATA` state from any other state. Each action is specified as an ARN. Up to 5 actions are allowed.
     */
    readonly insufficientDataActions: pulumi.Output<string[] | undefined>;
    /**
     * The set of actions to execute when this alarm transitions to an `OK` state from any other state. Each action is specified as an ARN. Up to 5 actions are allowed.
     */
    readonly okActions: pulumi.Output<string[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to associate with the alarm. Up to 50 tags are allowed. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a CompositeAlarm resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CompositeAlarmArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CompositeAlarm resources.
 */
export interface CompositeAlarmState {
    /**
     * Indicates whether actions should be executed during any changes to the alarm state of the composite alarm. Defaults to `true`.
     */
    actionsEnabled?: pulumi.Input<boolean>;
    /**
     * Actions will be suppressed if the suppressor alarm is in the ALARM state.
     */
    actionsSuppressor?: pulumi.Input<inputs.cloudwatch.CompositeAlarmActionsSuppressor>;
    /**
     * The set of actions to execute when this alarm transitions to the `ALARM` state from any other state. Each action is specified as an ARN. Up to 5 actions are allowed.
     */
    alarmActions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The description for the composite alarm.
     */
    alarmDescription?: pulumi.Input<string>;
    /**
     * The name for the composite alarm. This name must be unique within the region.
     */
    alarmName?: pulumi.Input<string>;
    /**
     * An expression that specifies which other alarms are to be evaluated to determine this composite alarm's state. For syntax, see [Creating a Composite Alarm](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/Create_Composite_Alarm.html). The maximum length is 10240 characters.
     */
    alarmRule?: pulumi.Input<string>;
    /**
     * The ARN of the composite alarm.
     */
    arn?: pulumi.Input<string>;
    /**
     * The set of actions to execute when this alarm transitions to the `INSUFFICIENT_DATA` state from any other state. Each action is specified as an ARN. Up to 5 actions are allowed.
     */
    insufficientDataActions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The set of actions to execute when this alarm transitions to an `OK` state from any other state. Each action is specified as an ARN. Up to 5 actions are allowed.
     */
    okActions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to associate with the alarm. Up to 50 tags are allowed. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a CompositeAlarm resource.
 */
export interface CompositeAlarmArgs {
    /**
     * Indicates whether actions should be executed during any changes to the alarm state of the composite alarm. Defaults to `true`.
     */
    actionsEnabled?: pulumi.Input<boolean>;
    /**
     * Actions will be suppressed if the suppressor alarm is in the ALARM state.
     */
    actionsSuppressor?: pulumi.Input<inputs.cloudwatch.CompositeAlarmActionsSuppressor>;
    /**
     * The set of actions to execute when this alarm transitions to the `ALARM` state from any other state. Each action is specified as an ARN. Up to 5 actions are allowed.
     */
    alarmActions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The description for the composite alarm.
     */
    alarmDescription?: pulumi.Input<string>;
    /**
     * The name for the composite alarm. This name must be unique within the region.
     */
    alarmName: pulumi.Input<string>;
    /**
     * An expression that specifies which other alarms are to be evaluated to determine this composite alarm's state. For syntax, see [Creating a Composite Alarm](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/Create_Composite_Alarm.html). The maximum length is 10240 characters.
     */
    alarmRule: pulumi.Input<string>;
    /**
     * The set of actions to execute when this alarm transitions to the `INSUFFICIENT_DATA` state from any other state. Each action is specified as an ARN. Up to 5 actions are allowed.
     */
    insufficientDataActions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The set of actions to execute when this alarm transitions to an `OK` state from any other state. Each action is specified as an ARN. Up to 5 actions are allowed.
     */
    okActions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to associate with the alarm. Up to 50 tags are allowed. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
