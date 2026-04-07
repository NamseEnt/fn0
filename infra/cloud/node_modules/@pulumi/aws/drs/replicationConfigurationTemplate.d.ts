import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Elastic Disaster Recovery replication configuration template resource. Before using DRS, your account must be [initialized](https://docs.aws.amazon.com/drs/latest/userguide/getting-started-initializing.html).
 *
 * > **NOTE:** Your configuration must use the PIT policy shown in the basic configuration due to AWS rules. The only value that you can change is the `retentionDuration` of `ruleId` 3.
 *
 * ## Example Usage
 *
 * ### Basic configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.drs.ReplicationConfigurationTemplate("example", {
 *     associateDefaultSecurityGroup: false,
 *     bandwidthThrottling: 12,
 *     createPublicIp: false,
 *     dataPlaneRouting: "PRIVATE_IP",
 *     defaultLargeStagingDiskType: "GP2",
 *     ebsEncryption: "DEFAULT",
 *     ebsEncryptionKeyArn: "arn:aws:kms:us-east-1:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab",
 *     replicationServerInstanceType: "t3.small",
 *     replicationServersSecurityGroupsIds: exampleAwsSecurityGroup.map(__item => __item.id),
 *     stagingAreaSubnetId: exampleAwsSubnet.id,
 *     useDedicatedReplicationServer: false,
 *     pitPolicies: [
 *         {
 *             enabled: true,
 *             interval: 10,
 *             retentionDuration: 60,
 *             units: "MINUTE",
 *             ruleId: 1,
 *         },
 *         {
 *             enabled: true,
 *             interval: 1,
 *             retentionDuration: 24,
 *             units: "HOUR",
 *             ruleId: 2,
 *         },
 *         {
 *             enabled: true,
 *             interval: 1,
 *             retentionDuration: 3,
 *             units: "DAY",
 *             ruleId: 3,
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DRS Replication Configuration Template using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:drs/replicationConfigurationTemplate:ReplicationConfigurationTemplate example templateid
 * ```
 */
export declare class ReplicationConfigurationTemplate extends pulumi.CustomResource {
    /**
     * Get an existing ReplicationConfigurationTemplate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ReplicationConfigurationTemplateState, opts?: pulumi.CustomResourceOptions): ReplicationConfigurationTemplate;
    /**
     * Returns true if the given object is an instance of ReplicationConfigurationTemplate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ReplicationConfigurationTemplate;
    /**
     * Replication configuration template ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Whether to associate the default Elastic Disaster Recovery Security group with the Replication Configuration Template.
     */
    readonly associateDefaultSecurityGroup: pulumi.Output<boolean>;
    /**
     * Whether to allow the AWS replication agent to automatically replicate newly added disks.
     */
    readonly autoReplicateNewDisks: pulumi.Output<boolean>;
    /**
     * Configure bandwidth throttling for the outbound data transfer rate of the Source Server in Mbps.
     */
    readonly bandwidthThrottling: pulumi.Output<number>;
    /**
     * Whether to create a Public IP for the Recovery Instance by default.
     */
    readonly createPublicIp: pulumi.Output<boolean>;
    /**
     * Data plane routing mechanism that will be used for replication. Valid values are `PUBLIC_IP` and `PRIVATE_IP`.
     */
    readonly dataPlaneRouting: pulumi.Output<string>;
    /**
     * Staging Disk EBS volume type to be used during replication. Valid values are `GP2`, `GP3`, `ST1`, or `AUTO`.
     */
    readonly defaultLargeStagingDiskType: pulumi.Output<string>;
    /**
     * Type of EBS encryption to be used during replication. Valid values are `DEFAULT` and `CUSTOM`.
     */
    readonly ebsEncryption: pulumi.Output<string>;
    /**
     * ARN of the EBS encryption key to be used during replication.
     */
    readonly ebsEncryptionKeyArn: pulumi.Output<string | undefined>;
    /**
     * Configuration block for Point in time (PIT) policy to manage snapshots taken during replication. See below.
     */
    readonly pitPolicies: pulumi.Output<outputs.drs.ReplicationConfigurationTemplatePitPolicy[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Instance type to be used for the replication server.
     */
    readonly replicationServerInstanceType: pulumi.Output<string>;
    /**
     * Security group IDs that will be used by the replication server.
     */
    readonly replicationServersSecurityGroupsIds: pulumi.Output<string[]>;
    /**
     * Subnet to be used by the replication staging area.
     */
    readonly stagingAreaSubnetId: pulumi.Output<string>;
    /**
     * Set of tags to be associated with all resources created in the replication staging area: EC2 replication server, EBS volumes, EBS snapshots, etc.
     */
    readonly stagingAreaTags: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Set of tags to be associated with the Replication Configuration Template resource.
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
    readonly timeouts: pulumi.Output<outputs.drs.ReplicationConfigurationTemplateTimeouts | undefined>;
    /**
     * Whether to use a dedicated Replication Server in the replication staging area.
     *
     * The following arguments are optional:
     */
    readonly useDedicatedReplicationServer: pulumi.Output<boolean>;
    /**
     * Create a ReplicationConfigurationTemplate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ReplicationConfigurationTemplateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ReplicationConfigurationTemplate resources.
 */
export interface ReplicationConfigurationTemplateState {
    /**
     * Replication configuration template ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * Whether to associate the default Elastic Disaster Recovery Security group with the Replication Configuration Template.
     */
    associateDefaultSecurityGroup?: pulumi.Input<boolean>;
    /**
     * Whether to allow the AWS replication agent to automatically replicate newly added disks.
     */
    autoReplicateNewDisks?: pulumi.Input<boolean>;
    /**
     * Configure bandwidth throttling for the outbound data transfer rate of the Source Server in Mbps.
     */
    bandwidthThrottling?: pulumi.Input<number>;
    /**
     * Whether to create a Public IP for the Recovery Instance by default.
     */
    createPublicIp?: pulumi.Input<boolean>;
    /**
     * Data plane routing mechanism that will be used for replication. Valid values are `PUBLIC_IP` and `PRIVATE_IP`.
     */
    dataPlaneRouting?: pulumi.Input<string>;
    /**
     * Staging Disk EBS volume type to be used during replication. Valid values are `GP2`, `GP3`, `ST1`, or `AUTO`.
     */
    defaultLargeStagingDiskType?: pulumi.Input<string>;
    /**
     * Type of EBS encryption to be used during replication. Valid values are `DEFAULT` and `CUSTOM`.
     */
    ebsEncryption?: pulumi.Input<string>;
    /**
     * ARN of the EBS encryption key to be used during replication.
     */
    ebsEncryptionKeyArn?: pulumi.Input<string>;
    /**
     * Configuration block for Point in time (PIT) policy to manage snapshots taken during replication. See below.
     */
    pitPolicies?: pulumi.Input<pulumi.Input<inputs.drs.ReplicationConfigurationTemplatePitPolicy>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Instance type to be used for the replication server.
     */
    replicationServerInstanceType?: pulumi.Input<string>;
    /**
     * Security group IDs that will be used by the replication server.
     */
    replicationServersSecurityGroupsIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Subnet to be used by the replication staging area.
     */
    stagingAreaSubnetId?: pulumi.Input<string>;
    /**
     * Set of tags to be associated with all resources created in the replication staging area: EC2 replication server, EBS volumes, EBS snapshots, etc.
     */
    stagingAreaTags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Set of tags to be associated with the Replication Configuration Template resource.
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
    timeouts?: pulumi.Input<inputs.drs.ReplicationConfigurationTemplateTimeouts>;
    /**
     * Whether to use a dedicated Replication Server in the replication staging area.
     *
     * The following arguments are optional:
     */
    useDedicatedReplicationServer?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a ReplicationConfigurationTemplate resource.
 */
export interface ReplicationConfigurationTemplateArgs {
    /**
     * Whether to associate the default Elastic Disaster Recovery Security group with the Replication Configuration Template.
     */
    associateDefaultSecurityGroup: pulumi.Input<boolean>;
    /**
     * Whether to allow the AWS replication agent to automatically replicate newly added disks.
     */
    autoReplicateNewDisks?: pulumi.Input<boolean>;
    /**
     * Configure bandwidth throttling for the outbound data transfer rate of the Source Server in Mbps.
     */
    bandwidthThrottling: pulumi.Input<number>;
    /**
     * Whether to create a Public IP for the Recovery Instance by default.
     */
    createPublicIp: pulumi.Input<boolean>;
    /**
     * Data plane routing mechanism that will be used for replication. Valid values are `PUBLIC_IP` and `PRIVATE_IP`.
     */
    dataPlaneRouting: pulumi.Input<string>;
    /**
     * Staging Disk EBS volume type to be used during replication. Valid values are `GP2`, `GP3`, `ST1`, or `AUTO`.
     */
    defaultLargeStagingDiskType: pulumi.Input<string>;
    /**
     * Type of EBS encryption to be used during replication. Valid values are `DEFAULT` and `CUSTOM`.
     */
    ebsEncryption: pulumi.Input<string>;
    /**
     * ARN of the EBS encryption key to be used during replication.
     */
    ebsEncryptionKeyArn?: pulumi.Input<string>;
    /**
     * Configuration block for Point in time (PIT) policy to manage snapshots taken during replication. See below.
     */
    pitPolicies?: pulumi.Input<pulumi.Input<inputs.drs.ReplicationConfigurationTemplatePitPolicy>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Instance type to be used for the replication server.
     */
    replicationServerInstanceType: pulumi.Input<string>;
    /**
     * Security group IDs that will be used by the replication server.
     */
    replicationServersSecurityGroupsIds: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Subnet to be used by the replication staging area.
     */
    stagingAreaSubnetId: pulumi.Input<string>;
    /**
     * Set of tags to be associated with all resources created in the replication staging area: EC2 replication server, EBS volumes, EBS snapshots, etc.
     */
    stagingAreaTags: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Set of tags to be associated with the Replication Configuration Template resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.drs.ReplicationConfigurationTemplateTimeouts>;
    /**
     * Whether to use a dedicated Replication Server in the replication staging area.
     *
     * The following arguments are optional:
     */
    useDedicatedReplicationServer: pulumi.Input<boolean>;
}
