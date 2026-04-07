import * as pulumi from "@pulumi/pulumi";
/**
 * Previews a CIDR from an IPAM address pool. Only works for private IPv4.
 *
 * ## Example Usage
 *
 * Basic usage:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegion({});
 * const exampleVpcIpam = new aws.ec2.VpcIpam("example", {operatingRegions: [{
 *     regionName: current.then(current => current.region),
 * }]});
 * const exampleVpcIpamPool = new aws.ec2.VpcIpamPool("example", {
 *     addressFamily: "ipv4",
 *     ipamScopeId: exampleVpcIpam.privateDefaultScopeId,
 *     locale: current.then(current => current.region),
 * });
 * const exampleVpcIpamPoolCidr = new aws.ec2.VpcIpamPoolCidr("example", {
 *     ipamPoolId: exampleVpcIpamPool.id,
 *     cidr: "172.20.0.0/16",
 * });
 * const example = new aws.ec2.VpcIpamPreviewNextCidr("example", {
 *     ipamPoolId: exampleVpcIpamPool.id,
 *     netmaskLength: 28,
 *     disallowedCidrs: ["172.2.0.0/32"],
 * }, {
 *     dependsOn: [exampleVpcIpamPoolCidr],
 * });
 * ```
 */
export declare class VpcIpamPreviewNextCidr extends pulumi.CustomResource {
    /**
     * Get an existing VpcIpamPreviewNextCidr resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcIpamPreviewNextCidrState, opts?: pulumi.CustomResourceOptions): VpcIpamPreviewNextCidr;
    /**
     * Returns true if the given object is an instance of VpcIpamPreviewNextCidr.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcIpamPreviewNextCidr;
    /**
     * The previewed CIDR from the pool.
     */
    readonly cidr: pulumi.Output<string>;
    /**
     * Exclude a particular CIDR range from being returned by the pool.
     */
    readonly disallowedCidrs: pulumi.Output<string[] | undefined>;
    /**
     * The ID of the pool to which you want to assign a CIDR.
     */
    readonly ipamPoolId: pulumi.Output<string>;
    /**
     * The netmask length of the CIDR you would like to preview from the IPAM pool.
     */
    readonly netmaskLength: pulumi.Output<number | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a VpcIpamPreviewNextCidr resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcIpamPreviewNextCidrArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcIpamPreviewNextCidr resources.
 */
export interface VpcIpamPreviewNextCidrState {
    /**
     * The previewed CIDR from the pool.
     */
    cidr?: pulumi.Input<string>;
    /**
     * Exclude a particular CIDR range from being returned by the pool.
     */
    disallowedCidrs?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of the pool to which you want to assign a CIDR.
     */
    ipamPoolId?: pulumi.Input<string>;
    /**
     * The netmask length of the CIDR you would like to preview from the IPAM pool.
     */
    netmaskLength?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VpcIpamPreviewNextCidr resource.
 */
export interface VpcIpamPreviewNextCidrArgs {
    /**
     * Exclude a particular CIDR range from being returned by the pool.
     */
    disallowedCidrs?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of the pool to which you want to assign a CIDR.
     */
    ipamPoolId: pulumi.Input<string>;
    /**
     * The netmask length of the CIDR you would like to preview from the IPAM pool.
     */
    netmaskLength?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
