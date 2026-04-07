import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage the accepter's side of a Direct Connect hosted public virtual interface.
 * This resource accepts ownership of a public virtual interface created by another AWS account.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const accepter = aws.getCallerIdentity({});
 * // Creator's side of the VIF
 * const creator = new aws.directconnect.HostedPublicVirtualInterface("creator", {
 *     connectionId: "dxcon-zzzzzzzz",
 *     ownerAccountId: accepter.then(accepter => accepter.accountId),
 *     name: "vif-foo",
 *     vlan: 4094,
 *     addressFamily: "ipv4",
 *     bgpAsn: 65352,
 *     customerAddress: "175.45.176.1/30",
 *     amazonAddress: "175.45.176.2/30",
 *     routeFilterPrefixes: [
 *         "210.52.109.0/24",
 *         "175.45.176.0/22",
 *     ],
 * });
 * // Accepter's side of the VIF.
 * const accepterHostedPublicVirtualInterfaceAccepter = new aws.directconnect.HostedPublicVirtualInterfaceAccepter("accepter", {
 *     virtualInterfaceId: creator.id,
 *     tags: {
 *         Side: "Accepter",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Direct Connect hosted public virtual interfaces using the VIF `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:directconnect/hostedPublicVirtualInterfaceAccepter:HostedPublicVirtualInterfaceAccepter test dxvif-33cc44dd
 * ```
 */
export declare class HostedPublicVirtualInterfaceAccepter extends pulumi.CustomResource {
    /**
     * Get an existing HostedPublicVirtualInterfaceAccepter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: HostedPublicVirtualInterfaceAccepterState, opts?: pulumi.CustomResourceOptions): HostedPublicVirtualInterfaceAccepter;
    /**
     * Returns true if the given object is an instance of HostedPublicVirtualInterfaceAccepter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is HostedPublicVirtualInterfaceAccepter;
    /**
     * The ARN of the virtual interface.
     */
    readonly arn: pulumi.Output<string>;
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
     * Create a HostedPublicVirtualInterfaceAccepter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: HostedPublicVirtualInterfaceAccepterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering HostedPublicVirtualInterfaceAccepter resources.
 */
export interface HostedPublicVirtualInterfaceAccepterState {
    /**
     * The ARN of the virtual interface.
     */
    arn?: pulumi.Input<string>;
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
}
/**
 * The set of arguments for constructing a HostedPublicVirtualInterfaceAccepter resource.
 */
export interface HostedPublicVirtualInterfaceAccepterArgs {
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
}
