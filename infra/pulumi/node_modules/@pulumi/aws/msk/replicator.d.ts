import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Managed Streaming for Kafka Replicator.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.msk.Replicator("test", {
 *     replicatorName: "test-name",
 *     description: "test-description",
 *     serviceExecutionRoleArn: sourceAwsIamRole.arn,
 *     kafkaClusters: [
 *         {
 *             amazonMskCluster: {
 *                 mskClusterArn: source.arn,
 *             },
 *             vpcConfig: {
 *                 subnetIds: sourceAwsSubnet.map(__item => __item.id),
 *                 securityGroupsIds: [sourceAwsSecurityGroup.id],
 *             },
 *         },
 *         {
 *             amazonMskCluster: {
 *                 mskClusterArn: target.arn,
 *             },
 *             vpcConfig: {
 *                 subnetIds: targetAwsSubnet.map(__item => __item.id),
 *                 securityGroupsIds: [targetAwsSecurityGroup.id],
 *             },
 *         },
 *     ],
 *     replicationInfoList: {
 *         sourceKafkaClusterArn: source.arn,
 *         targetKafkaClusterArn: target.arn,
 *         targetCompressionType: "NONE",
 *         topicReplications: [{
 *             topicNameConfiguration: {
 *                 type: "PREFIXED_WITH_SOURCE_CLUSTER_ALIAS",
 *             },
 *             topicsToReplicates: [".*"],
 *             startingPosition: {
 *                 type: "LATEST",
 *             },
 *         }],
 *         consumerGroupReplications: [{
 *             consumerGroupsToReplicates: [".*"],
 *         }],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import MSK replicators using the replicator ARN. For example:
 *
 * ```sh
 * $ pulumi import aws:msk/replicator:Replicator example arn:aws:kafka:us-west-2:123456789012:configuration/example/279c0212-d057-4dba-9aa9-1c4e5a25bfc7-3
 * ```
 */
export declare class Replicator extends pulumi.CustomResource {
    /**
     * Get an existing Replicator resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ReplicatorState, opts?: pulumi.CustomResourceOptions): Replicator;
    /**
     * Returns true if the given object is an instance of Replicator.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Replicator;
    /**
     * ARN of the Replicator.
     */
    readonly arn: pulumi.Output<string>;
    readonly currentVersion: pulumi.Output<string>;
    /**
     * A summary description of the replicator.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * A list of Kafka clusters which are targets of the replicator.
     */
    readonly kafkaClusters: pulumi.Output<outputs.msk.ReplicatorKafkaCluster[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A list of replication configurations, where each configuration targets a given source cluster to target cluster replication flow.
     */
    readonly replicationInfoList: pulumi.Output<outputs.msk.ReplicatorReplicationInfoList>;
    /**
     * The name of the replicator.
     */
    readonly replicatorName: pulumi.Output<string>;
    /**
     * The ARN of the IAM role used by the replicator to access resources in the customer's account (e.g source and target clusters).
     */
    readonly serviceExecutionRoleArn: pulumi.Output<string>;
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
     * Create a Replicator resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ReplicatorArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Replicator resources.
 */
export interface ReplicatorState {
    /**
     * ARN of the Replicator.
     */
    arn?: pulumi.Input<string>;
    currentVersion?: pulumi.Input<string>;
    /**
     * A summary description of the replicator.
     */
    description?: pulumi.Input<string>;
    /**
     * A list of Kafka clusters which are targets of the replicator.
     */
    kafkaClusters?: pulumi.Input<pulumi.Input<inputs.msk.ReplicatorKafkaCluster>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of replication configurations, where each configuration targets a given source cluster to target cluster replication flow.
     */
    replicationInfoList?: pulumi.Input<inputs.msk.ReplicatorReplicationInfoList>;
    /**
     * The name of the replicator.
     */
    replicatorName?: pulumi.Input<string>;
    /**
     * The ARN of the IAM role used by the replicator to access resources in the customer's account (e.g source and target clusters).
     */
    serviceExecutionRoleArn?: pulumi.Input<string>;
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
}
/**
 * The set of arguments for constructing a Replicator resource.
 */
export interface ReplicatorArgs {
    /**
     * A summary description of the replicator.
     */
    description?: pulumi.Input<string>;
    /**
     * A list of Kafka clusters which are targets of the replicator.
     */
    kafkaClusters: pulumi.Input<pulumi.Input<inputs.msk.ReplicatorKafkaCluster>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of replication configurations, where each configuration targets a given source cluster to target cluster replication flow.
     */
    replicationInfoList: pulumi.Input<inputs.msk.ReplicatorReplicationInfoList>;
    /**
     * The name of the replicator.
     */
    replicatorName: pulumi.Input<string>;
    /**
     * The ARN of the IAM role used by the replicator to access resources in the customer's account (e.g source and target clusters).
     */
    serviceExecutionRoleArn: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
