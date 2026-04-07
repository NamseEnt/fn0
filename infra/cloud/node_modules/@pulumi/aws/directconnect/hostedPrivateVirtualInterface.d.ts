import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Direct Connect hosted private virtual interface resource. This resource represents the allocator's side of the hosted virtual interface.
 * A hosted virtual interface is a virtual interface that is owned by another AWS account.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const foo = new aws.directconnect.HostedPrivateVirtualInterface("foo", {
 *     connectionId: "dxcon-zzzzzzzz",
 *     name: "vif-foo",
 *     vlan: 4094,
 *     addressFamily: "ipv4",
 *     bgpAsn: 65352,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Direct Connect hosted private virtual interfaces using the VIF `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:directconnect/hostedPrivateVirtualInterface:HostedPrivateVirtualInterface test dxvif-33cc44dd
 * ```
 */
export declare class HostedPrivateVirtualInterface extends pulumi.CustomResource {
    /**
     * Get an existing HostedPrivateVirtualInterface resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: HostedPrivateVirtualInterfaceState, opts?: pulumi.CustomResourceOptions): HostedPrivateVirtualInterface;
    /**
     * Returns true if the given object is an instance of HostedPrivateVirtualInterface.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is HostedPrivateVirtualInterface;
    /**
     * The address family for the BGP peer. `ipv4 ` or `ipv6`.
     */
    readonly addressFamily: pulumi.Output<string>;
    /**
     * The IPv4 CIDR address to use to send traffic to Amazon. Required for IPv4 BGP peers.
     */
    readonly amazonAddress: pulumi.Output<string>;
    readonly amazonSideAsn: pulumi.Output<string>;
    /**
     * The ARN of the virtual interface.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The Direct Connect endpoint on which the virtual interface terminates.
     */
    readonly awsDevice: pulumi.Output<string>;
    /**
     * The autonomous system (AS) number for Border Gateway Protocol (BGP) configuration.
     */
    readonly bgpAsn: pulumi.Output<number>;
    /**
     * The authentication key for BGP configuration.
     */
    readonly bgpAuthKey: pulumi.Output<string>;
    /**
     * The ID of the Direct Connect connection (or LAG) on which to create the virtual interface.
     */
    readonly connectionId: pulumi.Output<string>;
    /**
     * The IPv4 CIDR destination address to which Amazon should send traffic. Required for IPv4 BGP peers.
     */
    readonly customerAddress: pulumi.Output<string>;
    /**
     * Indicates whether jumbo frames (9001 MTU) are supported.
     */
    readonly jumboFrameCapable: pulumi.Output<boolean>;
    /**
     * The maximum transmission unit (MTU) is the size, in bytes, of the largest permissible packet that can be passed over the connection. The MTU of a virtual private interface can be either `1500` or `9001` (jumbo frames). Default is `1500`.
     */
    readonly mtu: pulumi.Output<number | undefined>;
    /**
     * The name for the virtual interface.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The AWS account that will own the new virtual interface.
     */
    readonly ownerAccountId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The VLAN ID.
     */
    readonly vlan: pulumi.Output<number>;
    /**
     * Create a HostedPrivateVirtualInterface resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: HostedPrivateVirtualInterfaceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering HostedPrivateVirtualInterface resources.
 */
export interface HostedPrivateVirtualInterfaceState {
    /**
     * The address family for the BGP peer. `ipv4 ` or `ipv6`.
     */
    addressFamily?: pulumi.Input<string>;
    /**
     * The IPv4 CIDR address to use to send traffic to Amazon. Required for IPv4 BGP peers.
     */
    amazonAddress?: pulumi.Input<string>;
    amazonSideAsn?: pulumi.Input<string>;
    /**
     * The ARN of the virtual interface.
     */
    arn?: pulumi.Input<string>;
    /**
     * The Direct Connect endpoint on which the virtual interface terminates.
     */
    awsDevice?: pulumi.Input<string>;
    /**
     * The autonomous system (AS) number for Border Gateway Protocol (BGP) configuration.
     */
    bgpAsn?: pulumi.Input<number>;
    /**
     * The authentication key for BGP configuration.
     */
    bgpAuthKey?: pulumi.Input<string>;
    /**
     * The ID of the Direct Connect connection (or LAG) on which to create the virtual interface.
     */
    connectionId?: pulumi.Input<string>;
    /**
     * The IPv4 CIDR destination address to which Amazon should send traffic. Required for IPv4 BGP peers.
     */
    customerAddress?: pulumi.Input<string>;
    /**
     * Indicates whether jumbo frames (9001 MTU) are supported.
     */
    jumboFrameCapable?: pulumi.Input<boolean>;
    /**
     * The maximum transmission unit (MTU) is the size, in bytes, of the largest permissible packet that can be passed over the connection. The MTU of a virtual private interface can be either `1500` or `9001` (jumbo frames). Default is `1500`.
     */
    mtu?: pulumi.Input<number>;
    /**
     * The name for the virtual interface.
     */
    name?: pulumi.Input<string>;
    /**
     * The AWS account that will own the new virtual interface.
     */
    ownerAccountId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The VLAN ID.
     */
    vlan?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a HostedPrivateVirtualInterface resource.
 */
export interface HostedPrivateVirtualInterfaceArgs {
    /**
     * The address family for the BGP peer. `ipv4 ` or `ipv6`.
     */
    addressFamily: pulumi.Input<string>;
    /**
     * The IPv4 CIDR address to use to send traffic to Amazon. Required for IPv4 BGP peers.
     */
    amazonAddress?: pulumi.Input<string>;
    /**
     * The autonomous system (AS) number for Border Gateway Protocol (BGP) configuration.
     */
    bgpAsn: pulumi.Input<number>;
    /**
     * The authentication key for BGP configuration.
     */
    bgpAuthKey?: pulumi.Input<string>;
    /**
     * The ID of the Direct Connect connection (or LAG) on which to create the virtual interface.
     */
    connectionId: pulumi.Input<string>;
    /**
     * The IPv4 CIDR destination address to which Amazon should send traffic. Required for IPv4 BGP peers.
     */
    customerAddress?: pulumi.Input<string>;
    /**
     * The maximum transmission unit (MTU) is the size, in bytes, of the largest permissible packet that can be passed over the connection. The MTU of a virtual private interface can be either `1500` or `9001` (jumbo frames). Default is `1500`.
     */
    mtu?: pulumi.Input<number>;
    /**
     * The name for the virtual interface.
     */
    name?: pulumi.Input<string>;
    /**
     * The AWS account that will own the new virtual interface.
     */
    ownerAccountId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The VLAN ID.
     */
    vlan: pulumi.Input<number>;
}
