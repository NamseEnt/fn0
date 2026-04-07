import * as pulumi from "@pulumi/pulumi";
/**
 * Registers sources (network interfaces) with the transit gateway multicast group.
 * A multicast source is a network interface attached to a supported instance that sends multicast traffic.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2transitgateway.MulticastGroupSource("example", {
 *     groupIpAddress: "224.0.0.1",
 *     networkInterfaceId: exampleAwsNetworkInterface.id,
 *     transitGatewayMulticastDomainId: exampleAwsEc2TransitGatewayMulticastDomain.id,
 * });
 * ```
 */
export declare class MulticastGroupSource extends pulumi.CustomResource {
    /**
     * Get an existing MulticastGroupSource resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MulticastGroupSourceState, opts?: pulumi.CustomResourceOptions): MulticastGroupSource;
    /**
     * Returns true if the given object is an instance of MulticastGroupSource.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is MulticastGroupSource;
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
     * Create a MulticastGroupSource resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MulticastGroupSourceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering MulticastGroupSource resources.
 */
export interface MulticastGroupSourceState {
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
 * The set of arguments for constructing a MulticastGroupSource resource.
 */
export interface MulticastGroupSourceArgs {
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
