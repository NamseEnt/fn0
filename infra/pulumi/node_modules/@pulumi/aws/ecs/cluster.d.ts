import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an ECS cluster.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const foo = new aws.ecs.Cluster("foo", {
 *     name: "white-hart",
 *     settings: [{
 *         name: "containerInsights",
 *         value: "enabled",
 *     }],
 * });
 * ```
 *
 * ### Execute Command Configuration with Override Logging
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.Key("example", {
 *     description: "example",
 *     deletionWindowInDays: 7,
 * });
 * const exampleLogGroup = new aws.cloudwatch.LogGroup("example", {name: "example"});
 * const test = new aws.ecs.Cluster("test", {
 *     name: "example",
 *     configuration: {
 *         executeCommandConfiguration: {
 *             kmsKeyId: example.arn,
 *             logging: "OVERRIDE",
 *             logConfiguration: {
 *                 cloudWatchEncryptionEnabled: true,
 *                 cloudWatchLogGroupName: exampleLogGroup.name,
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ### Fargate Ephemeral Storage Encryption with Customer-Managed KMS Key
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const example = new aws.kms.Key("example", {
 *     description: "example",
 *     deletionWindowInDays: 7,
 * });
 * const exampleKeyPolicy = new aws.kms.KeyPolicy("example", {
 *     keyId: example.id,
 *     policy: JSON.stringify({
 *         Id: "ECSClusterFargatePolicy",
 *         Statement: [
 *             {
 *                 Sid: "Enable IAM User Permissions",
 *                 Effect: "Allow",
 *                 Principal: {
 *                     AWS: "*",
 *                 },
 *                 Action: "kms:*",
 *                 Resource: "*",
 *             },
 *             {
 *                 Sid: "Allow generate data key access for Fargate tasks.",
 *                 Effect: "Allow",
 *                 Principal: {
 *                     Service: "fargate.amazonaws.com",
 *                 },
 *                 Action: ["kms:GenerateDataKeyWithoutPlaintext"],
 *                 Condition: {
 *                     StringEquals: {
 *                         "kms:EncryptionContext:aws:ecs:clusterAccount": [current.then(current => current.accountId)],
 *                         "kms:EncryptionContext:aws:ecs:clusterName": ["example"],
 *                     },
 *                 },
 *                 Resource: "*",
 *             },
 *             {
 *                 Sid: "Allow grant creation permission for Fargate tasks.",
 *                 Effect: "Allow",
 *                 Principal: {
 *                     Service: "fargate.amazonaws.com",
 *                 },
 *                 Action: ["kms:CreateGrant"],
 *                 Condition: {
 *                     StringEquals: {
 *                         "kms:EncryptionContext:aws:ecs:clusterAccount": [current.then(current => current.accountId)],
 *                         "kms:EncryptionContext:aws:ecs:clusterName": ["example"],
 *                     },
 *                     "ForAllValues:StringEquals": {
 *                         "kms:GrantOperations": ["Decrypt"],
 *                     },
 *                 },
 *                 Resource: "*",
 *             },
 *         ],
 *         Version: "2012-10-17",
 *     }),
 * });
 * const test = new aws.ecs.Cluster("test", {
 *     name: "example",
 *     configuration: {
 *         managedStorageConfiguration: {
 *             fargateEphemeralStorageKmsKeyId: example.id,
 *         },
 *     },
 * }, {
 *     dependsOn: [exampleKeyPolicy],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import ECS clusters using the cluster name. For example:
 *
 * ```sh
 * $ pulumi import aws:ecs/cluster:Cluster stateless stateless-app
 * ```
 */
export declare class Cluster extends pulumi.CustomResource {
    /**
     * Get an existing Cluster resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClusterState, opts?: pulumi.CustomResourceOptions): Cluster;
    /**
     * Returns true if the given object is an instance of Cluster.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Cluster;
    /**
     * ARN that identifies the cluster.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Execute command configuration for the cluster. See `configuration` Block for details.
     */
    readonly configuration: pulumi.Output<outputs.ecs.ClusterConfiguration | undefined>;
    /**
     * Name of the cluster (up to 255 letters, numbers, hyphens, and underscores)
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Default Service Connect namespace. See `serviceConnectDefaults` Block for details.
     */
    readonly serviceConnectDefaults: pulumi.Output<outputs.ecs.ClusterServiceConnectDefaults | undefined>;
    /**
     * Configuration block(s) with cluster settings. For example, this can be used to enable CloudWatch Container Insights for a cluster. See `setting` Block for details.
     */
    readonly settings: pulumi.Output<outputs.ecs.ClusterSetting[]>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Cluster resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ClusterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Cluster resources.
 */
export interface ClusterState {
    /**
     * ARN that identifies the cluster.
     */
    arn?: pulumi.Input<string>;
    /**
     * Execute command configuration for the cluster. See `configuration` Block for details.
     */
    configuration?: pulumi.Input<inputs.ecs.ClusterConfiguration>;
    /**
     * Name of the cluster (up to 255 letters, numbers, hyphens, and underscores)
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Default Service Connect namespace. See `serviceConnectDefaults` Block for details.
     */
    serviceConnectDefaults?: pulumi.Input<inputs.ecs.ClusterServiceConnectDefaults>;
    /**
     * Configuration block(s) with cluster settings. For example, this can be used to enable CloudWatch Container Insights for a cluster. See `setting` Block for details.
     */
    settings?: pulumi.Input<pulumi.Input<inputs.ecs.ClusterSetting>[]>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a Cluster resource.
 */
export interface ClusterArgs {
    /**
     * Execute command configuration for the cluster. See `configuration` Block for details.
     */
    configuration?: pulumi.Input<inputs.ecs.ClusterConfiguration>;
    /**
     * Name of the cluster (up to 255 letters, numbers, hyphens, and underscores)
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Default Service Connect namespace. See `serviceConnectDefaults` Block for details.
     */
    serviceConnectDefaults?: pulumi.Input<inputs.ecs.ClusterServiceConnectDefaults>;
    /**
     * Configuration block(s) with cluster settings. For example, this can be used to enable CloudWatch Container Insights for a cluster. See `setting` Block for details.
     */
    settings?: pulumi.Input<pulumi.Input<inputs.ecs.ClusterSetting>[]>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
