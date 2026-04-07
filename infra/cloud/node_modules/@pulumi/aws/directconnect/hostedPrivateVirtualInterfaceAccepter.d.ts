import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage the accepter's side of a Direct Connect hosted private virtual interface.
 * This resource accepts ownership of a private virtual interface created by another AWS account.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const accepter = aws.getCallerIdentity({});
 * // Accepter's side of the VIF.
 * const vpnGw = new aws.ec2.VpnGateway("vpn_gw", {});
 * // Creator's side of the VIF
 * const creator = new aws.directconnect.HostedPrivateVirtualInterface("creator", {
 *     connectionId: "dxcon-zzzzzzzz",
 *     ownerAccountId: accepter.then(accepter => accepter.accountId),
 *     name: "vif-foo",
 *     vlan: 4094,
 *     addressFamily: "ipv4",
 *     bgpAsn: 65352,
 * }, {
 *     dependsOn: [vpnGw],
 * });
 * const accepterHostedPrivateVirtualInterfaceAccepter = new aws.directconnect.HostedPrivateVirtualInterfaceAccepter("accepter", {
 *     virtualInterfaceId: creator.id,
 *     vpnGatewayId: vpnGw.id,
 *     tags: {
 *         Side: "Accepter",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Direct Connect hosted private virtual interfaces using the VIF `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:directconnect/hostedPrivateVirtualInterfaceAccepter:HostedPrivateVirtualInterfaceAccepter test dxvif-33cc44dd
 * ```
 */
export declare class HostedPrivateVirtualInterfaceAccepter extends pulumi.CustomResource {
    /**
     * Get an existing HostedPrivateVirtualInterfaceAccepter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: HostedPrivateVirtualInterfaceAccepterState, opts?: pulumi.CustomResourceOptions): HostedPrivateVirtualInterfaceAccepter;
    /**
     * Returns true if the given object is an instance of HostedPrivateVirtualInterfaceAccepter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is HostedPrivateVirtualInterfaceAccepter;
    /**
     * The ARN of the virtual interface.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The ID of the Direct Connect gateway to which to connect the virtual interface.
     */
    readonly dxGatewayId: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * The ID of the Direct Connect virtual interface to accept.
     */
    readonly virtualInterfaceId: pulumi.Output<string>;
    /**
     * The ID of the virtual private gateway to which to connect the virtual interface.
     */
    readonly vpnGatewayId: pulumi.Output<string | undefined>;
    /**
     * Create a HostedPrivateVirtualInterfaceAccepter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: HostedPrivateVirtualInterfaceAccepterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering HostedPrivateVirtualInterfaceAccepter resources.
 */
export interface HostedPrivateVirtualInterfaceAccepterState {
    /**
     * The ARN of the virtual interface.
     */
    arn?: pulumi.Input<string>;
    /**
     * The ID of the Direct Connect gateway to which to connect the virtual interface.
     */
    dxGatewayId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
     * The ID of the Direct Connect virtual interface to accept.
     */
    virtualInterfaceId?: pulumi.Input<string>;
    /**
     * The ID of the virtual private gateway to which to connect the virtual interface.
     */
    vpnGatewayId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a HostedPrivateVirtualInterfaceAccepter resource.
 */
export interface HostedPrivateVirtualInterfaceAccepterArgs {
    /**
     * The ID of the Direct Connect gateway to which to connect the virtual interface.
     */
    dxGatewayId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the Direct Connect virtual interface to accept.
     */
    virtualInterfaceId: pulumi.Input<string>;
    /**
     * The ID of the virtual private gateway to which to connect the virtual interface.
     */
    vpnGatewayId?: pulumi.Input<string>;
}
