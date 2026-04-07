import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an SSM Maintenance Window Task resource
 *
 * ## Example Usage
 *
 * ### Automation Tasks
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ssm.MaintenanceWindowTask("example", {
 *     maxConcurrency: "2",
 *     maxErrors: "1",
 *     priority: 1,
 *     taskArn: "AWS-RestartEC2Instance",
 *     taskType: "AUTOMATION",
 *     windowId: exampleAwsSsmMaintenanceWindow.id,
 *     targets: [{
 *         key: "InstanceIds",
 *         values: [exampleAwsInstance.id],
 *     }],
 *     taskInvocationParameters: {
 *         automationParameters: {
 *             documentVersion: "$LATEST",
 *             parameters: [{
 *                 name: "InstanceId",
 *                 values: [exampleAwsInstance.id],
 *             }],
 *         },
 *     },
 * });
 * ```
 *
 * ### Lambda Tasks
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.ssm.MaintenanceWindowTask("example", {
 *     maxConcurrency: "2",
 *     maxErrors: "1",
 *     priority: 1,
 *     taskArn: exampleAwsLambdaFunction.arn,
 *     taskType: "LAMBDA",
 *     windowId: exampleAwsSsmMaintenanceWindow.id,
 *     targets: [{
 *         key: "InstanceIds",
 *         values: [exampleAwsInstance.id],
 *     }],
 *     taskInvocationParameters: {
 *         lambdaParameters: {
 *             clientContext: std.base64encode({
 *                 input: "{\"key1\":\"value1\"}",
 *             }).then(invoke => invoke.result),
 *             payload: "{\"key1\":\"value1\"}",
 *         },
 *     },
 * });
 * ```
 *
 * ### Run Command Tasks
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ssm.MaintenanceWindowTask("example", {
 *     maxConcurrency: "2",
 *     maxErrors: "1",
 *     priority: 1,
 *     taskArn: "AWS-RunShellScript",
 *     taskType: "RUN_COMMAND",
 *     windowId: exampleAwsSsmMaintenanceWindow.id,
 *     targets: [{
 *         key: "InstanceIds",
 *         values: [exampleAwsInstance.id],
 *     }],
 *     taskInvocationParameters: {
 *         runCommandParameters: {
 *             outputS3Bucket: exampleAwsS3Bucket.id,
 *             outputS3KeyPrefix: "output",
 *             serviceRoleArn: exampleAwsIamRole.arn,
 *             timeoutSeconds: 600,
 *             notificationConfig: {
 *                 notificationArn: exampleAwsSnsTopic.arn,
 *                 notificationEvents: ["All"],
 *                 notificationType: "Command",
 *             },
 *             parameters: [{
 *                 name: "commands",
 *                 values: ["date"],
 *             }],
 *         },
 *     },
 * });
 * ```
 *
 * ### Step Function Tasks
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ssm.MaintenanceWindowTask("example", {
 *     maxConcurrency: "2",
 *     maxErrors: "1",
 *     priority: 1,
 *     taskArn: exampleAwsSfnActivity.id,
 *     taskType: "STEP_FUNCTIONS",
 *     windowId: exampleAwsSsmMaintenanceWindow.id,
 *     targets: [{
 *         key: "InstanceIds",
 *         values: [exampleAwsInstance.id],
 *     }],
 *     taskInvocationParameters: {
 *         stepFunctionsParameters: {
 *             input: "{\"key1\":\"value1\"}",
 *             name: "example",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `window_id` - (String) ID of the maintenance window.
 *
 * * `id` - (String) ID of the maintenance window task.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * * `region` (String) Region where this resource is managed.
 *
 * Using `pulumi import`, import AWS Maintenance Window Task using the `window_id` and `window_task_id` separated by `/`. For example:
 *
 * console
 *
 * % pulumi import aws_ssm_maintenance_window_task.example <window_id>/<window_task_id>
 */
export declare class MaintenanceWindowTask extends pulumi.CustomResource {
    /**
     * Get an existing MaintenanceWindowTask resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MaintenanceWindowTaskState, opts?: pulumi.CustomResourceOptions): MaintenanceWindowTask;
    /**
     * Returns true if the given object is an instance of MaintenanceWindowTask.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is MaintenanceWindowTask;
    /**
     * The ARN of the maintenance window task.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Indicates whether tasks should continue to run after the cutoff time specified in the maintenance windows is reached. Valid values are `CONTINUE_TASK` and `CANCEL_TASK`.
     */
    readonly cutoffBehavior: pulumi.Output<string | undefined>;
    /**
     * The description of the maintenance window task.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The maximum number of targets this task can be run for in parallel.
     */
    readonly maxConcurrency: pulumi.Output<string>;
    /**
     * The maximum number of errors allowed before this task stops being scheduled.
     */
    readonly maxErrors: pulumi.Output<string>;
    /**
     * The name of the maintenance window task.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The priority of the task in the Maintenance Window, the lower the number the higher the priority. Tasks in a Maintenance Window are scheduled in priority order with tasks that have the same priority scheduled in parallel.
     */
    readonly priority: pulumi.Output<number | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The role that should be assumed when executing the task. If a role is not provided, Systems Manager uses your account's service-linked role. If no service-linked role for Systems Manager exists in your account, it is created for you.
     */
    readonly serviceRoleArn: pulumi.Output<string>;
    /**
     * The targets (either instances or window target ids). Instances are specified using Key=InstanceIds,Values=instanceid1,instanceid2. Window target ids are specified using Key=WindowTargetIds,Values=window target id1, window target id2.
     */
    readonly targets: pulumi.Output<outputs.ssm.MaintenanceWindowTaskTarget[] | undefined>;
    /**
     * The ARN of the task to execute.
     */
    readonly taskArn: pulumi.Output<string>;
    /**
     * Configuration block with parameters for task execution.
     */
    readonly taskInvocationParameters: pulumi.Output<outputs.ssm.MaintenanceWindowTaskTaskInvocationParameters | undefined>;
    /**
     * The type of task being registered. Valid values: `AUTOMATION`, `LAMBDA`, `RUN_COMMAND` or `STEP_FUNCTIONS`.
     */
    readonly taskType: pulumi.Output<string>;
    /**
     * The Id of the maintenance window to register the task with.
     */
    readonly windowId: pulumi.Output<string>;
    /**
     * The ID of the maintenance window task.
     */
    readonly windowTaskId: pulumi.Output<string>;
    /**
     * Create a MaintenanceWindowTask resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MaintenanceWindowTaskArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering MaintenanceWindowTask resources.
 */
export interface MaintenanceWindowTaskState {
    /**
     * The ARN of the maintenance window task.
     */
    arn?: pulumi.Input<string>;
    /**
     * Indicates whether tasks should continue to run after the cutoff time specified in the maintenance windows is reached. Valid values are `CONTINUE_TASK` and `CANCEL_TASK`.
     */
    cutoffBehavior?: pulumi.Input<string>;
    /**
     * The description of the maintenance window task.
     */
    description?: pulumi.Input<string>;
    /**
     * The maximum number of targets this task can be run for in parallel.
     */
    maxConcurrency?: pulumi.Input<string>;
    /**
     * The maximum number of errors allowed before this task stops being scheduled.
     */
    maxErrors?: pulumi.Input<string>;
    /**
     * The name of the maintenance window task.
     */
    name?: pulumi.Input<string>;
    /**
     * The priority of the task in the Maintenance Window, the lower the number the higher the priority. Tasks in a Maintenance Window are scheduled in priority order with tasks that have the same priority scheduled in parallel.
     */
    priority?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The role that should be assumed when executing the task. If a role is not provided, Systems Manager uses your account's service-linked role. If no service-linked role for Systems Manager exists in your account, it is created for you.
     */
    serviceRoleArn?: pulumi.Input<string>;
    /**
     * The targets (either instances or window target ids). Instances are specified using Key=InstanceIds,Values=instanceid1,instanceid2. Window target ids are specified using Key=WindowTargetIds,Values=window target id1, window target id2.
     */
    targets?: pulumi.Input<pulumi.Input<inputs.ssm.MaintenanceWindowTaskTarget>[]>;
    /**
     * The ARN of the task to execute.
     */
    taskArn?: pulumi.Input<string>;
    /**
     * Configuration block with parameters for task execution.
     */
    taskInvocationParameters?: pulumi.Input<inputs.ssm.MaintenanceWindowTaskTaskInvocationParameters>;
    /**
     * The type of task being registered. Valid values: `AUTOMATION`, `LAMBDA`, `RUN_COMMAND` or `STEP_FUNCTIONS`.
     */
    taskType?: pulumi.Input<string>;
    /**
     * The Id of the maintenance window to register the task with.
     */
    windowId?: pulumi.Input<string>;
    /**
     * The ID of the maintenance window task.
     */
    windowTaskId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a MaintenanceWindowTask resource.
 */
export interface MaintenanceWindowTaskArgs {
    /**
     * Indicates whether tasks should continue to run after the cutoff time specified in the maintenance windows is reached. Valid values are `CONTINUE_TASK` and `CANCEL_TASK`.
     */
    cutoffBehavior?: pulumi.Input<string>;
    /**
     * The description of the maintenance window task.
     */
    description?: pulumi.Input<string>;
    /**
     * The maximum number of targets this task can be run for in parallel.
     */
    maxConcurrency?: pulumi.Input<string>;
    /**
     * The maximum number of errors allowed before this task stops being scheduled.
     */
    maxErrors?: pulumi.Input<string>;
    /**
     * The name of the maintenance window task.
     */
    name?: pulumi.Input<string>;
    /**
     * The priority of the task in the Maintenance Window, the lower the number the higher the priority. Tasks in a Maintenance Window are scheduled in priority order with tasks that have the same priority scheduled in parallel.
     */
    priority?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The role that should be assumed when executing the task. If a role is not provided, Systems Manager uses your account's service-linked role. If no service-linked role for Systems Manager exists in your account, it is created for you.
     */
    serviceRoleArn?: pulumi.Input<string>;
    /**
     * The targets (either instances or window target ids). Instances are specified using Key=InstanceIds,Values=instanceid1,instanceid2. Window target ids are specified using Key=WindowTargetIds,Values=window target id1, window target id2.
     */
    targets?: pulumi.Input<pulumi.Input<inputs.ssm.MaintenanceWindowTaskTarget>[]>;
    /**
     * The ARN of the task to execute.
     */
    taskArn: pulumi.Input<string>;
    /**
     * Configuration block with parameters for task execution.
     */
    taskInvocationParameters?: pulumi.Input<inputs.ssm.MaintenanceWindowTaskTaskInvocationParameters>;
    /**
     * The type of task being registered. Valid values: `AUTOMATION`, `LAMBDA`, `RUN_COMMAND` or `STEP_FUNCTIONS`.
     */
    taskType: pulumi.Input<string>;
    /**
     * The Id of the maintenance window to register the task with.
     */
    windowId: pulumi.Input<string>;
}
