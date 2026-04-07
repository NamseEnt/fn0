import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an EC2 Transit Gateway Connect Peer.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2transitgateway.Connect("example", {
 *     transportAttachmentId: exampleAwsEc2TransitGatewayVpcAttachment.id,
 *     transitGatewayId: exampleAwsEc2TransitGateway.id,
 * });
 * const exampleConnectPeer = new aws.ec2transitgateway.ConnectPeer("example", {
 *     peerAddress: "10.1.2.3",
 *     insideCidrBlocks: ["169.254.100.0/29"],
 *     transitGatewayAttachmentId: example.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_ec2_transit_gateway_connect_peer` using the EC2 Transit Gateway Connect Peer identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2transitgateway/connectPeer:ConnectPeer example tgw-connect-peer-12345678
 * ```
 */
export declare class ConnectPeer extends pulumi.CustomResource {
    /**
     * Get an existing ConnectPeer resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConnectPeerState, opts?: pulumi.CustomResourceOptions): ConnectPeer;
    /**
     * Returns true if the given object is an instance of ConnectPeer.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ConnectPeer;
    /**
     * EC2 Transit Gateway Connect Peer ARN
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The BGP ASN number assigned customer device. If not provided, it will use the same BGP ASN as is associated with Transit Gateway.
     */
    readonly bgpAsn: pulumi.Output<string>;
    /**
     * The IP address assigned to customer device, which is used as BGP IP address.
     */
    readonly bgpPeerAddress: pulumi.Output<string>;
    /**
     * The IP addresses assigned to Transit Gateway, which are used as BGP IP addresses.
     */
    readonly bgpTransitGatewayAddresses: pulumi.Output<string[]>;
    /**
     * The CIDR block that will be used for addressing within the tunnel. It must contain exactly one IPv4 CIDR block and up to one IPv6 CIDR block. The IPv4 CIDR block must be /29 size and must be within 169.254.0.0/16 range, with exception of: 169.254.0.0/29, 169.254.1.0/29, 169.254.2.0/29, 169.254.3.0/29, 169.254.4.0/29, 169.254.5.0/29, 169.254.169.248/29. The IPv6 CIDR block must be /125 size and must be within fd00::/8. The first IP from each CIDR block is assigned for customer gateway, the second and third is for Transit Gateway (An example: from range 169.254.100.0/29, .1 is assigned to customer gateway and .2 and .3 are assigned to Transit Gateway)
     */
    readonly insideCidrBlocks: pulumi.Output<string[]>;
    /**
     * The IP addressed assigned to customer device, which will be used as tunnel endpoint. It can be IPv4 or IPv6 address, but must be the same address family as `transitGatewayAddress`
     */
    readonly peerAddress: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value tags for the EC2 Transit Gateway Connect Peer. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The IP address assigned to Transit Gateway, which will be used as tunnel endpoint. This address must be from associated Transit Gateway CIDR block. The address must be from the same address family as `peerAddress`. If not set explicitly, it will be selected from associated Transit Gateway CIDR blocks
     */
    readonly transitGatewayAddress: pulumi.Output<string>;
    /**
     * The Transit Gateway Connect
     */
    readonly transitGatewayAttachmentId: pulumi.Output<string>;
    /**
     * Create a ConnectPeer resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConnectPeerArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ConnectPeer resources.
 */
export interface ConnectPeerState {
    /**
     * EC2 Transit Gateway Connect Peer ARN
     */
    arn?: pulumi.Input<string>;
    /**
     * The BGP ASN number assigned customer device. If not provided, it will use the same BGP ASN as is associated with Transit Gateway.
     */
    bgpAsn?: pulumi.Input<string>;
    /**
     * The IP address assigned to customer device, which is used as BGP IP address.
     */
    bgpPeerAddress?: pulumi.Input<string>;
    /**
     * The IP addresses assigned to Transit Gateway, which are used as BGP IP addresses.
     */
    bgpTransitGatewayAddresses?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The CIDR block that will be used for addressing within the tunnel. It must contain exactly one IPv4 CIDR block and up to one IPv6 CIDR block. The IPv4 CIDR block must be /29 size and must be within 169.254.0.0/16 range, with exception of: 169.254.0.0/29, 169.254.1.0/29, 169.254.2.0/29, 169.254.3.0/29, 169.254.4.0/29, 169.254.5.0/29, 169.254.169.248/29. The IPv6 CIDR block must be /125 size and must be within fd00::/8. The first IP from each CIDR block is assigned for customer gateway, the second and third is for Transit Gateway (An example: from range 169.254.100.0/29, .1 is assigned to customer gateway and .2 and .3 are assigned to Transit Gateway)
     */
    insideCidrBlocks?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The IP addressed assigned to customer device, which will be used as tunnel endpoint. It can be IPv4 or IPv6 address, but must be the same address family as `transitGatewayAddress`
     */
    peerAddress?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value tags for the EC2 Transit Gateway Connect Peer. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The IP address assigned to Transit Gateway, which will be used as tunnel endpoint. This address must be from associated Transit Gateway CIDR block. The address must be from the same address family as `peerAddress`. If not set explicitly, it will be selected from associated Transit Gateway CIDR blocks
     */
    transitGatewayAddress?: pulumi.Input<string>;
    /**
     * The Transit Gateway Connect
     */
    transitGatewayAttachmentId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ConnectPeer resource.
 */
export interface ConnectPeerArgs {
    /**
     * The BGP ASN number assigned customer device. If not provided, it will use the same BGP ASN as is associated with Transit Gateway.
     */
    bgpAsn?: pulumi.Input<string>;
    /**
     * The CIDR block that will be used for addressing within the tunnel. It must contain exactly one IPv4 CIDR block and up to one IPv6 CIDR block. The IPv4 CIDR block must be /29 size and must be within 169.254.0.0/16 range, with exception of: 169.254.0.0/29, 169.254.1.0/29, 169.254.2.0/29, 169.254.3.0/29, 169.254.4.0/29, 169.254.5.0/29, 169.254.169.248/29. The IPv6 CIDR block must be /125 size and must be within fd00::/8. The first IP from each CIDR block is assigned for customer gateway, the second and third is for Transit Gateway (An example: from range 169.254.100.0/29, .1 is assigned to customer gateway and .2 and .3 are assigned to Transit Gateway)
     */
    insideCidrBlocks: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The IP addressed assigned to customer device, which will be used as tunnel endpoint. It can be IPv4 or IPv6 address, but must be the same address family as `transitGatewayAddress`
     */
    peerAddress: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value tags for the EC2 Transit Gateway Connect Peer. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The IP address assigned to Transit Gateway, which will be used as tunnel endpoint. This address must be from associated Transit Gateway CIDR block. The address must be from the same address family as `peerAddress`. If not set explicitly, it will be selected from associated Transit Gateway CIDR blocks
     */
    transitGatewayAddress?: pulumi.Input<string>;
    /**
     * The Transit Gateway Connect
     */
    transitGatewayAttachmentId: pulumi.Input<string>;
}
