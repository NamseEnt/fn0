import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an AWS DataSync Agent deployed on premises.
 *
 * > **NOTE:** One of `activationKey` or `ipAddress` must be provided for resource creation (agent activation). Neither is required for resource import. If using `ipAddress`, this provider must be able to make an HTTP (port 80) GET request to the specified IP address from where it is running. The agent will turn off that HTTP server after activation.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.datasync.Agent("example", {
 *     ipAddress: "1.2.3.4",
 *     name: "example",
 * });
 * ```
 *
 * ### With VPC Endpoints
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegion({});
 * const exampleVpcEndpoint = new aws.ec2.VpcEndpoint("example", {
 *     serviceName: current.then(current => `com.amazonaws.${current.region}.datasync`),
 *     vpcId: exampleAwsVpc.id,
 *     securityGroupIds: [exampleAwsSecurityGroup.id],
 *     subnetIds: [exampleAwsSubnet.id],
 *     vpcEndpointType: "Interface",
 * });
 * const example = aws.ec2.getNetworkInterfaceOutput({
 *     id: exampleVpcEndpoint.networkInterfaceIds[0],
 * });
 * const exampleAgent = new aws.datasync.Agent("example", {
 *     ipAddress: "1.2.3.4",
 *     securityGroupArns: [exampleAwsSecurityGroup.arn],
 *     subnetArns: [exampleAwsSubnet.arn],
 *     vpcEndpointId: exampleVpcEndpoint.id,
 *     privateLinkEndpoint: example.apply(example => example.privateIp),
 *     name: "example",
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the DataSync agent.
 *
 * Using `pulumi import`, import `aws_datasync_agent` using the DataSync Agent Amazon Resource Name (ARN). For example:
 *
 * console
 *
 * % pulumi import aws_datasync_agent.example arn:aws:datasync:us-east-1:123456789012:agent/agent-12345678901234567
 */
export declare class Agent extends pulumi.CustomResource {
    /**
     * Get an existing Agent resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentState, opts?: pulumi.CustomResourceOptions): Agent;
    /**
     * Returns true if the given object is an instance of Agent.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Agent;
    /**
     * DataSync Agent activation key during resource creation. Conflicts with `ipAddress`. If an `ipAddress` is provided instead, the provider will retrieve the `activationKey` as part of the resource creation.
     */
    readonly activationKey: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the DataSync Agent.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * DataSync Agent IP address to retrieve activation key during resource creation. Conflicts with `activationKey`. DataSync Agent must be accessible on port 80 from where the provider is running.
     */
    readonly ipAddress: pulumi.Output<string>;
    /**
     * Name of the DataSync Agent.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The IP address of the VPC endpoint the agent should connect to when retrieving an activation key during resource creation. Conflicts with `activationKey`.
     */
    readonly privateLinkEndpoint: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ARNs of the security groups used to protect your data transfer task subnets.
     */
    readonly securityGroupArns: pulumi.Output<string[] | undefined>;
    /**
     * The Amazon Resource Names (ARNs) of the subnets in which DataSync will create elastic network interfaces for each data transfer task.
     */
    readonly subnetArns: pulumi.Output<string[] | undefined>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Agent. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The ID of the VPC (virtual private cloud) endpoint that the agent has access to.
     */
    readonly vpcEndpointId: pulumi.Output<string | undefined>;
    /**
     * Create a Agent resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: AgentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Agent resources.
 */
export interface AgentState {
    /**
     * DataSync Agent activation key during resource creation. Conflicts with `ipAddress`. If an `ipAddress` is provided instead, the provider will retrieve the `activationKey` as part of the resource creation.
     */
    activationKey?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the DataSync Agent.
     */
    arn?: pulumi.Input<string>;
    /**
     * DataSync Agent IP address to retrieve activation key during resource creation. Conflicts with `activationKey`. DataSync Agent must be accessible on port 80 from where the provider is running.
     */
    ipAddress?: pulumi.Input<string>;
    /**
     * Name of the DataSync Agent.
     */
    name?: pulumi.Input<string>;
    /**
     * The IP address of the VPC endpoint the agent should connect to when retrieving an activation key during resource creation. Conflicts with `activationKey`.
     */
    privateLinkEndpoint?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARNs of the security groups used to protect your data transfer task subnets.
     */
    securityGroupArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The Amazon Resource Names (ARNs) of the subnets in which DataSync will create elastic network interfaces for each data transfer task.
     */
    subnetArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Agent. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The ID of the VPC (virtual private cloud) endpoint that the agent has access to.
     */
    vpcEndpointId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Agent resource.
 */
export interface AgentArgs {
    /**
     * DataSync Agent activation key during resource creation. Conflicts with `ipAddress`. If an `ipAddress` is provided instead, the provider will retrieve the `activationKey` as part of the resource creation.
     */
    activationKey?: pulumi.Input<string>;
    /**
     * DataSync Agent IP address to retrieve activation key during resource creation. Conflicts with `activationKey`. DataSync Agent must be accessible on port 80 from where the provider is running.
     */
    ipAddress?: pulumi.Input<string>;
    /**
     * Name of the DataSync Agent.
     */
    name?: pulumi.Input<string>;
    /**
     * The IP address of the VPC endpoint the agent should connect to when retrieving an activation key during resource creation. Conflicts with `activationKey`.
     */
    privateLinkEndpoint?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARNs of the security groups used to protect your data transfer task subnets.
     */
    securityGroupArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The Amazon Resource Names (ARNs) of the subnets in which DataSync will create elastic network interfaces for each data transfer task.
     */
    subnetArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value pairs of resource tags to assign to the DataSync Agent. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the VPC (virtual private cloud) endpoint that the agent has access to.
     */
    vpcEndpointId?: pulumi.Input<string>;
}
