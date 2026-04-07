import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a DMS Serverless replication config resource.
 *
 * > **NOTE:** Changing most arguments will stop the replication if it is running. You can set `startReplication` to resume the replication afterwards.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const name = new aws.dms.ReplicationConfig("name", {
 *     replicationConfigIdentifier: "test-dms-serverless-replication-tf",
 *     resourceIdentifier: "test-dms-serverless-replication-tf",
 *     replicationType: "cdc",
 *     sourceEndpointArn: source.endpointArn,
 *     targetEndpointArn: target.endpointArn,
 *     tableMappings: `  {
 *     \\"rules\\":[{\\"rule-type\\":\\"selection\\",\\"rule-id\\":\\"1\\",\\"rule-name\\":\\"1\\",\\"rule-action\\":\\"include\\",\\"object-locator\\":{\\"schema-name\\":\\"%%\\",\\"table-name\\":\\"%%\\"}}]
 *   }
 * `,
 *     startReplication: true,
 *     computeConfig: {
 *         replicationSubnetGroupId: _default.replicationSubnetGroupId,
 *         maxCapacityUnits: 64,
 *         minCapacityUnits: 2,
 *         preferredMaintenanceWindow: "sun:23:45-mon:00:30",
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
 * - `arn` (String) Amazon Resource Name (ARN) of the DMS replication configuration.
 *
 * Using `pulumi import`, import a replication config using the `arn`. For example:
 *
 * console
 *
 * % pulumi import aws_dms_replication_config.example arn:aws:dms:us-east-1:123456789012:replication-config:UX6OL6MHMMJKFFOXE3H7LLJCMEKBDUG4ZV7DRSI
 */
export declare class ReplicationConfig extends pulumi.CustomResource {
    /**
     * Get an existing ReplicationConfig resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ReplicationConfigState, opts?: pulumi.CustomResourceOptions): ReplicationConfig;
    /**
     * Returns true if the given object is an instance of ReplicationConfig.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ReplicationConfig;
    /**
     * The Amazon Resource Name (ARN) for the serverless replication config.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Configuration block for provisioning an DMS Serverless replication.
     */
    readonly computeConfig: pulumi.Output<outputs.dms.ReplicationConfigComputeConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Unique identifier that you want to use to create the config.
     */
    readonly replicationConfigIdentifier: pulumi.Output<string>;
    /**
     * An escaped JSON string that are used to provision this replication configuration. For example, [Change processing tuning settings](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.CustomizingTasks.TaskSettings.ChangeProcessingTuning.html)
     */
    readonly replicationSettings: pulumi.Output<string>;
    /**
     * The migration type. Can be one of `full-load | cdc | full-load-and-cdc`.
     */
    readonly replicationType: pulumi.Output<string>;
    /**
     * Unique value or name that you set for a given resource that can be used to construct an Amazon Resource Name (ARN) for that resource. For more information, see [Fine-grained access control using resource names and tags](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Security.html#CHAP_Security.FineGrainedAccess)
     */
    readonly resourceIdentifier: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) string that uniquely identifies the source endpoint.
     */
    readonly sourceEndpointArn: pulumi.Output<string>;
    /**
     * Whether to run or stop the serverless replication, default is false.
     */
    readonly startReplication: pulumi.Output<boolean | undefined>;
    /**
     * JSON settings for specifying supplemental data. For more information see [Specifying supplemental data for task settings](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.TaskData.html)
     */
    readonly supplementalSettings: pulumi.Output<string | undefined>;
    /**
     * An escaped JSON string that contains the table mappings. For information on table mapping see [Using Table Mapping with an AWS Database Migration Service Task to Select and Filter Data](http://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.CustomizingTasks.TableMapping.html)
     */
    readonly tableMappings: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The Amazon Resource Name (ARN) string that uniquely identifies the target endpoint.
     */
    readonly targetEndpointArn: pulumi.Output<string>;
    /**
     * Create a ReplicationConfig resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ReplicationConfigArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ReplicationConfig resources.
 */
export interface ReplicationConfigState {
    /**
     * The Amazon Resource Name (ARN) for the serverless replication config.
     */
    arn?: pulumi.Input<string>;
    /**
     * Configuration block for provisioning an DMS Serverless replication.
     */
    computeConfig?: pulumi.Input<inputs.dms.ReplicationConfigComputeConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Unique identifier that you want to use to create the config.
     */
    replicationConfigIdentifier?: pulumi.Input<string>;
    /**
     * An escaped JSON string that are used to provision this replication configuration. For example, [Change processing tuning settings](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.CustomizingTasks.TaskSettings.ChangeProcessingTuning.html)
     */
    replicationSettings?: pulumi.Input<string>;
    /**
     * The migration type. Can be one of `full-load | cdc | full-load-and-cdc`.
     */
    replicationType?: pulumi.Input<string>;
    /**
     * Unique value or name that you set for a given resource that can be used to construct an Amazon Resource Name (ARN) for that resource. For more information, see [Fine-grained access control using resource names and tags](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Security.html#CHAP_Security.FineGrainedAccess)
     */
    resourceIdentifier?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) string that uniquely identifies the source endpoint.
     */
    sourceEndpointArn?: pulumi.Input<string>;
    /**
     * Whether to run or stop the serverless replication, default is false.
     */
    startReplication?: pulumi.Input<boolean>;
    /**
     * JSON settings for specifying supplemental data. For more information see [Specifying supplemental data for task settings](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.TaskData.html)
     */
    supplementalSettings?: pulumi.Input<string>;
    /**
     * An escaped JSON string that contains the table mappings. For information on table mapping see [Using Table Mapping with an AWS Database Migration Service Task to Select and Filter Data](http://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.CustomizingTasks.TableMapping.html)
     */
    tableMappings?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The Amazon Resource Name (ARN) string that uniquely identifies the target endpoint.
     */
    targetEndpointArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ReplicationConfig resource.
 */
export interface ReplicationConfigArgs {
    /**
     * Configuration block for provisioning an DMS Serverless replication.
     */
    computeConfig: pulumi.Input<inputs.dms.ReplicationConfigComputeConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Unique identifier that you want to use to create the config.
     */
    replicationConfigIdentifier: pulumi.Input<string>;
    /**
     * An escaped JSON string that are used to provision this replication configuration. For example, [Change processing tuning settings](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.CustomizingTasks.TaskSettings.ChangeProcessingTuning.html)
     */
    replicationSettings?: pulumi.Input<string>;
    /**
     * The migration type. Can be one of `full-load | cdc | full-load-and-cdc`.
     */
    replicationType: pulumi.Input<string>;
    /**
     * Unique value or name that you set for a given resource that can be used to construct an Amazon Resource Name (ARN) for that resource. For more information, see [Fine-grained access control using resource names and tags](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Security.html#CHAP_Security.FineGrainedAccess)
     */
    resourceIdentifier?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) string that uniquely identifies the source endpoint.
     */
    sourceEndpointArn: pulumi.Input<string>;
    /**
     * Whether to run or stop the serverless replication, default is false.
     */
    startReplication?: pulumi.Input<boolean>;
    /**
     * JSON settings for specifying supplemental data. For more information see [Specifying supplemental data for task settings](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.TaskData.html)
     */
    supplementalSettings?: pulumi.Input<string>;
    /**
     * An escaped JSON string that contains the table mappings. For information on table mapping see [Using Table Mapping with an AWS Database Migration Service Task to Select and Filter Data](http://docs.aws.amazon.com/dms/latest/userguide/CHAP_Tasks.CustomizingTasks.TableMapping.html)
     */
    tableMappings: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The Amazon Resource Name (ARN) string that uniquely identifies the target endpoint.
     */
    targetEndpointArn: pulumi.Input<string>;
}
