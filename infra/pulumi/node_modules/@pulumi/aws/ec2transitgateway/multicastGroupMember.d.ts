import * as pulumi from "@pulumi/pulumi";
/**
 * Registers members (network interfaces) with the transit gateway multicast group.
 * A member is a network interface associated with a supported EC2 instance that receives multicast traffic.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2transitgateway.MulticastGroupMember("example", {
 *     groupIpAddress: "224.0.0.1",
 *     networkInterfaceId: exampleAwsNetworkInterface.id,
 *     transitGatewayMulticastDomainId: exampleAwsEc2TransitGatewayMulticastDomain.id,
 * });
 * ```
 */
export declare class MulticastGroupMember extends pulumi.CustomResource {
    /**
     * Get an existing MulticastGroupMember resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MulticastGroupMemberState, opts?: pulumi.CustomResourceOptions): MulticastGroupMember;
    /**
     * Returns true if the given object is an instance of MulticastGroupMember.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is MulticastGroupMember;
    /**
     * The IP address assigned to the transit gateway multicast group.
     */
    readonly groupIpAddress: pulumi.Output<string>;
    /**
     * The group members' network interface ID to register with the transit gateway multicast group.
     */
    readonly networkInterfaceId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the transit gateway multicast domain.
     */
    readonly transitGatewayMulticastDomainId: pulumi.Output<string>;
    /**
     * Create a MulticastGroupMember resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MulticastGroupMemberArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering MulticastGroupMember resources.
 */
export interface MulticastGroupMemberState {
    /**
     * The IP address assigned to the transit gateway multicast group.
     */
    groupIpAddress?: pulumi.Input<string>;
    /**
     * The group members' network interface ID to register with the transit gateway multicast group.
     */
    networkInterfaceId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the transit gateway multicast domain.
     */
    transitGatewayMulticastDomainId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a MulticastGroupMember resource.
 */
export interface MulticastGroupMemberArgs {
    /**
     * The IP address assigned to the transit gateway multicast group.
     */
    groupIpAddress: pulumi.Input<string>;
    /**
     * The group members' network interface ID to register with the transit gateway multicast group.
     */
    networkInterfaceId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the transit gateway multicast domain.
     */
    transitGatewayMulticastDomainId: pulumi.Input<string>;
}
