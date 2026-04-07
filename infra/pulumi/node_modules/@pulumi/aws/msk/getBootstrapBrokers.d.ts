import * as pulumi from "@pulumi/pulumi";
/**
 * Get a list of brokers that a client application can use to bootstrap.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.msk.getBootstrapBrokers({
 *     clusterArn: exampleAwsMskCluster.arn,
 * });
 * ```
 */
export declare function getBootstrapBrokers(args: GetBootstrapBrokersArgs, opts?: pulumi.InvokeOptions): Promise<GetBootstrapBrokersResult>;
/**
 * A collection of arguments for invoking getBootstrapBrokers.
 */
export interface GetBootstrapBrokersArgs {
    /**
     * ARN of the cluster the nodes belong to.
     */
    clusterArn: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getBootstrapBrokers.
 */
export interface GetBootstrapBrokersResult {
    /**
     * Comma separated list of one or more hostname:port pairs of kafka brokers suitable to bootstrap connectivity to the kafka cluster.
     */
    readonly bootstrapBrokers: string;
    /**
     * One or more DNS names (or IP addresses) and SASL IAM port pairs.
     */
    readonly bootstrapBrokersPublicSaslIam: string;
    /**
     * One or more DNS names (or IP addresses) and SASL SCRAM port pairs.
     */
    readonly bootstrapBrokersPublicSaslScram: string;
    /**
     * One or more DNS names (or IP addresses) and TLS port pairs.
     */
    readonly bootstrapBrokersPublicTls: string;
    /**
     * One or more DNS names (or IP addresses) and SASL IAM port pairs.
     */
    readonly bootstrapBrokersSaslIam: string;
    /**
     * One or more DNS names (or IP addresses) and SASL SCRAM port pairs.
     */
    readonly bootstrapBrokersSaslScram: string;
    /**
     * One or more DNS names (or IP addresses) and TLS port pairs.
     */
    readonly bootstrapBrokersTls: string;
    /**
     * A string containing one or more DNS names (or IP addresses) and SASL IAM port pairs for VPC connectivity.
     */
    readonly bootstrapBrokersVpcConnectivitySaslIam: string;
    /**
     * A string containing one or more DNS names (or IP addresses) and SASL SCRAM port pairs for VPC connectivity.
     */
    readonly bootstrapBrokersVpcConnectivitySaslScram: string;
    /**
     * A string containing one or more DNS names (or IP addresses) and TLS port pairs for VPC connectivity.
     */
    readonly bootstrapBrokersVpcConnectivityTls: string;
    readonly clusterArn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Get a list of brokers that a client application can use to bootstrap.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.msk.getBootstrapBrokers({
 *     clusterArn: exampleAwsMskCluster.arn,
 * });
 * ```
 */
export declare function getBootstrapBrokersOutput(args: GetBootstrapBrokersOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetBootstrapBrokersResult>;
/**
 * A collection of arguments for invoking getBootstrapBrokers.
 */
export interface GetBootstrapBrokersOutputArgs {
    /**
     * ARN of the cluster the nodes belong to.
     */
    clusterArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
