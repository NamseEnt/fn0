import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a DMS (Data Migration Service) replication task resource. DMS replication tasks can be created, updated, deleted, and imported.
 *
 * > **NOTE:** Changing most arguments will stop the task if it is running. You can set `startReplicationTask` to resume the task afterwards.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * // Create a new replication task
 * const test = new aws.dms.ReplicationTask("test", {
 *     cdcStartTime: "1993-05-21T05:50:00Z",
 *     migrationType: "full-load",
 *     replicationInstanceArn: test_dms_replication_instance_tf.replicationInstanceArn,
 *     replicationTaskId: "test-dms-replication-task-tf",
 *     replicationTaskSettings: "...",
 *     sourceEndpointArn: test_dms_source_endpoint_tf.endpointArn,
 *     tableMappings: "{\"rules\":[{\"rule-type\":\"selection\",\"rule-id\":\"1\",\"rule-name\":\"1\",\"object-locator\":{\"schema-name\":\"%\",\"table-name\":\"%\"},\"rule-action\":\"include\"}]}",
 *     tags: {
 *         Name: "test",
 *     },
 *     targetEndpointArn: test_dms_target_endpoint_tf.endpointArn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import replication tasks using the `replication_task_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:dms/replicationTask:ReplicationTask test test-dms-replication-task-tf
 * ```
 */
export declare class ReplicationTask extends pulumi.CustomResource {
    /**
     * Get an existing ReplicationTask resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ReplicationTaskState, opts?: pulumi.CustomResourceOptions): ReplicationTask;
    /**
     * Returns true if the given object is an instance of ReplicationTask.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ReplicationTask;
    /**
     * Indicates when you want a change data capture (CDC) operation to start. The value can be a RFC3339 formatted date, a checkpoint, or a LSN/SCN format depending on the source engine. For more information see [Determining a CDC native start point](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Task.CDC.html#CHAP_Task.CDC.StartPoint.Native).
     */
    readonly cdcStartPosition: pulumi.Output<string>;
    /**
     * RFC3339 formatted date string or UNIX timestamp for the start of the Change Data Capture (CDC) operation.
     */
    readonly cdcStartTime: pulumi.Output<string | undefined>;
    /**
     * Migration type. Can be one of `full-load | cdc | full-load-and-cdc`.
     */
    readonly migrationType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the replication instance.
     */
    readonly replicationInstanceArn: pulumi.Output<string>;
    /**
     * ARN for the replication task.
     */
    readonly replicationTaskArn: pulumi.Output<string>;
    /**
     * Replication task identifier which must contain from 1 to 255 alphanumeric characters or hyphens, first character must be a letter, cannot end with a hyphen, and cannot contain two consecutive hyphens.
     */
    readonly replicationTaskId: pulumi.Output<string>;
    /**
     * Escaped JSON string that contains the task settings. For a complete list of task settings, see [Task Settings for AWS Database Migration Service Tasks](http://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.CustomizingTasks.TaskSettings.html). Note that `Logging.CloudWatchLogGroup` and `Logging.CloudWatchLogStream` are read only and should not be defined, even as `null`, in the configuration since AWS provides a value for these settings.
     */
    readonly replicationTaskSettings: pulumi.Output<string>;
    /**
     * A friendly name for the resource identifier at the end of the EndpointArn response parameter that is returned in the created Endpoint object.
     */
    readonly resourceIdentifier: pulumi.Output<string | undefined>;
    /**
     * ARN that uniquely identifies the source endpoint.
     */
    readonly sourceEndpointArn: pulumi.Output<string>;
    /**
     * Whether to run or stop the replication task.
     */
    readonly startReplicationTask: pulumi.Output<boolean | undefined>;
    /**
     * Replication Task status.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Escaped JSON string that contains the table mappings. For information on table mapping see [Using Table Mapping with an AWS Database Migration Service Task to Select and Filter Data](http://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.CustomizingTasks.TableMapping.html)
     */
    readonly tableMappings: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * ARN that uniquely identifies the target endpoint.
     */
    readonly targetEndpointArn: pulumi.Output<string>;
    /**
     * Create a ReplicationTask resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ReplicationTaskArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ReplicationTask resources.
 */
export interface ReplicationTaskState {
    /**
     * Indicates when you want a change data capture (CDC) operation to start. The value can be a RFC3339 formatted date, a checkpoint, or a LSN/SCN format depending on the source engine. For more information see [Determining a CDC native start point](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Task.CDC.html#CHAP_Task.CDC.StartPoint.Native).
     */
    cdcStartPosition?: pulumi.Input<string>;
    /**
     * RFC3339 formatted date string or UNIX timestamp for the start of the Change Data Capture (CDC) operation.
     */
    cdcStartTime?: pulumi.Input<string>;
    /**
     * Migration type. Can be one of `full-load | cdc | full-load-and-cdc`.
     */
    migrationType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the replication instance.
     */
    replicationInstanceArn?: pulumi.Input<string>;
    /**
     * ARN for the replication task.
     */
    replicationTaskArn?: pulumi.Input<string>;
    /**
     * Replication task identifier which must contain from 1 to 255 alphanumeric characters or hyphens, first character must be a letter, cannot end with a hyphen, and cannot contain two consecutive hyphens.
     */
    replicationTaskId?: pulumi.Input<string>;
    /**
     * Escaped JSON string that contains the task settings. For a complete list of task settings, see [Task Settings for AWS Database Migration Service Tasks](http://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.CustomizingTasks.TaskSettings.html). Note that `Logging.CloudWatchLogGroup` and `Logging.CloudWatchLogStream` are read only and should not be defined, even as `null`, in the configuration since AWS provides a value for these settings.
     */
    replicationTaskSettings?: pulumi.Input<string>;
    /**
     * A friendly name for the resource identifier at the end of the EndpointArn response parameter that is returned in the created Endpoint object.
     */
    resourceIdentifier?: pulumi.Input<string>;
    /**
     * ARN that uniquely identifies the source endpoint.
     */
    sourceEndpointArn?: pulumi.Input<string>;
    /**
     * Whether to run or stop the replication task.
     */
    startReplicationTask?: pulumi.Input<boolean>;
    /**
     * Replication Task status.
     */
    status?: pulumi.Input<string>;
    /**
     * Escaped JSON string that contains the table mappings. For information on table mapping see [Using Table Mapping with an AWS Database Migration Service Task to Select and Filter Data](http://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.CustomizingTasks.TableMapping.html)
     */
    tableMappings?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ARN that uniquely identifies the target endpoint.
     */
    targetEndpointArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ReplicationTask resource.
 */
export interface ReplicationTaskArgs {
    /**
     * Indicates when you want a change data capture (CDC) operation to start. The value can be a RFC3339 formatted date, a checkpoint, or a LSN/SCN format depending on the source engine. For more information see [Determining a CDC native start point](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Task.CDC.html#CHAP_Task.CDC.StartPoint.Native).
     */
    cdcStartPosition?: pulumi.Input<string>;
    /**
     * RFC3339 formatted date string or UNIX timestamp for the start of the Change Data Capture (CDC) operation.
     */
    cdcStartTime?: pulumi.Input<string>;
    /**
     * Migration type. Can be one of `full-load | cdc | full-load-and-cdc`.
     */
    migrationType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the replication instance.
     */
    replicationInstanceArn: pulumi.Input<string>;
    /**
     * Replication task identifier which must contain from 1 to 255 alphanumeric characters or hyphens, first character must be a letter, cannot end with a hyphen, and cannot contain two consecutive hyphens.
     */
    replicationTaskId: pulumi.Input<string>;
    /**
     * Escaped JSON string that contains the task settings. For a complete list of task settings, see [Task Settings for AWS Database Migration Service Tasks](http://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.CustomizingTasks.TaskSettings.html). Note that `Logging.CloudWatchLogGroup` and `Logging.CloudWatchLogStream` are read only and should not be defined, even as `null`, in the configuration since AWS provides a value for these settings.
     */
    replicationTaskSettings?: pulumi.Input<string>;
    /**
     * A friendly name for the resource identifier at the end of the EndpointArn response parameter that is returned in the created Endpoint object.
     */
    resourceIdentifier?: pulumi.Input<string>;
    /**
     * ARN that uniquely identifies the source endpoint.
     */
    sourceEndpointArn: pulumi.Input<string>;
    /**
     * Whether to run or stop the replication task.
     */
    startReplicationTask?: pulumi.Input<boolean>;
    /**
     * Escaped JSON string that contains the table mappings. For information on table mapping see [Using Table Mapping with an AWS Database Migration Service Task to Select and Filter Data](http://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.CustomizingTasks.TableMapping.html)
     */
    tableMappings: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ARN that uniquely identifies the target endpoint.
     */
    targetEndpointArn: pulumi.Input<string>;
}
