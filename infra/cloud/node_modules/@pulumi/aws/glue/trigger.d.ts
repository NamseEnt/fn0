import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Glue Trigger resource.
 *
 * ## Example Usage
 *
 * ### Conditional Trigger
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Trigger("example", {
 *     name: "example",
 *     type: "CONDITIONAL",
 *     actions: [{
 *         jobName: example1.name,
 *     }],
 *     predicate: {
 *         conditions: [{
 *             jobName: example2.name,
 *             state: "SUCCEEDED",
 *         }],
 *     },
 * });
 * ```
 *
 * ### On-Demand Trigger
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Trigger("example", {
 *     name: "example",
 *     type: "ON_DEMAND",
 *     actions: [{
 *         jobName: exampleAwsGlueJob.name,
 *     }],
 * });
 * ```
 *
 * ### Scheduled Trigger
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Trigger("example", {
 *     name: "example",
 *     schedule: "cron(15 12 * * ? *)",
 *     type: "SCHEDULED",
 *     actions: [{
 *         jobName: exampleAwsGlueJob.name,
 *     }],
 * });
 * ```
 *
 * ### Conditional Trigger with Crawler Action
 *
 * **Note:** Triggers can have both a crawler action and a crawler condition, just no example provided.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Trigger("example", {
 *     name: "example",
 *     type: "CONDITIONAL",
 *     actions: [{
 *         crawlerName: example1.name,
 *     }],
 *     predicate: {
 *         conditions: [{
 *             jobName: example2.name,
 *             state: "SUCCEEDED",
 *         }],
 *     },
 * });
 * ```
 *
 * ### Conditional Trigger with Crawler Condition
 *
 * **Note:** Triggers can have both a crawler action and a crawler condition, just no example provided.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Trigger("example", {
 *     name: "example",
 *     type: "CONDITIONAL",
 *     actions: [{
 *         jobName: example1.name,
 *     }],
 *     predicate: {
 *         conditions: [{
 *             crawlerName: example2.name,
 *             crawlState: "SUCCEEDED",
 *         }],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Glue Triggers using `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:glue/trigger:Trigger MyTrigger MyTrigger
 * ```
 */
export declare class Trigger extends pulumi.CustomResource {
    /**
     * Get an existing Trigger resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TriggerState, opts?: pulumi.CustomResourceOptions): Trigger;
    /**
     * Returns true if the given object is an instance of Trigger.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Trigger;
    /**
     * List of actions initiated by this trigger when it fires. See Actions Below.
     */
    readonly actions: pulumi.Output<outputs.glue.TriggerAction[]>;
    /**
     * Amazon Resource Name (ARN) of Glue Trigger
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A description of the new trigger.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Start the trigger. Defaults to `true`.
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * Batch condition that must be met (specified number of events received or batch time window expired) before EventBridge event trigger fires. See Event Batching Condition.
     */
    readonly eventBatchingConditions: pulumi.Output<outputs.glue.TriggerEventBatchingCondition[] | undefined>;
    /**
     * The name of the trigger.
     */
    readonly name: pulumi.Output<string>;
    /**
     * A predicate to specify when the new trigger should fire. Required when trigger type is `CONDITIONAL`. See Predicate Below.
     */
    readonly predicate: pulumi.Output<outputs.glue.TriggerPredicate | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A cron expression used to specify the schedule. [Time-Based Schedules for Jobs and Crawlers](https://docs.aws.amazon.com/glue/latest/dg/monitor-data-warehouse-schedule.html)
     */
    readonly schedule: pulumi.Output<string | undefined>;
    /**
     * Set to true to start `SCHEDULED` and `CONDITIONAL` triggers when created. True is not supported for `ON_DEMAND` triggers.
     */
    readonly startOnCreation: pulumi.Output<boolean | undefined>;
    /**
     * The current state of the trigger.
     */
    readonly state: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The type of trigger. Valid values are `CONDITIONAL`, `EVENT`, `ON_DEMAND`, and `SCHEDULED`.
     */
    readonly type: pulumi.Output<string>;
    /**
     * A workflow to which the trigger should be associated to. Every workflow graph (DAG) needs a starting trigger (`ON_DEMAND` or `SCHEDULED` type) and can contain multiple additional `CONDITIONAL` triggers.
     */
    readonly workflowName: pulumi.Output<string | undefined>;
    /**
     * Create a Trigger resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TriggerArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Trigger resources.
 */
export interface TriggerState {
    /**
     * List of actions initiated by this trigger when it fires. See Actions Below.
     */
    actions?: pulumi.Input<pulumi.Input<inputs.glue.TriggerAction>[]>;
    /**
     * Amazon Resource Name (ARN) of Glue Trigger
     */
    arn?: pulumi.Input<string>;
    /**
     * A description of the new trigger.
     */
    description?: pulumi.Input<string>;
    /**
     * Start the trigger. Defaults to `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Batch condition that must be met (specified number of events received or batch time window expired) before EventBridge event trigger fires. See Event Batching Condition.
     */
    eventBatchingConditions?: pulumi.Input<pulumi.Input<inputs.glue.TriggerEventBatchingCondition>[]>;
    /**
     * The name of the trigger.
     */
    name?: pulumi.Input<string>;
    /**
     * A predicate to specify when the new trigger should fire. Required when trigger type is `CONDITIONAL`. See Predicate Below.
     */
    predicate?: pulumi.Input<inputs.glue.TriggerPredicate>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A cron expression used to specify the schedule. [Time-Based Schedules for Jobs and Crawlers](https://docs.aws.amazon.com/glue/latest/dg/monitor-data-warehouse-schedule.html)
     */
    schedule?: pulumi.Input<string>;
    /**
     * Set to true to start `SCHEDULED` and `CONDITIONAL` triggers when created. True is not supported for `ON_DEMAND` triggers.
     */
    startOnCreation?: pulumi.Input<boolean>;
    /**
     * The current state of the trigger.
     */
    state?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The type of trigger. Valid values are `CONDITIONAL`, `EVENT`, `ON_DEMAND`, and `SCHEDULED`.
     */
    type?: pulumi.Input<string>;
    /**
     * A workflow to which the trigger should be associated to. Every workflow graph (DAG) needs a starting trigger (`ON_DEMAND` or `SCHEDULED` type) and can contain multiple additional `CONDITIONAL` triggers.
     */
    workflowName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Trigger resource.
 */
export interface TriggerArgs {
    /**
     * List of actions initiated by this trigger when it fires. See Actions Below.
     */
    actions: pulumi.Input<pulumi.Input<inputs.glue.TriggerAction>[]>;
    /**
     * A description of the new trigger.
     */
    description?: pulumi.Input<string>;
    /**
     * Start the trigger. Defaults to `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Batch condition that must be met (specified number of events received or batch time window expired) before EventBridge event trigger fires. See Event Batching Condition.
     */
    eventBatchingConditions?: pulumi.Input<pulumi.Input<inputs.glue.TriggerEventBatchingCondition>[]>;
    /**
     * The name of the trigger.
     */
    name?: pulumi.Input<string>;
    /**
     * A predicate to specify when the new trigger should fire. Required when trigger type is `CONDITIONAL`. See Predicate Below.
     */
    predicate?: pulumi.Input<inputs.glue.TriggerPredicate>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A cron expression used to specify the schedule. [Time-Based Schedules for Jobs and Crawlers](https://docs.aws.amazon.com/glue/latest/dg/monitor-data-warehouse-schedule.html)
     */
    schedule?: pulumi.Input<string>;
    /**
     * Set to true to start `SCHEDULED` and `CONDITIONAL` triggers when created. True is not supported for `ON_DEMAND` triggers.
     */
    startOnCreation?: pulumi.Input<boolean>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The type of trigger. Valid values are `CONDITIONAL`, `EVENT`, `ON_DEMAND`, and `SCHEDULED`.
     */
    type: pulumi.Input<string>;
    /**
     * A workflow to which the trigger should be associated to. Every workflow graph (DAG) needs a starting trigger (`ON_DEMAND` or `SCHEDULED` type) and can contain multiple additional `CONDITIONAL` triggers.
     */
    workflowName?: pulumi.Input<string>;
}
