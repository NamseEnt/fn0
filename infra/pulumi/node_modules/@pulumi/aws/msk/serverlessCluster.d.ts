import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an Amazon MSK Serverless cluster.
 *
 * > **Note:** To manage a _provisioned_ Amazon MSK cluster, use the `aws.msk.Cluster` resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.msk.ServerlessCluster("example", {
 *     clusterName: "Example",
 *     vpcConfigs: [{
 *         subnetIds: exampleAwsSubnet.map(__item => __item.id),
 *         securityGroupIds: [exampleAwsSecurityGroup.id],
 *     }],
 *     clientAuthentication: {
 *         sasl: {
 *             iam: {
 *                 enabled: true,
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import MSK serverless clusters using the cluster `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:msk/serverlessCluster:ServerlessCluster example arn:aws:kafka:us-west-2:123456789012:cluster/example/279c0212-d057-4dba-9aa9-1c4e5a25bfc7-3
 * ```
 */
export declare class ServerlessCluster extends pulumi.CustomResource {
    /**
     * Get an existing ServerlessCluster resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServerlessClusterState, opts?: pulumi.CustomResourceOptions): ServerlessCluster;
    /**
     * Returns true if the given object is an instance of ServerlessCluster.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ServerlessCluster;
    /**
     * The ARN of the serverless cluster.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * One or more DNS names (or IP addresses) and SASL IAM port pairs. For example, `boot-abcdefg.c2.kafka-serverless.eu-central-1.amazonaws.com:9098`. The resource sorts the list alphabetically. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    readonly bootstrapBrokersSaslIam: pulumi.Output<string>;
    /**
     * Specifies client authentication information for the serverless cluster. See below.
     */
    readonly clientAuthentication: pulumi.Output<outputs.msk.ServerlessClusterClientAuthentication>;
    /**
     * The name of the serverless cluster.
     */
    readonly clusterName: pulumi.Output<string>;
    /**
     * UUID of the serverless cluster, for use in IAM policies.
     */
    readonly clusterUuid: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * VPC configuration information. See below.
     */
    readonly vpcConfigs: pulumi.Output<outputs.msk.ServerlessClusterVpcConfig[]>;
    /**
     * Create a ServerlessCluster resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ServerlessClusterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ServerlessCluster resources.
 */
export interface ServerlessClusterState {
    /**
     * The ARN of the serverless cluster.
     */
    arn?: pulumi.Input<string>;
    /**
     * One or more DNS names (or IP addresses) and SASL IAM port pairs. For example, `boot-abcdefg.c2.kafka-serverless.eu-central-1.amazonaws.com:9098`. The resource sorts the list alphabetically. AWS may not always return all endpoints so the values may not be stable across applies.
     */
    bootstrapBrokersSaslIam?: pulumi.Input<string>;
    /**
     * Specifies client authentication information for the serverless cluster. See below.
     */
    clientAuthentication?: pulumi.Input<inputs.msk.ServerlessClusterClientAuthentication>;
    /**
     * The name of the serverless cluster.
     */
    clusterName?: pulumi.Input<string>;
    /**
     * UUID of the serverless cluster, for use in IAM policies.
     */
    clusterUuid?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
     * VPC configuration information. See below.
     */
    vpcConfigs?: pulumi.Input<pulumi.Input<inputs.msk.ServerlessClusterVpcConfig>[]>;
}
/**
 * The set of arguments for constructing a ServerlessCluster resource.
 */
export interface ServerlessClusterArgs {
    /**
     * Specifies client authentication information for the serverless cluster. See below.
     */
    clientAuthentication: pulumi.Input<inputs.msk.ServerlessClusterClientAuthentication>;
    /**
     * The name of the serverless cluster.
     */
    clusterName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * VPC configuration information. See below.
     */
    vpcConfigs: pulumi.Input<pulumi.Input<inputs.msk.ServerlessClusterVpcConfig>[]>;
}
