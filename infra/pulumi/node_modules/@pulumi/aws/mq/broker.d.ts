import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ### Basic Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.mq.Broker("example", {
 *     brokerName: "example",
 *     configuration: {
 *         id: test.id,
 *         revision: test.latestRevision,
 *     },
 *     engineType: "ActiveMQ",
 *     engineVersion: "5.17.6",
 *     hostInstanceType: "mq.t2.micro",
 *     securityGroups: [testAwsSecurityGroup.id],
 *     users: [{
 *         username: "example_user",
 *         password: "<password>",
 *     }],
 * });
 * ```
 *
 * ### High-throughput Optimized Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.mq.Broker("example", {
 *     brokerName: "example",
 *     configuration: {
 *         id: test.id,
 *         revision: test.latestRevision,
 *     },
 *     engineType: "ActiveMQ",
 *     engineVersion: "5.17.6",
 *     storageType: "ebs",
 *     hostInstanceType: "mq.m5.large",
 *     securityGroups: [testAwsSecurityGroup.id],
 *     users: [{
 *         username: "example_user",
 *         password: "<password>",
 *     }],
 * });
 * ```
 *
 * ### Cross-Region Data Replication
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const examplePrimary = new aws.mq.Broker("example_primary", {
 *     applyImmediately: true,
 *     brokerName: "example_primary",
 *     engineType: "ActiveMQ",
 *     engineVersion: "5.17.6",
 *     hostInstanceType: "mq.m5.large",
 *     securityGroups: [examplePrimaryAwsSecurityGroup.id],
 *     deploymentMode: "ACTIVE_STANDBY_MULTI_AZ",
 *     users: [
 *         {
 *             username: "example_user",
 *             password: "<password>",
 *         },
 *         {
 *             username: "example_replication_user",
 *             password: "<password>",
 *             replicationUser: true,
 *         },
 *     ],
 * });
 * const example = new aws.mq.Broker("example", {
 *     applyImmediately: true,
 *     brokerName: "example",
 *     engineType: "ActiveMQ",
 *     engineVersion: "5.17.6",
 *     hostInstanceType: "mq.m5.large",
 *     securityGroups: [exampleAwsSecurityGroup.id],
 *     deploymentMode: "ACTIVE_STANDBY_MULTI_AZ",
 *     dataReplicationMode: "CRDR",
 *     dataReplicationPrimaryBrokerArn: primary.arn,
 *     users: [
 *         {
 *             username: "example_user",
 *             password: "<password>",
 *         },
 *         {
 *             username: "example_replication_user",
 *             password: "<password>",
 *             replicationUser: true,
 *         },
 *     ],
 * });
 * ```
 *
 * See the [AWS MQ documentation](https://docs.aws.amazon.com/amazon-mq/latest/developer-guide/crdr-for-active-mq.html) on cross-region data replication for additional details.
 *
 * ## Import
 *
 * Using `pulumi import`, import MQ Brokers using their broker id. For example:
 *
 * ```sh
 * $ pulumi import aws:mq/broker:Broker example a1b2c3d4-d5f6-7777-8888-9999aaaabbbbcccc
 * ```
 */
export declare class Broker extends pulumi.CustomResource {
    /**
     * Get an existing Broker resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BrokerState, opts?: pulumi.CustomResourceOptions): Broker;
    /**
     * Returns true if the given object is an instance of Broker.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Broker;
    /**
     * Whether to apply broker modifications immediately. Default is `false`.
     */
    readonly applyImmediately: pulumi.Output<boolean | undefined>;
    /**
     * ARN of the broker.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Authentication strategy used to secure the broker. Valid values are `simple` and `ldap`. `ldap` is not supported for `engineType` `RabbitMQ`.
     */
    readonly authenticationStrategy: pulumi.Output<string>;
    /**
     * Whether to automatically upgrade to new minor versions of brokers as Amazon MQ makes releases available.
     */
    readonly autoMinorVersionUpgrade: pulumi.Output<boolean | undefined>;
    /**
     * Name of the broker.
     */
    readonly brokerName: pulumi.Output<string>;
    /**
     * Configuration block for broker configuration. Applies to `engineType` of `ActiveMQ` and `RabbitMQ` only. Detailed below.
     */
    readonly configuration: pulumi.Output<outputs.mq.BrokerConfiguration>;
    /**
     * Whether this broker is part of a data replication pair. Valid values are `CRDR` and `NONE`.
     */
    readonly dataReplicationMode: pulumi.Output<string>;
    /**
     * ARN of the primary broker used to replicate data in a data replication pair. Required when `dataReplicationMode` is `CRDR`.
     */
    readonly dataReplicationPrimaryBrokerArn: pulumi.Output<string | undefined>;
    /**
     * Deployment mode of the broker. Valid values are `SINGLE_INSTANCE`, `ACTIVE_STANDBY_MULTI_AZ`, and `CLUSTER_MULTI_AZ`. Default is `SINGLE_INSTANCE`.
     */
    readonly deploymentMode: pulumi.Output<string | undefined>;
    /**
     * Configuration block containing encryption options. Detailed below.
     */
    readonly encryptionOptions: pulumi.Output<outputs.mq.BrokerEncryptionOptions | undefined>;
    /**
     * Type of broker engine. Valid values are `ActiveMQ` and `RabbitMQ`.
     */
    readonly engineType: pulumi.Output<string>;
    /**
     * Version of the broker engine.
     */
    readonly engineVersion: pulumi.Output<string>;
    /**
     * Broker's instance type. For example, `mq.t3.micro`, `mq.m5.large`.
     */
    readonly hostInstanceType: pulumi.Output<string>;
    /**
     * List of information about allocated brokers (both active & standby).
     */
    readonly instances: pulumi.Output<outputs.mq.BrokerInstance[]>;
    /**
     * Configuration block for the LDAP server used to authenticate and authorize connections. Not supported for `engineType` `RabbitMQ`. Detailed below.
     */
    readonly ldapServerMetadata: pulumi.Output<outputs.mq.BrokerLdapServerMetadata | undefined>;
    /**
     * Configuration block for the logging configuration. Detailed below.
     */
    readonly logs: pulumi.Output<outputs.mq.BrokerLogs | undefined>;
    /**
     * Configuration block for the maintenance window start time. Detailed below.
     */
    readonly maintenanceWindowStartTime: pulumi.Output<outputs.mq.BrokerMaintenanceWindowStartTime>;
    /**
     * Data replication mode that will be applied after reboot.
     */
    readonly pendingDataReplicationMode: pulumi.Output<string>;
    /**
     * Whether to enable connections from applications outside of the VPC that hosts the broker's subnets.
     */
    readonly publiclyAccessible: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * List of security group IDs assigned to the broker.
     */
    readonly securityGroups: pulumi.Output<string[] | undefined>;
    /**
     * Storage type of the broker. For `engineType` `ActiveMQ`, valid values are `efs` and `ebs` (AWS-default is `efs`). For `engineType` `RabbitMQ`, only `ebs` is supported. When using `ebs`, only the `mq.m5` broker instance type family is supported.
     */
    readonly storageType: pulumi.Output<string>;
    /**
     * List of subnet IDs in which to launch the broker. A `SINGLE_INSTANCE` deployment requires one subnet. An `ACTIVE_STANDBY_MULTI_AZ` deployment requires multiple subnets.
     */
    readonly subnetIds: pulumi.Output<string[]>;
    /**
     * Map of tags to assign to the broker. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Configuration block for broker users. For `engineType` of `RabbitMQ`, Amazon MQ does not return broker users preventing this resource from making user updates and drift detection. Detailed below.
     *
     * The following arguments are optional:
     */
    readonly users: pulumi.Output<outputs.mq.BrokerUser[]>;
    /**
     * Create a Broker resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BrokerArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Broker resources.
 */
export interface BrokerState {
    /**
     * Whether to apply broker modifications immediately. Default is `false`.
     */
    applyImmediately?: pulumi.Input<boolean>;
    /**
     * ARN of the broker.
     */
    arn?: pulumi.Input<string>;
    /**
     * Authentication strategy used to secure the broker. Valid values are `simple` and `ldap`. `ldap` is not supported for `engineType` `RabbitMQ`.
     */
    authenticationStrategy?: pulumi.Input<string>;
    /**
     * Whether to automatically upgrade to new minor versions of brokers as Amazon MQ makes releases available.
     */
    autoMinorVersionUpgrade?: pulumi.Input<boolean>;
    /**
     * Name of the broker.
     */
    brokerName?: pulumi.Input<string>;
    /**
     * Configuration block for broker configuration. Applies to `engineType` of `ActiveMQ` and `RabbitMQ` only. Detailed below.
     */
    configuration?: pulumi.Input<inputs.mq.BrokerConfiguration>;
    /**
     * Whether this broker is part of a data replication pair. Valid values are `CRDR` and `NONE`.
     */
    dataReplicationMode?: pulumi.Input<string>;
    /**
     * ARN of the primary broker used to replicate data in a data replication pair. Required when `dataReplicationMode` is `CRDR`.
     */
    dataReplicationPrimaryBrokerArn?: pulumi.Input<string>;
    /**
     * Deployment mode of the broker. Valid values are `SINGLE_INSTANCE`, `ACTIVE_STANDBY_MULTI_AZ`, and `CLUSTER_MULTI_AZ`. Default is `SINGLE_INSTANCE`.
     */
    deploymentMode?: pulumi.Input<string>;
    /**
     * Configuration block containing encryption options. Detailed below.
     */
    encryptionOptions?: pulumi.Input<inputs.mq.BrokerEncryptionOptions>;
    /**
     * Type of broker engine. Valid values are `ActiveMQ` and `RabbitMQ`.
     */
    engineType?: pulumi.Input<string>;
    /**
     * Version of the broker engine.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * Broker's instance type. For example, `mq.t3.micro`, `mq.m5.large`.
     */
    hostInstanceType?: pulumi.Input<string>;
    /**
     * List of information about allocated brokers (both active & standby).
     */
    instances?: pulumi.Input<pulumi.Input<inputs.mq.BrokerInstance>[]>;
    /**
     * Configuration block for the LDAP server used to authenticate and authorize connections. Not supported for `engineType` `RabbitMQ`. Detailed below.
     */
    ldapServerMetadata?: pulumi.Input<inputs.mq.BrokerLdapServerMetadata>;
    /**
     * Configuration block for the logging configuration. Detailed below.
     */
    logs?: pulumi.Input<inputs.mq.BrokerLogs>;
    /**
     * Configuration block for the maintenance window start time. Detailed below.
     */
    maintenanceWindowStartTime?: pulumi.Input<inputs.mq.BrokerMaintenanceWindowStartTime>;
    /**
     * Data replication mode that will be applied after reboot.
     */
    pendingDataReplicationMode?: pulumi.Input<string>;
    /**
     * Whether to enable connections from applications outside of the VPC that hosts the broker's subnets.
     */
    publiclyAccessible?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of security group IDs assigned to the broker.
     */
    securityGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Storage type of the broker. For `engineType` `ActiveMQ`, valid values are `efs` and `ebs` (AWS-default is `efs`). For `engineType` `RabbitMQ`, only `ebs` is supported. When using `ebs`, only the `mq.m5` broker instance type family is supported.
     */
    storageType?: pulumi.Input<string>;
    /**
     * List of subnet IDs in which to launch the broker. A `SINGLE_INSTANCE` deployment requires one subnet. An `ACTIVE_STANDBY_MULTI_AZ` deployment requires multiple subnets.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Map of tags to assign to the broker. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Configuration block for broker users. For `engineType` of `RabbitMQ`, Amazon MQ does not return broker users preventing this resource from making user updates and drift detection. Detailed below.
     *
     * The following arguments are optional:
     */
    users?: pulumi.Input<pulumi.Input<inputs.mq.BrokerUser>[]>;
}
/**
 * The set of arguments for constructing a Broker resource.
 */
export interface BrokerArgs {
    /**
     * Whether to apply broker modifications immediately. Default is `false`.
     */
    applyImmediately?: pulumi.Input<boolean>;
    /**
     * Authentication strategy used to secure the broker. Valid values are `simple` and `ldap`. `ldap` is not supported for `engineType` `RabbitMQ`.
     */
    authenticationStrategy?: pulumi.Input<string>;
    /**
     * Whether to automatically upgrade to new minor versions of brokers as Amazon MQ makes releases available.
     */
    autoMinorVersionUpgrade?: pulumi.Input<boolean>;
    /**
     * Name of the broker.
     */
    brokerName?: pulumi.Input<string>;
    /**
     * Configuration block for broker configuration. Applies to `engineType` of `ActiveMQ` and `RabbitMQ` only. Detailed below.
     */
    configuration?: pulumi.Input<inputs.mq.BrokerConfiguration>;
    /**
     * Whether this broker is part of a data replication pair. Valid values are `CRDR` and `NONE`.
     */
    dataReplicationMode?: pulumi.Input<string>;
    /**
     * ARN of the primary broker used to replicate data in a data replication pair. Required when `dataReplicationMode` is `CRDR`.
     */
    dataReplicationPrimaryBrokerArn?: pulumi.Input<string>;
    /**
     * Deployment mode of the broker. Valid values are `SINGLE_INSTANCE`, `ACTIVE_STANDBY_MULTI_AZ`, and `CLUSTER_MULTI_AZ`. Default is `SINGLE_INSTANCE`.
     */
    deploymentMode?: pulumi.Input<string>;
    /**
     * Configuration block containing encryption options. Detailed below.
     */
    encryptionOptions?: pulumi.Input<inputs.mq.BrokerEncryptionOptions>;
    /**
     * Type of broker engine. Valid values are `ActiveMQ` and `RabbitMQ`.
     */
    engineType: pulumi.Input<string>;
    /**
     * Version of the broker engine.
     */
    engineVersion: pulumi.Input<string>;
    /**
     * Broker's instance type. For example, `mq.t3.micro`, `mq.m5.large`.
     */
    hostInstanceType: pulumi.Input<string>;
    /**
     * Configuration block for the LDAP server used to authenticate and authorize connections. Not supported for `engineType` `RabbitMQ`. Detailed below.
     */
    ldapServerMetadata?: pulumi.Input<inputs.mq.BrokerLdapServerMetadata>;
    /**
     * Configuration block for the logging configuration. Detailed below.
     */
    logs?: pulumi.Input<inputs.mq.BrokerLogs>;
    /**
     * Configuration block for the maintenance window start time. Detailed below.
     */
    maintenanceWindowStartTime?: pulumi.Input<inputs.mq.BrokerMaintenanceWindowStartTime>;
    /**
     * Whether to enable connections from applications outside of the VPC that hosts the broker's subnets.
     */
    publiclyAccessible?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of security group IDs assigned to the broker.
     */
    securityGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Storage type of the broker. For `engineType` `ActiveMQ`, valid values are `efs` and `ebs` (AWS-default is `efs`). For `engineType` `RabbitMQ`, only `ebs` is supported. When using `ebs`, only the `mq.m5` broker instance type family is supported.
     */
    storageType?: pulumi.Input<string>;
    /**
     * List of subnet IDs in which to launch the broker. A `SINGLE_INSTANCE` deployment requires one subnet. An `ACTIVE_STANDBY_MULTI_AZ` deployment requires multiple subnets.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Map of tags to assign to the broker. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block for broker users. For `engineType` of `RabbitMQ`, Amazon MQ does not return broker users preventing this resource from making user updates and drift detection. Detailed below.
     *
     * The following arguments are optional:
     */
    users: pulumi.Input<pulumi.Input<inputs.mq.BrokerUser>[]>;
}
