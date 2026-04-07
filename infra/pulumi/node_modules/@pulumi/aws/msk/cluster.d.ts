import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an Amazon MSK cluster.
 *
 * > **Note:** This resource manages _provisioned_ clusters. To manage a _serverless_ Amazon MSK cluster, use the `aws.msk.ServerlessCluster` resource.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const vpc = new aws.ec2.Vpc("vpc", {cidrBlock: "192.168.0.0/22"});
 * const azs = aws.getAvailabilityZones({
 *     state: "available",
 * });
 * const subnetAz1 = new aws.ec2.Subnet("subnet_az1", {
 *     availabilityZone: azs.then(azs => azs.names?.[0]),
 *     cidrBlock: "192.168.0.0/24",
 *     vpcId: vpc.id,
 * });
 * const subnetAz2 = new aws.ec2.Subnet("subnet_az2", {
 *     availabilityZone: azs.then(azs => azs.names?.[1]),
 *     cidrBlock: "192.168.1.0/24",
 *     vpcId: vpc.id,
 * });
 * const subnetAz3 = new aws.ec2.Subnet("subnet_az3", {
 *     availabilityZone: azs.then(azs => azs.names?.[2]),
 *     cidrBlock: "192.168.2.0/24",
 *     vpcId: vpc.id,
 * });
 * const sg = new aws.ec2.SecurityGroup("sg", {vpcId: vpc.id});
 * const kms = new aws.kms.Key("kms", {description: "example"});
 * const test = new aws.cloudwatch.LogGroup("test", {name: "msk_broker_logs"});
 * const bucket = new aws.s3.Bucket("bucket", {bucket: "msk-broker-logs-bucket"});
 * const bucketAcl = new aws.s3.BucketAcl("bucket_acl", {
 *     bucket: bucket.id,
 *     acl: "private",
 * });
 * const assumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["firehose.amazonaws.com"],
 *         }],
 *         actions: ["sts:AssumeRole"],
 *     }],
 * });
 * const firehoseRole = new aws.iam.Role("firehose_role", {
 *     name: "firehose_test_role",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const testStream = new aws.kinesis.FirehoseDeliveryStream("test_stream", {
 *     name: "kinesis-firehose-msk-broker-logs-stream",
 *     destination: "extended_s3",
 *     extendedS3Configuration: {
 *         roleArn: firehoseRole.arn,
 *         bucketArn: bucket.arn,
 *     },
 *     tags: {
 *         LogDeliveryEnabled: "placeholder",
 *     },
 * });
 * const example = new aws.msk.Cluster("example", {
 *     clusterName: "example",
 *     kafkaVersion: "3.2.0",
 *     numberOfBrokerNodes: 3,
 *     brokerNodeGroupInfo: {
 *         instanceType: "kafka.m5.large",
 *         clientSubnets: [
 *             subnetAz1.id,
 *             subnetAz2.id,
 *             subnetAz3.id,
 *         ],
 *         storageInfo: {
 *             ebsStorageInfo: {
 *                 volumeSize: 1000,
 *             },
 *         },
 *         securityGroups: [sg.id],
 *     },
 *     encryptionInfo: {
 *         encryptionAtRestKmsKeyArn: kms.arn,
 *     },
 *     openMonitoring: {
 *         prometheus: {
 *             jmxExporter: {
 *                 enabledInBroker: true,
 *             },
 *             nodeExporter: {
 *                 enabledInBroker: true,
 *             },
 *         },
 *     },
 *     loggingInfo: {
 *         brokerLogs: {
 *             cloudwatchLogs: {
 *                 enabled: true,
 *                 logGroup: test.name,
 *             },
 *             firehose: {
 *                 enabled: true,
 *                 deliveryStream: testStream.name,
 *             },
 *             s3: {
 *                 enabled: true,
 *                 bucket: bucket.id,
 *                 prefix: "logs/msk-",
 *             },
 *         },
 *     },
 *     tags: {
 *         foo: "bar",
 *     },
 * });
 * export const zookeeperConnectString = example.zookeeperConnectString;
 * export const bootstrapBrokersTls = example.bootstrapBrokersTls;
 * ```
 *
 * ### With volumeThroughput argument
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.msk.Cluster("example", {
 *     clusterName: "example",
 *     kafkaVersion: "2.7.1",
 *     numberOfBrokerNodes: 3,
 *     brokerNodeGroupInfo: {
 *         instanceType: "kafka.m5.4xlarge",
 *         clientSubnets: [
 *             subnetAz1.id,
 *             subnetAz2.id,
 *             subnetAz3.id,
 *         ],
 *         storageInfo: {
 *             ebsStorageInfo: {
 *                 provisionedThroughput: {
 *                     enabled: true,
 *                     volumeThroughput: 250,
 *                 },
 *                 volumeSize: 1000,
 *             },
 *         },
 *         securityGroups: [sg.id],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import MSK clusters using the cluster `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:msk/cluster:Cluster example arn:aws:kafka:us-west-2:123456789012:cluster/example/279c0212-d057-4dba-9aa9-1c4e5a25bfc7-3
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
     * Amazon Resource Name (ARN) of the MSK cluster.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Comma separated list of one or more hostname:port pairs of kafka brokers suitable to bootstrap connectivity to the kafka cluster. Contains a value if `encryption_info.0.encryption_in_transit.0.client_broker` is set to `PLAINTEXT` or `TLS_PLAINTEXT`. The resource sorts values alphabetically. AWS may not always return all endpoints so this value is not guaranteed to be stable across applies.
     */
    readonly bootstrapBrokers: pulumi.Output<string>;
    /**
     * One or more DNS names (or IP addresses) and SASL IAM port pairs. For example, `b-1-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9198,b-2-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9198,b-3-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9198`. This attribute will have a value if `encryption_info.0.encryption_in_transit.0.client_broker` is set to `TLS_PLAINTEXT` or `TLS` and `client_authentication.0.sasl.0.iam` is set to `true` and `broker_node_group_info.0.connectivity_info.0.public_access.0.type` is set to `SERVICE_PROVIDED_EIPS` and the cluster fulfill all other requirements for public access. The resource sorts the list alphabetically. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    readonly bootstrapBrokersPublicSaslIam: pulumi.Output<string>;
    /**
     * One or more DNS names (or IP addresses) and SASL SCRAM port pairs. For example, `b-1-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9196,b-2-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9196,b-3-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9196`. This attribute will have a value if `encryption_info.0.encryption_in_transit.0.client_broker` is set to `TLS_PLAINTEXT` or `TLS` and `client_authentication.0.sasl.0.scram` is set to `true` and `broker_node_group_info.0.connectivity_info.0.public_access.0.type` is set to `SERVICE_PROVIDED_EIPS` and the cluster fulfill all other requirements for public access. The resource sorts the list alphabetically. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    readonly bootstrapBrokersPublicSaslScram: pulumi.Output<string>;
    /**
     * One or more DNS names (or IP addresses) and TLS port pairs. For example, `b-1-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9194,b-2-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9194,b-3-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9194`. This attribute will have a value if `encryption_info.0.encryption_in_transit.0.client_broker` is set to `TLS_PLAINTEXT` or `TLS` and `broker_node_group_info.0.connectivity_info.0.public_access.0.type` is set to `SERVICE_PROVIDED_EIPS` and the cluster fulfill all other requirements for public access. The resource sorts the list alphabetically. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    readonly bootstrapBrokersPublicTls: pulumi.Output<string>;
    /**
     * One or more DNS names (or IP addresses) and SASL IAM port pairs. For example, `b-1.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9098,b-2.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9098,b-3.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9098`. This attribute will have a value if `encryption_info.0.encryption_in_transit.0.client_broker` is set to `TLS_PLAINTEXT` or `TLS` and `client_authentication.0.sasl.0.iam` is set to `true`. The resource sorts the list alphabetically. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    readonly bootstrapBrokersSaslIam: pulumi.Output<string>;
    /**
     * One or more DNS names (or IP addresses) and SASL SCRAM port pairs. For example, `b-1.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9096,b-2.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9096,b-3.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9096`. This attribute will have a value if `encryption_info.0.encryption_in_transit.0.client_broker` is set to `TLS_PLAINTEXT` or `TLS` and `client_authentication.0.sasl.0.scram` is set to `true`. The resource sorts the list alphabetically. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    readonly bootstrapBrokersSaslScram: pulumi.Output<string>;
    /**
     * One or more DNS names (or IP addresses) and TLS port pairs. For example, `b-1.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9094,b-2.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9094,b-3.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9094`. This attribute will have a value if `encryption_info.0.encryption_in_transit.0.client_broker` is set to `TLS_PLAINTEXT` or `TLS`. The resource sorts the list alphabetically. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    readonly bootstrapBrokersTls: pulumi.Output<string>;
    /**
     * A string containing one or more DNS names (or IP addresses) and SASL IAM port pairs for VPC connectivity. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    readonly bootstrapBrokersVpcConnectivitySaslIam: pulumi.Output<string>;
    /**
     * A string containing one or more DNS names (or IP addresses) and SASL SCRAM port pairs for VPC connectivity. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    readonly bootstrapBrokersVpcConnectivitySaslScram: pulumi.Output<string>;
    /**
     * A string containing one or more DNS names (or IP addresses) and TLS port pairs for VPC connectivity. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    readonly bootstrapBrokersVpcConnectivityTls: pulumi.Output<string>;
    /**
     * Configuration block for the broker nodes of the Kafka cluster. See brokerNodeGroupInfo Argument Reference below.
     */
    readonly brokerNodeGroupInfo: pulumi.Output<outputs.msk.ClusterBrokerNodeGroupInfo>;
    /**
     * Configuration block for specifying a client authentication. See clientAuthentication Argument Reference below.
     */
    readonly clientAuthentication: pulumi.Output<outputs.msk.ClusterClientAuthentication | undefined>;
    /**
     * Name of the MSK cluster.
     */
    readonly clusterName: pulumi.Output<string>;
    /**
     * UUID of the MSK cluster, for use in IAM policies.
     */
    readonly clusterUuid: pulumi.Output<string>;
    /**
     * Configuration block for specifying an MSK Configuration to attach to Kafka brokers. See configurationInfo Argument Reference below.
     */
    readonly configurationInfo: pulumi.Output<outputs.msk.ClusterConfigurationInfo | undefined>;
    /**
     * Current version of the MSK Cluster used for updates, e.g., `K13V1IB3VIYZZH`
     */
    readonly currentVersion: pulumi.Output<string>;
    /**
     * Configuration block for specifying encryption. See encryptionInfo Argument Reference below.
     */
    readonly encryptionInfo: pulumi.Output<outputs.msk.ClusterEncryptionInfo | undefined>;
    /**
     * Specify the desired enhanced MSK CloudWatch monitoring level. See [Monitoring Amazon MSK with Amazon CloudWatch](https://docs.aws.amazon.com/msk/latest/developerguide/monitoring.html)
     */
    readonly enhancedMonitoring: pulumi.Output<string | undefined>;
    /**
     * Specify the desired Kafka software version.
     */
    readonly kafkaVersion: pulumi.Output<string>;
    /**
     * Configuration block for streaming broker logs to Cloudwatch/S3/Kinesis Firehose. See loggingInfo Argument Reference below.
     */
    readonly loggingInfo: pulumi.Output<outputs.msk.ClusterLoggingInfo | undefined>;
    /**
     * The desired total number of broker nodes in the kafka cluster.  It must be a multiple of the number of specified client subnets.
     */
    readonly numberOfBrokerNodes: pulumi.Output<number>;
    /**
     * Configuration block for JMX and Node monitoring for the MSK cluster. See openMonitoring Argument Reference below.
     */
    readonly openMonitoring: pulumi.Output<outputs.msk.ClusterOpenMonitoring | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Controls storage mode for supported storage tiers. Valid values are: `LOCAL` or `TIERED`.
     */
    readonly storageMode: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * A comma separated list of one or more hostname:port pairs to use to connect to the Apache Zookeeper cluster. The returned values are sorted alphabetically. The AWS API may not return all endpoints, so this value is not guaranteed to be stable across applies.
     */
    readonly zookeeperConnectString: pulumi.Output<string>;
    /**
     * A comma separated list of one or more hostname:port pairs to use to connect to the Apache Zookeeper cluster via TLS. The returned values are sorted alphabetically. The AWS API may not return all endpoints, so this value is not guaranteed to be stable across applies.
     */
    readonly zookeeperConnectStringTls: pulumi.Output<string>;
    /**
     * Create a Cluster resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ClusterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Cluster resources.
 */
export interface ClusterState {
    /**
     * Amazon Resource Name (ARN) of the MSK cluster.
     */
    arn?: pulumi.Input<string>;
    /**
     * Comma separated list of one or more hostname:port pairs of kafka brokers suitable to bootstrap connectivity to the kafka cluster. Contains a value if `encryption_info.0.encryption_in_transit.0.client_broker` is set to `PLAINTEXT` or `TLS_PLAINTEXT`. The resource sorts values alphabetically. AWS may not always return all endpoints so this value is not guaranteed to be stable across applies.
     */
    bootstrapBrokers?: pulumi.Input<string>;
    /**
     * One or more DNS names (or IP addresses) and SASL IAM port pairs. For example, `b-1-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9198,b-2-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9198,b-3-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9198`. This attribute will have a value if `encryption_info.0.encryption_in_transit.0.client_broker` is set to `TLS_PLAINTEXT` or `TLS` and `client_authentication.0.sasl.0.iam` is set to `true` and `broker_node_group_info.0.connectivity_info.0.public_access.0.type` is set to `SERVICE_PROVIDED_EIPS` and the cluster fulfill all other requirements for public access. The resource sorts the list alphabetically. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    bootstrapBrokersPublicSaslIam?: pulumi.Input<string>;
    /**
     * One or more DNS names (or IP addresses) and SASL SCRAM port pairs. For example, `b-1-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9196,b-2-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9196,b-3-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9196`. This attribute will have a value if `encryption_info.0.encryption_in_transit.0.client_broker` is set to `TLS_PLAINTEXT` or `TLS` and `client_authentication.0.sasl.0.scram` is set to `true` and `broker_node_group_info.0.connectivity_info.0.public_access.0.type` is set to `SERVICE_PROVIDED_EIPS` and the cluster fulfill all other requirements for public access. The resource sorts the list alphabetically. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    bootstrapBrokersPublicSaslScram?: pulumi.Input<string>;
    /**
     * One or more DNS names (or IP addresses) and TLS port pairs. For example, `b-1-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9194,b-2-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9194,b-3-public.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9194`. This attribute will have a value if `encryption_info.0.encryption_in_transit.0.client_broker` is set to `TLS_PLAINTEXT` or `TLS` and `broker_node_group_info.0.connectivity_info.0.public_access.0.type` is set to `SERVICE_PROVIDED_EIPS` and the cluster fulfill all other requirements for public access. The resource sorts the list alphabetically. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    bootstrapBrokersPublicTls?: pulumi.Input<string>;
    /**
     * One or more DNS names (or IP addresses) and SASL IAM port pairs. For example, `b-1.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9098,b-2.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9098,b-3.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9098`. This attribute will have a value if `encryption_info.0.encryption_in_transit.0.client_broker` is set to `TLS_PLAINTEXT` or `TLS` and `client_authentication.0.sasl.0.iam` is set to `true`. The resource sorts the list alphabetically. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    bootstrapBrokersSaslIam?: pulumi.Input<string>;
    /**
     * One or more DNS names (or IP addresses) and SASL SCRAM port pairs. For example, `b-1.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9096,b-2.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9096,b-3.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9096`. This attribute will have a value if `encryption_info.0.encryption_in_transit.0.client_broker` is set to `TLS_PLAINTEXT` or `TLS` and `client_authentication.0.sasl.0.scram` is set to `true`. The resource sorts the list alphabetically. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    bootstrapBrokersSaslScram?: pulumi.Input<string>;
    /**
     * One or more DNS names (or IP addresses) and TLS port pairs. For example, `b-1.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9094,b-2.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9094,b-3.exampleClusterName.abcde.c2.kafka.us-east-1.amazonaws.com:9094`. This attribute will have a value if `encryption_info.0.encryption_in_transit.0.client_broker` is set to `TLS_PLAINTEXT` or `TLS`. The resource sorts the list alphabetically. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    bootstrapBrokersTls?: pulumi.Input<string>;
    /**
     * A string containing one or more DNS names (or IP addresses) and SASL IAM port pairs for VPC connectivity. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    bootstrapBrokersVpcConnectivitySaslIam?: pulumi.Input<string>;
    /**
     * A string containing one or more DNS names (or IP addresses) and SASL SCRAM port pairs for VPC connectivity. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    bootstrapBrokersVpcConnectivitySaslScram?: pulumi.Input<string>;
    /**
     * A string containing one or more DNS names (or IP addresses) and TLS port pairs for VPC connectivity. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    bootstrapBrokersVpcConnectivityTls?: pulumi.Input<string>;
    /**
     * Configuration block for the broker nodes of the Kafka cluster. See brokerNodeGroupInfo Argument Reference below.
     */
    brokerNodeGroupInfo?: pulumi.Input<inputs.msk.ClusterBrokerNodeGroupInfo>;
    /**
     * Configuration block for specifying a client authentication. See clientAuthentication Argument Reference below.
     */
    clientAuthentication?: pulumi.Input<inputs.msk.ClusterClientAuthentication>;
    /**
     * Name of the MSK cluster.
     */
    clusterName?: pulumi.Input<string>;
    /**
     * UUID of the MSK cluster, for use in IAM policies.
     */
    clusterUuid?: pulumi.Input<string>;
    /**
     * Configuration block for specifying an MSK Configuration to attach to Kafka brokers. See configurationInfo Argument Reference below.
     */
    configurationInfo?: pulumi.Input<inputs.msk.ClusterConfigurationInfo>;
    /**
     * Current version of the MSK Cluster used for updates, e.g., `K13V1IB3VIYZZH`
     */
    currentVersion?: pulumi.Input<string>;
    /**
     * Configuration block for specifying encryption. See encryptionInfo Argument Reference below.
     */
    encryptionInfo?: pulumi.Input<inputs.msk.ClusterEncryptionInfo>;
    /**
     * Specify the desired enhanced MSK CloudWatch monitoring level. See [Monitoring Amazon MSK with Amazon CloudWatch](https://docs.aws.amazon.com/msk/latest/developerguide/monitoring.html)
     */
    enhancedMonitoring?: pulumi.Input<string>;
    /**
     * Specify the desired Kafka software version.
     */
    kafkaVersion?: pulumi.Input<string>;
    /**
     * Configuration block for streaming broker logs to Cloudwatch/S3/Kinesis Firehose. See loggingInfo Argument Reference below.
     */
    loggingInfo?: pulumi.Input<inputs.msk.ClusterLoggingInfo>;
    /**
     * The desired total number of broker nodes in the kafka cluster.  It must be a multiple of the number of specified client subnets.
     */
    numberOfBrokerNodes?: pulumi.Input<number>;
    /**
     * Configuration block for JMX and Node monitoring for the MSK cluster. See openMonitoring Argument Reference below.
     */
    openMonitoring?: pulumi.Input<inputs.msk.ClusterOpenMonitoring>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Controls storage mode for supported storage tiers. Valid values are: `LOCAL` or `TIERED`.
     */
    storageMode?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * A comma separated list of one or more hostname:port pairs to use to connect to the Apache Zookeeper cluster. The returned values are sorted alphabetically. The AWS API may not return all endpoints, so this value is not guaranteed to be stable across applies.
     */
    zookeeperConnectString?: pulumi.Input<string>;
    /**
     * A comma separated list of one or more hostname:port pairs to use to connect to the Apache Zookeeper cluster via TLS. The returned values are sorted alphabetically. The AWS API may not return all endpoints, so this value is not guaranteed to be stable across applies.
     */
    zookeeperConnectStringTls?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Cluster resource.
 */
export interface ClusterArgs {
    /**
     * Configuration block for the broker nodes of the Kafka cluster. See brokerNodeGroupInfo Argument Reference below.
     */
    brokerNodeGroupInfo: pulumi.Input<inputs.msk.ClusterBrokerNodeGroupInfo>;
    /**
     * Configuration block for specifying a client authentication. See clientAuthentication Argument Reference below.
     */
    clientAuthentication?: pulumi.Input<inputs.msk.ClusterClientAuthentication>;
    /**
     * Name of the MSK cluster.
     */
    clusterName?: pulumi.Input<string>;
    /**
     * Configuration block for specifying an MSK Configuration to attach to Kafka brokers. See configurationInfo Argument Reference below.
     */
    configurationInfo?: pulumi.Input<inputs.msk.ClusterConfigurationInfo>;
    /**
     * Configuration block for specifying encryption. See encryptionInfo Argument Reference below.
     */
    encryptionInfo?: pulumi.Input<inputs.msk.ClusterEncryptionInfo>;
    /**
     * Specify the desired enhanced MSK CloudWatch monitoring level. See [Monitoring Amazon MSK with Amazon CloudWatch](https://docs.aws.amazon.com/msk/latest/developerguide/monitoring.html)
     */
    enhancedMonitoring?: pulumi.Input<string>;
    /**
     * Specify the desired Kafka software version.
     */
    kafkaVersion: pulumi.Input<string>;
    /**
     * Configuration block for streaming broker logs to Cloudwatch/S3/Kinesis Firehose. See loggingInfo Argument Reference below.
     */
    loggingInfo?: pulumi.Input<inputs.msk.ClusterLoggingInfo>;
    /**
     * The desired total number of broker nodes in the kafka cluster.  It must be a multiple of the number of specified client subnets.
     */
    numberOfBrokerNodes: pulumi.Input<number>;
    /**
     * Configuration block for JMX and Node monitoring for the MSK cluster. See openMonitoring Argument Reference below.
     */
    openMonitoring?: pulumi.Input<inputs.msk.ClusterOpenMonitoring>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Controls storage mode for supported storage tiers. Valid values are: `LOCAL` or `TIERED`.
     */
    storageMode?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
