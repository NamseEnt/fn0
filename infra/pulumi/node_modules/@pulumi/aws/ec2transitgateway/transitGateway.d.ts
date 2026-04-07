import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an EC2 Transit Gateway.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2transitgateway.TransitGateway("example", {description: "example"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_ec2_transit_gateway` using the EC2 Transit Gateway identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2transitgateway/transitGateway:TransitGateway example tgw-12345678
 * ```
 */
export declare class TransitGateway extends pulumi.CustomResource {
    /**
     * Get an existing TransitGateway resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TransitGatewayState, opts?: pulumi.CustomResourceOptions): TransitGateway;
    /**
     * Returns true if the given object is an instance of TransitGateway.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TransitGateway;
    /**
     * Private Autonomous System Number (ASN) for the Amazon side of a BGP session. The range is `64512` to `65534` for 16-bit ASNs and `4200000000` to `4294967294` for 32-bit ASNs. Default value: `64512`.
     *
     * > **NOTE:** Modifying `amazonSideAsn` on a Transit Gateway with active BGP sessions is [not allowed](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_ModifyTransitGatewayOptions.html). You must first delete all Transit Gateway attachments that have BGP configured prior to modifying `amazonSideAsn`.
     */
    readonly amazonSideAsn: pulumi.Output<number | undefined>;
    /**
     * EC2 Transit Gateway Amazon Resource Name (ARN)
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Identifier of the default association route table
     */
    readonly associationDefaultRouteTableId: pulumi.Output<string>;
    /**
     * Whether resource attachment requests are automatically accepted. Valid values: `disable`, `enable`. Default value: `disable`.
     */
    readonly autoAcceptSharedAttachments: pulumi.Output<string | undefined>;
    /**
     * Whether resource attachments are automatically associated with the default association route table. Valid values: `disable`, `enable`. Default value: `enable`.
     */
    readonly defaultRouteTableAssociation: pulumi.Output<string | undefined>;
    /**
     * Whether resource attachments automatically propagate routes to the default propagation route table. Valid values: `disable`, `enable`. Default value: `enable`.
     */
    readonly defaultRouteTablePropagation: pulumi.Output<string | undefined>;
    /**
     * Description of the EC2 Transit Gateway.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Whether DNS support is enabled. Valid values: `disable`, `enable`. Default value: `enable`.
     */
    readonly dnsSupport: pulumi.Output<string | undefined>;
    /**
     * Whether Multicast support is enabled. Required to use `ec2TransitGatewayMulticastDomain`. Valid values: `disable`, `enable`. Default value: `disable`.
     */
    readonly multicastSupport: pulumi.Output<string | undefined>;
    /**
     * Identifier of the AWS account that owns the EC2 Transit Gateway
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * Identifier of the default propagation route table
     */
    readonly propagationDefaultRouteTableId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Whether Security Group Referencing Support is enabled. Valid values: `disable`, `enable`. Default value: `disable`.
     */
    readonly securityGroupReferencingSupport: pulumi.Output<string | undefined>;
    /**
     * Key-value tags for the EC2 Transit Gateway. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * One or more IPv4 or IPv6 CIDR blocks for the transit gateway. Must be a size /24 CIDR block or larger for IPv4, or a size /64 CIDR block or larger for IPv6.
     */
    readonly transitGatewayCidrBlocks: pulumi.Output<string[] | undefined>;
    /**
     * Whether VPN Equal Cost Multipath Protocol support is enabled. Valid values: `disable`, `enable`. Default value: `enable`.
     */
    readonly vpnEcmpSupport: pulumi.Output<string | undefined>;
    /**
     * Create a TransitGateway resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: TransitGatewayArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TransitGateway resources.
 */
export interface TransitGatewayState {
    /**
     * Private Autonomous System Number (ASN) for the Amazon side of a BGP session. The range is `64512` to `65534` for 16-bit ASNs and `4200000000` to `4294967294` for 32-bit ASNs. Default value: `64512`.
     *
     * > **NOTE:** Modifying `amazonSideAsn` on a Transit Gateway with active BGP sessions is [not allowed](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_ModifyTransitGatewayOptions.html). You must first delete all Transit Gateway attachments that have BGP configured prior to modifying `amazonSideAsn`.
     */
    amazonSideAsn?: pulumi.Input<number>;
    /**
     * EC2 Transit Gateway Amazon Resource Name (ARN)
     */
    arn?: pulumi.Input<string>;
    /**
     * Identifier of the default association route table
     */
    associationDefaultRouteTableId?: pulumi.Input<string>;
    /**
     * Whether resource attachment requests are automatically accepted. Valid values: `disable`, `enable`. Default value: `disable`.
     */
    autoAcceptSharedAttachments?: pulumi.Input<string>;
    /**
     * Whether resource attachments are automatically associated with the default association route table. Valid values: `disable`, `enable`. Default value: `enable`.
     */
    defaultRouteTableAssociation?: pulumi.Input<string>;
    /**
     * Whether resource attachments automatically propagate routes to the default propagation route table. Valid values: `disable`, `enable`. Default value: `enable`.
     */
    defaultRouteTablePropagation?: pulumi.Input<string>;
    /**
     * Description of the EC2 Transit Gateway.
     */
    description?: pulumi.Input<string>;
    /**
     * Whether DNS support is enabled. Valid values: `disable`, `enable`. Default value: `enable`.
     */
    dnsSupport?: pulumi.Input<string>;
    /**
     * Whether Multicast support is enabled. Required to use `ec2TransitGatewayMulticastDomain`. Valid values: `disable`, `enable`. Default value: `disable`.
     */
    multicastSupport?: pulumi.Input<string>;
    /**
     * Identifier of the AWS account that owns the EC2 Transit Gateway
     */
    ownerId?: pulumi.Input<string>;
    /**
     * Identifier of the default propagation route table
     */
    propagationDefaultRouteTableId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether Security Group Referencing Support is enabled. Valid values: `disable`, `enable`. Default value: `disable`.
     */
    securityGroupReferencingSupport?: pulumi.Input<string>;
    /**
     * Key-value tags for the EC2 Transit Gateway. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * One or more IPv4 or IPv6 CIDR blocks for the transit gateway. Must be a size /24 CIDR block or larger for IPv4, or a size /64 CIDR block or larger for IPv6.
     */
    transitGatewayCidrBlocks?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Whether VPN Equal Cost Multipath Protocol support is enabled. Valid values: `disable`, `enable`. Default value: `enable`.
     */
    vpnEcmpSupport?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TransitGateway resource.
 */
export interface TransitGatewayArgs {
    /**
     * Private Autonomous System Number (ASN) for the Amazon side of a BGP session. The range is `64512` to `65534` for 16-bit ASNs and `4200000000` to `4294967294` for 32-bit ASNs. Default value: `64512`.
     *
     * > **NOTE:** Modifying `amazonSideAsn` on a Transit Gateway with active BGP sessions is [not allowed](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_ModifyTransitGatewayOptions.html). You must first delete all Transit Gateway attachments that have BGP configured prior to modifying `amazonSideAsn`.
     */
    amazonSideAsn?: pulumi.Input<number>;
    /**
     * Whether resource attachment requests are automatically accepted. Valid values: `disable`, `enable`. Default value: `disable`.
     */
    autoAcceptSharedAttachments?: pulumi.Input<string>;
    /**
     * Whether resource attachments are automatically associated with the default association route table. Valid values: `disable`, `enable`. Default value: `enable`.
     */
    defaultRouteTableAssociation?: pulumi.Input<string>;
    /**
     * Whether resource attachments automatically propagate routes to the default propagation route table. Valid values: `disable`, `enable`. Default value: `enable`.
     */
    defaultRouteTablePropagation?: pulumi.Input<string>;
    /**
     * Description of the EC2 Transit Gateway.
     */
    description?: pulumi.Input<string>;
    /**
     * Whether DNS support is enabled. Valid values: `disable`, `enable`. Default value: `enable`.
     */
    dnsSupport?: pulumi.Input<string>;
    /**
     * Whether Multicast support is enabled. Required to use `ec2TransitGatewayMulticastDomain`. Valid values: `disable`, `enable`. Default value: `disable`.
     */
    multicastSupport?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether Security Group Referencing Support is enabled. Valid values: `disable`, `enable`. Default value: `disable`.
     */
    securityGroupReferencingSupport?: pulumi.Input<string>;
    /**
     * Key-value tags for the EC2 Transit Gateway. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * One or more IPv4 or IPv6 CIDR blocks for the transit gateway. Must be a size /24 CIDR block or larger for IPv4, or a size /64 CIDR block or larger for IPv6.
     */
    transitGatewayCidrBlocks?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Whether VPN Equal Cost Multipath Protocol support is enabled. Valid values: `disable`, `enable`. Default value: `enable`.
     */
    vpnEcmpSupport?: pulumi.Input<string>;
}
