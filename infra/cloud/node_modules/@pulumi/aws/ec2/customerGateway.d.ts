import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a customer gateway inside a VPC. These objects can be connected to VPN gateways via VPN connections, and allow you to establish tunnels between your network and the VPC.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = new aws.ec2.CustomerGateway("main", {
 *     bgpAsn: "65000",
 *     ipAddress: "172.83.124.10",
 *     type: "ipsec.1",
 *     tags: {
 *         Name: "main-customer-gateway",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Customer Gateways using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/customerGateway:CustomerGateway main cgw-b4dc3961
 * ```
 */
export declare class CustomerGateway extends pulumi.CustomResource {
    /**
     * Get an existing CustomerGateway resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CustomerGatewayState, opts?: pulumi.CustomResourceOptions): CustomerGateway;
    /**
     * Returns true if the given object is an instance of CustomerGateway.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CustomerGateway;
    /**
     * The ARN of the customer gateway.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The gateway's Border Gateway Protocol (BGP) Autonomous System Number (ASN). Valid values are from  `1` to `2147483647`. Conflicts with `bgpAsnExtended`.
     */
    readonly bgpAsn: pulumi.Output<string | undefined>;
    /**
     * The gateway's Border Gateway Protocol (BGP) Autonomous System Number (ASN). Valid values are from  `2147483648` to `4294967295` Conflicts with `bgpAsn`.
     */
    readonly bgpAsnExtended: pulumi.Output<string | undefined>;
    /**
     * The Amazon Resource Name (ARN) for the customer gateway certificate.
     */
    readonly certificateArn: pulumi.Output<string | undefined>;
    /**
     * A name for the customer gateway device.
     */
    readonly deviceName: pulumi.Output<string | undefined>;
    /**
     * The IPv4 address for the customer gateway device's outside interface.
     */
    readonly ipAddress: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Tags to apply to the gateway. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The type of customer gateway. The only type AWS
     * supports at this time is "ipsec.1".
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a CustomerGateway resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CustomerGatewayArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CustomerGateway resources.
 */
export interface CustomerGatewayState {
    /**
     * The ARN of the customer gateway.
     */
    arn?: pulumi.Input<string>;
    /**
     * The gateway's Border Gateway Protocol (BGP) Autonomous System Number (ASN). Valid values are from  `1` to `2147483647`. Conflicts with `bgpAsnExtended`.
     */
    bgpAsn?: pulumi.Input<string>;
    /**
     * The gateway's Border Gateway Protocol (BGP) Autonomous System Number (ASN). Valid values are from  `2147483648` to `4294967295` Conflicts with `bgpAsn`.
     */
    bgpAsnExtended?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for the customer gateway certificate.
     */
    certificateArn?: pulumi.Input<string>;
    /**
     * A name for the customer gateway device.
     */
    deviceName?: pulumi.Input<string>;
    /**
     * The IPv4 address for the customer gateway device's outside interface.
     */
    ipAddress?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the gateway. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The type of customer gateway. The only type AWS
     * supports at this time is "ipsec.1".
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a CustomerGateway resource.
 */
export interface CustomerGatewayArgs {
    /**
     * The gateway's Border Gateway Protocol (BGP) Autonomous System Number (ASN). Valid values are from  `1` to `2147483647`. Conflicts with `bgpAsnExtended`.
     */
    bgpAsn?: pulumi.Input<string>;
    /**
     * The gateway's Border Gateway Protocol (BGP) Autonomous System Number (ASN). Valid values are from  `2147483648` to `4294967295` Conflicts with `bgpAsn`.
     */
    bgpAsnExtended?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for the customer gateway certificate.
     */
    certificateArn?: pulumi.Input<string>;
    /**
     * A name for the customer gateway device.
     */
    deviceName?: pulumi.Input<string>;
    /**
     * The IPv4 address for the customer gateway device's outside interface.
     */
    ipAddress?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the gateway. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The type of customer gateway. The only type AWS
     * supports at this time is "ipsec.1".
     */
    type: pulumi.Input<string>;
}
