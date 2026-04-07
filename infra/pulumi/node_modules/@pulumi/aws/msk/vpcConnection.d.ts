import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Managed Streaming for Kafka VPC Connection.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.msk.VpcConnection("test", {
 *     authentication: "SASL_IAM",
 *     targetClusterArn: "aws_msk_cluster.arn",
 *     vpcId: testAwsVpc.id,
 *     clientSubnets: testAwsSubnet.map(__item => __item.id),
 *     securityGroups: [testAwsSecurityGroup.id],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import MSK configurations using the configuration ARN. For example:
 *
 * ```sh
 * $ pulumi import aws:msk/vpcConnection:VpcConnection example arn:aws:kafka:eu-west-2:123456789012:vpc-connection/123456789012/example/38173259-79cd-4ee8-87f3-682ea6023f48-2
 * ```
 */
export declare class VpcConnection extends pulumi.CustomResource {
    /**
     * Get an existing VpcConnection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcConnectionState, opts?: pulumi.CustomResourceOptions): VpcConnection;
    /**
     * Returns true if the given object is an instance of VpcConnection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcConnection;
    /**
     * Amazon Resource Name (ARN) of the VPC connection.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The authentication type for the client VPC connection. Specify one of these auth type strings: SASL_IAM, SASL_SCRAM, or TLS.
     */
    readonly authentication: pulumi.Output<string>;
    /**
     * The list of subnets in the client VPC to connect to.
     */
    readonly clientSubnets: pulumi.Output<string[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The security groups to attach to the ENIs for the broker nodes.
     */
    readonly securityGroups: pulumi.Output<string[]>;
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
     * The Amazon Resource Name (ARN) of the cluster.
     */
    readonly targetClusterArn: pulumi.Output<string>;
    /**
     * The VPC ID of the remote client.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a VpcConnection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcConnectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcConnection resources.
 */
export interface VpcConnectionState {
    /**
     * Amazon Resource Name (ARN) of the VPC connection.
     */
    arn?: pulumi.Input<string>;
    /**
     * The authentication type for the client VPC connection. Specify one of these auth type strings: SASL_IAM, SASL_SCRAM, or TLS.
     */
    authentication?: pulumi.Input<string>;
    /**
     * The list of subnets in the client VPC to connect to.
     */
    clientSubnets?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The security groups to attach to the ENIs for the broker nodes.
     */
    securityGroups?: pulumi.Input<pulumi.Input<string>[]>;
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
     * The Amazon Resource Name (ARN) of the cluster.
     */
    targetClusterArn?: pulumi.Input<string>;
    /**
     * The VPC ID of the remote client.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VpcConnection resource.
 */
export interface VpcConnectionArgs {
    /**
     * The authentication type for the client VPC connection. Specify one of these auth type strings: SASL_IAM, SASL_SCRAM, or TLS.
     */
    authentication: pulumi.Input<string>;
    /**
     * The list of subnets in the client VPC to connect to.
     */
    clientSubnets: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The security groups to attach to the ENIs for the broker nodes.
     */
    securityGroups: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The Amazon Resource Name (ARN) of the cluster.
     */
    targetClusterArn: pulumi.Input<string>;
    /**
     * The VPC ID of the remote client.
     */
    vpcId: pulumi.Input<string>;
}
