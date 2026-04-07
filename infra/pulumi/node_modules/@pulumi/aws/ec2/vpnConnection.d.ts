import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Site-to-Site VPN connection. A Site-to-Site VPN connection is an Internet Protocol security (IPsec) VPN connection between a VPC and an on-premises network.
 * Any new Site-to-Site VPN connection that you create is an [AWS VPN connection](https://docs.aws.amazon.com/vpn/latest/s2svpn/vpn-categories.html).
 *
 * > **Note:** The CIDR blocks in the arguments `tunnel1InsideCidr` and `tunnel2InsideCidr` must have a prefix of /30 and be a part of a specific range.
 * [Read more about this in the AWS documentation](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_VpnTunnelOptionsSpecification.html).
 *
 * ## Example Usage
 *
 * ### EC2 Transit Gateway
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2transitgateway.TransitGateway("example", {});
 * const exampleCustomerGateway = new aws.ec2.CustomerGateway("example", {
 *     bgpAsn: "65000",
 *     ipAddress: "172.0.0.1",
 *     type: "ipsec.1",
 * });
 * const exampleVpnConnection = new aws.ec2.VpnConnection("example", {
 *     customerGatewayId: exampleCustomerGateway.id,
 *     transitGatewayId: example.id,
 *     type: exampleCustomerGateway.type,
 * });
 * ```
 *
 * ### Virtual Private Gateway
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const vpc = new aws.ec2.Vpc("vpc", {cidrBlock: "10.0.0.0/16"});
 * const vpnGateway = new aws.ec2.VpnGateway("vpn_gateway", {vpcId: vpc.id});
 * const customerGateway = new aws.ec2.CustomerGateway("customer_gateway", {
 *     bgpAsn: "65000",
 *     ipAddress: "172.0.0.1",
 *     type: "ipsec.1",
 * });
 * const main = new aws.ec2.VpnConnection("main", {
 *     vpnGatewayId: vpnGateway.id,
 *     customerGatewayId: customerGateway.id,
 *     type: "ipsec.1",
 *     staticRoutesOnly: true,
 * });
 * ```
 *
 * ### AWS Site to Site Private VPN
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleGateway = new aws.directconnect.Gateway("example", {
 *     name: "example_ipsec_vpn_example",
 *     amazonSideAsn: "64512",
 * });
 * const exampleTransitGateway = new aws.ec2transitgateway.TransitGateway("example", {
 *     amazonSideAsn: 64513,
 *     description: "example_ipsec_vpn_example",
 *     transitGatewayCidrBlocks: ["10.0.0.0/24"],
 * });
 * const exampleCustomerGateway = new aws.ec2.CustomerGateway("example", {
 *     bgpAsn: "64514",
 *     ipAddress: "10.0.0.1",
 *     type: "ipsec.1",
 *     tags: {
 *         Name: "example_ipsec_vpn_example",
 *     },
 * });
 * const exampleGatewayAssociation = new aws.directconnect.GatewayAssociation("example", {
 *     dxGatewayId: exampleGateway.id,
 *     associatedGatewayId: exampleTransitGateway.id,
 *     allowedPrefixes: ["10.0.0.0/8"],
 * });
 * const example = aws.ec2transitgateway.getDirectConnectGatewayAttachmentOutput({
 *     transitGatewayId: exampleTransitGateway.id,
 *     dxGatewayId: exampleGateway.id,
 * });
 * const exampleVpnConnection = new aws.ec2.VpnConnection("example", {
 *     customerGatewayId: exampleCustomerGateway.id,
 *     outsideIpAddressType: "PrivateIpv4",
 *     transitGatewayId: exampleTransitGateway.id,
 *     transportTransitGatewayAttachmentId: example.apply(example => example.id),
 *     type: "ipsec.1",
 *     tags: {
 *         Name: "example_ipsec_vpn_example",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPN Connections using the VPN connection `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/vpnConnection:VpnConnection testvpnconnection vpn-40f41529
 * ```
 */
export declare class VpnConnection extends pulumi.CustomResource {
    /**
     * Get an existing VpnConnection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpnConnectionState, opts?: pulumi.CustomResourceOptions): VpnConnection;
    /**
     * Returns true if the given object is an instance of VpnConnection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpnConnection;
    /**
     * Amazon Resource Name (ARN) of the VPN Connection.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The ARN of the core network.
     */
    readonly coreNetworkArn: pulumi.Output<string>;
    /**
     * The ARN of the core network attachment.
     */
    readonly coreNetworkAttachmentArn: pulumi.Output<string>;
    /**
     * The configuration information for the VPN connection's customer gateway (in the native XML format).
     */
    readonly customerGatewayConfiguration: pulumi.Output<string>;
    /**
     * The ID of the customer gateway.
     */
    readonly customerGatewayId: pulumi.Output<string>;
    /**
     * Indicate whether to enable acceleration for the VPN connection. Supports only EC2 Transit Gateway.
     */
    readonly enableAcceleration: pulumi.Output<boolean>;
    /**
     * The IPv4 CIDR on the customer gateway (on-premises) side of the VPN connection.
     */
    readonly localIpv4NetworkCidr: pulumi.Output<string>;
    /**
     * The IPv6 CIDR on the customer gateway (on-premises) side of the VPN connection.
     */
    readonly localIpv6NetworkCidr: pulumi.Output<string>;
    /**
     * Indicates if a Public S2S VPN or Private S2S VPN over AWS Direct Connect. Valid values are `PublicIpv4 | PrivateIpv4`
     */
    readonly outsideIpAddressType: pulumi.Output<string>;
    /**
     * ARN of the Secrets Manager secret storing the pre-shared key(s) for the VPN connection. Note that even if it returns a valid Secrets Manager ARN, the pre-shared key(s) will not be stored in Secrets Manager unless the `presharedKeyStorage` argument is set to `SecretsManager`.
     */
    readonly presharedKeyArn: pulumi.Output<string>;
    /**
     * Storage mode for the pre-shared key (PSK). Valid values are `Standard` (stored in the Site-to-Site VPN service) or `SecretsManager` (stored in AWS Secrets Manager).
     */
    readonly presharedKeyStorage: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The IPv4 CIDR on the AWS side of the VPN connection.
     */
    readonly remoteIpv4NetworkCidr: pulumi.Output<string>;
    /**
     * The IPv6 CIDR on the AWS side of the VPN connection.
     */
    readonly remoteIpv6NetworkCidr: pulumi.Output<string>;
    /**
     * The static routes associated with the VPN connection. Detailed below.
     */
    readonly routes: pulumi.Output<outputs.ec2.VpnConnectionRoute[]>;
    /**
     * Whether the VPN connection uses static routes exclusively. Static routes must be used for devices that don't support BGP.
     */
    readonly staticRoutesOnly: pulumi.Output<boolean>;
    /**
     * Tags to apply to the connection. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * When associated with an EC2 Transit Gateway (`transitGatewayId` argument), the attachment ID. See also the `aws.ec2.Tag` resource for tagging the EC2 Transit Gateway VPN Attachment.
     */
    readonly transitGatewayAttachmentId: pulumi.Output<string>;
    /**
     * The ID of the EC2 Transit Gateway.
     */
    readonly transitGatewayId: pulumi.Output<string | undefined>;
    /**
     * . The attachment ID of the Transit Gateway attachment to Direct Connect Gateway. The ID is obtained through a data source only.
     */
    readonly transportTransitGatewayAttachmentId: pulumi.Output<string | undefined>;
    /**
     * The public IP address of the first VPN tunnel.
     */
    readonly tunnel1Address: pulumi.Output<string>;
    /**
     * The bgp asn number of the first VPN tunnel.
     */
    readonly tunnel1BgpAsn: pulumi.Output<string>;
    /**
     * The bgp holdtime of the first VPN tunnel.
     */
    readonly tunnel1BgpHoldtime: pulumi.Output<number>;
    /**
     * The RFC 6890 link-local address of the first VPN tunnel (Customer Gateway Side).
     */
    readonly tunnel1CgwInsideAddress: pulumi.Output<string>;
    /**
     * The action to take after DPD timeout occurs for the first VPN tunnel. Specify restart to restart the IKE initiation. Specify clear to end the IKE session. Valid values are `clear | none | restart`.
     */
    readonly tunnel1DpdTimeoutAction: pulumi.Output<string | undefined>;
    /**
     * The number of seconds after which a DPD timeout occurs for the first VPN tunnel. Valid value is equal or higher than `30`.
     */
    readonly tunnel1DpdTimeoutSeconds: pulumi.Output<number | undefined>;
    /**
     * Turn on or off tunnel endpoint lifecycle control feature for the first VPN tunnel. Valid values are `true | false`.
     */
    readonly tunnel1EnableTunnelLifecycleControl: pulumi.Output<boolean | undefined>;
    /**
     * The IKE versions that are permitted for the first VPN tunnel. Valid values are `ikev1 | ikev2`.
     */
    readonly tunnel1IkeVersions: pulumi.Output<string[] | undefined>;
    /**
     * The CIDR block of the inside IP addresses for the first VPN tunnel. Valid value is a size /30 CIDR block from the 169.254.0.0/16 range.
     */
    readonly tunnel1InsideCidr: pulumi.Output<string>;
    /**
     * The range of inside IPv6 addresses for the first VPN tunnel. Supports only EC2 Transit Gateway. Valid value is a size /126 CIDR block from the local fd00::/8 range.
     */
    readonly tunnel1InsideIpv6Cidr: pulumi.Output<string>;
    /**
     * Options for logging VPN tunnel activity. See Log Options below for more details.
     */
    readonly tunnel1LogOptions: pulumi.Output<outputs.ec2.VpnConnectionTunnel1LogOptions>;
    /**
     * List of one or more Diffie-Hellman group numbers that are permitted for the first VPN tunnel for phase 1 IKE negotiations. Valid values are ` 2 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24`.
     */
    readonly tunnel1Phase1DhGroupNumbers: pulumi.Output<number[] | undefined>;
    /**
     * List of one or more encryption algorithms that are permitted for the first VPN tunnel for phase 1 IKE negotiations. Valid values are `AES128 | AES256 | AES128-GCM-16 | AES256-GCM-16`.
     */
    readonly tunnel1Phase1EncryptionAlgorithms: pulumi.Output<string[] | undefined>;
    /**
     * One or more integrity algorithms that are permitted for the first VPN tunnel for phase 1 IKE negotiations. Valid values are `SHA1 | SHA2-256 | SHA2-384 | SHA2-512`.
     */
    readonly tunnel1Phase1IntegrityAlgorithms: pulumi.Output<string[] | undefined>;
    /**
     * The lifetime for phase 1 of the IKE negotiation for the first VPN tunnel, in seconds. Valid value is between `900` and `28800`.
     */
    readonly tunnel1Phase1LifetimeSeconds: pulumi.Output<number | undefined>;
    /**
     * List of one or more Diffie-Hellman group numbers that are permitted for the first VPN tunnel for phase 2 IKE negotiations. Valid values are `2 | 5 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24`.
     */
    readonly tunnel1Phase2DhGroupNumbers: pulumi.Output<number[] | undefined>;
    /**
     * List of one or more encryption algorithms that are permitted for the first VPN tunnel for phase 2 IKE negotiations. Valid values are `AES128 | AES256 | AES128-GCM-16 | AES256-GCM-16`.
     */
    readonly tunnel1Phase2EncryptionAlgorithms: pulumi.Output<string[] | undefined>;
    /**
     * List of one or more integrity algorithms that are permitted for the first VPN tunnel for phase 2 IKE negotiations. Valid values are `SHA1 | SHA2-256 | SHA2-384 | SHA2-512`.
     */
    readonly tunnel1Phase2IntegrityAlgorithms: pulumi.Output<string[] | undefined>;
    /**
     * The lifetime for phase 2 of the IKE negotiation for the first VPN tunnel, in seconds. Valid value is between `900` and `3600`.
     */
    readonly tunnel1Phase2LifetimeSeconds: pulumi.Output<number | undefined>;
    /**
     * The preshared key of the first VPN tunnel. The preshared key must be between 8 and 64 characters in length and cannot start with zero(0). Allowed characters are alphanumeric characters, periods(.) and underscores(_).
     */
    readonly tunnel1PresharedKey: pulumi.Output<string>;
    /**
     * The percentage of the rekey window for the first VPN tunnel (determined by `tunnel1RekeyMarginTimeSeconds`) during which the rekey time is randomly selected. Valid value is between `0` and `100`.
     */
    readonly tunnel1RekeyFuzzPercentage: pulumi.Output<number | undefined>;
    /**
     * The margin time, in seconds, before the phase 2 lifetime expires, during which the AWS side of the first VPN connection performs an IKE rekey. The exact time of the rekey is randomly selected based on the value for `tunnel1RekeyFuzzPercentage`. Valid value is between `60` and half of `tunnel1Phase2LifetimeSeconds`.
     */
    readonly tunnel1RekeyMarginTimeSeconds: pulumi.Output<number | undefined>;
    /**
     * The number of packets in an IKE replay window for the first VPN tunnel. Valid value is between `64` and `2048`.
     */
    readonly tunnel1ReplayWindowSize: pulumi.Output<number | undefined>;
    /**
     * The action to take when the establishing the tunnel for the first VPN connection. By default, your customer gateway device must initiate the IKE negotiation and bring up the tunnel. Specify start for AWS to initiate the IKE negotiation. Valid values are `add | start`.
     */
    readonly tunnel1StartupAction: pulumi.Output<string | undefined>;
    /**
     * The RFC 6890 link-local address of the first VPN tunnel (VPN Gateway Side).
     */
    readonly tunnel1VgwInsideAddress: pulumi.Output<string>;
    /**
     * The public IP address of the second VPN tunnel.
     */
    readonly tunnel2Address: pulumi.Output<string>;
    /**
     * The bgp asn number of the second VPN tunnel.
     */
    readonly tunnel2BgpAsn: pulumi.Output<string>;
    /**
     * The bgp holdtime of the second VPN tunnel.
     */
    readonly tunnel2BgpHoldtime: pulumi.Output<number>;
    /**
     * The RFC 6890 link-local address of the second VPN tunnel (Customer Gateway Side).
     */
    readonly tunnel2CgwInsideAddress: pulumi.Output<string>;
    /**
     * The action to take after DPD timeout occurs for the second VPN tunnel. Specify restart to restart the IKE initiation. Specify clear to end the IKE session. Valid values are `clear | none | restart`.
     */
    readonly tunnel2DpdTimeoutAction: pulumi.Output<string | undefined>;
    /**
     * The number of seconds after which a DPD timeout occurs for the second VPN tunnel. Valid value is equal or higher than `30`.
     */
    readonly tunnel2DpdTimeoutSeconds: pulumi.Output<number | undefined>;
    /**
     * Turn on or off tunnel endpoint lifecycle control feature for the second VPN tunnel. Valid values are `true | false`.
     */
    readonly tunnel2EnableTunnelLifecycleControl: pulumi.Output<boolean | undefined>;
    /**
     * The IKE versions that are permitted for the second VPN tunnel. Valid values are `ikev1 | ikev2`.
     */
    readonly tunnel2IkeVersions: pulumi.Output<string[] | undefined>;
    /**
     * The CIDR block of the inside IP addresses for the second VPN tunnel. Valid value is a size /30 CIDR block from the 169.254.0.0/16 range.
     */
    readonly tunnel2InsideCidr: pulumi.Output<string>;
    /**
     * The range of inside IPv6 addresses for the second VPN tunnel. Supports only EC2 Transit Gateway. Valid value is a size /126 CIDR block from the local fd00::/8 range.
     */
    readonly tunnel2InsideIpv6Cidr: pulumi.Output<string>;
    /**
     * Options for logging VPN tunnel activity. See Log Options below for more details.
     */
    readonly tunnel2LogOptions: pulumi.Output<outputs.ec2.VpnConnectionTunnel2LogOptions>;
    /**
     * List of one or more Diffie-Hellman group numbers that are permitted for the second VPN tunnel for phase 1 IKE negotiations. Valid values are ` 2 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24`.
     */
    readonly tunnel2Phase1DhGroupNumbers: pulumi.Output<number[] | undefined>;
    /**
     * List of one or more encryption algorithms that are permitted for the second VPN tunnel for phase 1 IKE negotiations. Valid values are `AES128 | AES256 | AES128-GCM-16 | AES256-GCM-16`.
     */
    readonly tunnel2Phase1EncryptionAlgorithms: pulumi.Output<string[] | undefined>;
    /**
     * One or more integrity algorithms that are permitted for the second VPN tunnel for phase 1 IKE negotiations. Valid values are `SHA1 | SHA2-256 | SHA2-384 | SHA2-512`.
     */
    readonly tunnel2Phase1IntegrityAlgorithms: pulumi.Output<string[] | undefined>;
    /**
     * The lifetime for phase 1 of the IKE negotiation for the second VPN tunnel, in seconds. Valid value is between `900` and `28800`.
     */
    readonly tunnel2Phase1LifetimeSeconds: pulumi.Output<number | undefined>;
    /**
     * List of one or more Diffie-Hellman group numbers that are permitted for the second VPN tunnel for phase 2 IKE negotiations. Valid values are `2 | 5 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24`.
     */
    readonly tunnel2Phase2DhGroupNumbers: pulumi.Output<number[] | undefined>;
    /**
     * List of one or more encryption algorithms that are permitted for the second VPN tunnel for phase 2 IKE negotiations. Valid values are `AES128 | AES256 | AES128-GCM-16 | AES256-GCM-16`.
     */
    readonly tunnel2Phase2EncryptionAlgorithms: pulumi.Output<string[] | undefined>;
    /**
     * List of one or more integrity algorithms that are permitted for the second VPN tunnel for phase 2 IKE negotiations. Valid values are `SHA1 | SHA2-256 | SHA2-384 | SHA2-512`.
     */
    readonly tunnel2Phase2IntegrityAlgorithms: pulumi.Output<string[] | undefined>;
    /**
     * The lifetime for phase 2 of the IKE negotiation for the second VPN tunnel, in seconds. Valid value is between `900` and `3600`.
     */
    readonly tunnel2Phase2LifetimeSeconds: pulumi.Output<number | undefined>;
    /**
     * The preshared key of the second VPN tunnel. The preshared key must be between 8 and 64 characters in length and cannot start with zero(0). Allowed characters are alphanumeric characters, periods(.) and underscores(_).
     */
    readonly tunnel2PresharedKey: pulumi.Output<string>;
    /**
     * The percentage of the rekey window for the second VPN tunnel (determined by `tunnel2RekeyMarginTimeSeconds`) during which the rekey time is randomly selected. Valid value is between `0` and `100`.
     */
    readonly tunnel2RekeyFuzzPercentage: pulumi.Output<number | undefined>;
    /**
     * The margin time, in seconds, before the phase 2 lifetime expires, during which the AWS side of the second VPN connection performs an IKE rekey. The exact time of the rekey is randomly selected based on the value for `tunnel2RekeyFuzzPercentage`. Valid value is between `60` and half of `tunnel2Phase2LifetimeSeconds`.
     */
    readonly tunnel2RekeyMarginTimeSeconds: pulumi.Output<number | undefined>;
    /**
     * The number of packets in an IKE replay window for the second VPN tunnel. Valid value is between `64` and `2048`.
     */
    readonly tunnel2ReplayWindowSize: pulumi.Output<number | undefined>;
    /**
     * The action to take when the establishing the tunnel for the second VPN connection. By default, your customer gateway device must initiate the IKE negotiation and bring up the tunnel. Specify start for AWS to initiate the IKE negotiation. Valid values are `add | start`.
     */
    readonly tunnel2StartupAction: pulumi.Output<string | undefined>;
    /**
     * The RFC 6890 link-local address of the second VPN tunnel (VPN Gateway Side).
     */
    readonly tunnel2VgwInsideAddress: pulumi.Output<string>;
    /**
     * Indicate whether the VPN tunnels process IPv4 or IPv6 traffic. Valid values are `ipv4 | ipv6`. `ipv6` Supports only EC2 Transit Gateway.
     */
    readonly tunnelInsideIpVersion: pulumi.Output<string>;
    /**
     * The type of VPN connection. The only type AWS supports at this time is "ipsec.1".
     */
    readonly type: pulumi.Output<string>;
    /**
     * Telemetry for the VPN tunnels. Detailed below.
     */
    readonly vgwTelemetries: pulumi.Output<outputs.ec2.VpnConnectionVgwTelemetry[]>;
    /**
     * The ID of the Virtual Private Gateway.
     */
    readonly vpnGatewayId: pulumi.Output<string | undefined>;
    /**
     * Create a VpnConnection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpnConnectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpnConnection resources.
 */
export interface VpnConnectionState {
    /**
     * Amazon Resource Name (ARN) of the VPN Connection.
     */
    arn?: pulumi.Input<string>;
    /**
     * The ARN of the core network.
     */
    coreNetworkArn?: pulumi.Input<string>;
    /**
     * The ARN of the core network attachment.
     */
    coreNetworkAttachmentArn?: pulumi.Input<string>;
    /**
     * The configuration information for the VPN connection's customer gateway (in the native XML format).
     */
    customerGatewayConfiguration?: pulumi.Input<string>;
    /**
     * The ID of the customer gateway.
     */
    customerGatewayId?: pulumi.Input<string>;
    /**
     * Indicate whether to enable acceleration for the VPN connection. Supports only EC2 Transit Gateway.
     */
    enableAcceleration?: pulumi.Input<boolean>;
    /**
     * The IPv4 CIDR on the customer gateway (on-premises) side of the VPN connection.
     */
    localIpv4NetworkCidr?: pulumi.Input<string>;
    /**
     * The IPv6 CIDR on the customer gateway (on-premises) side of the VPN connection.
     */
    localIpv6NetworkCidr?: pulumi.Input<string>;
    /**
     * Indicates if a Public S2S VPN or Private S2S VPN over AWS Direct Connect. Valid values are `PublicIpv4 | PrivateIpv4`
     */
    outsideIpAddressType?: pulumi.Input<string>;
    /**
     * ARN of the Secrets Manager secret storing the pre-shared key(s) for the VPN connection. Note that even if it returns a valid Secrets Manager ARN, the pre-shared key(s) will not be stored in Secrets Manager unless the `presharedKeyStorage` argument is set to `SecretsManager`.
     */
    presharedKeyArn?: pulumi.Input<string>;
    /**
     * Storage mode for the pre-shared key (PSK). Valid values are `Standard` (stored in the Site-to-Site VPN service) or `SecretsManager` (stored in AWS Secrets Manager).
     */
    presharedKeyStorage?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IPv4 CIDR on the AWS side of the VPN connection.
     */
    remoteIpv4NetworkCidr?: pulumi.Input<string>;
    /**
     * The IPv6 CIDR on the AWS side of the VPN connection.
     */
    remoteIpv6NetworkCidr?: pulumi.Input<string>;
    /**
     * The static routes associated with the VPN connection. Detailed below.
     */
    routes?: pulumi.Input<pulumi.Input<inputs.ec2.VpnConnectionRoute>[]>;
    /**
     * Whether the VPN connection uses static routes exclusively. Static routes must be used for devices that don't support BGP.
     */
    staticRoutesOnly?: pulumi.Input<boolean>;
    /**
     * Tags to apply to the connection. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * When associated with an EC2 Transit Gateway (`transitGatewayId` argument), the attachment ID. See also the `aws.ec2.Tag` resource for tagging the EC2 Transit Gateway VPN Attachment.
     */
    transitGatewayAttachmentId?: pulumi.Input<string>;
    /**
     * The ID of the EC2 Transit Gateway.
     */
    transitGatewayId?: pulumi.Input<string>;
    /**
     * . The attachment ID of the Transit Gateway attachment to Direct Connect Gateway. The ID is obtained through a data source only.
     */
    transportTransitGatewayAttachmentId?: pulumi.Input<string>;
    /**
     * The public IP address of the first VPN tunnel.
     */
    tunnel1Address?: pulumi.Input<string>;
    /**
     * The bgp asn number of the first VPN tunnel.
     */
    tunnel1BgpAsn?: pulumi.Input<string>;
    /**
     * The bgp holdtime of the first VPN tunnel.
     */
    tunnel1BgpHoldtime?: pulumi.Input<number>;
    /**
     * The RFC 6890 link-local address of the first VPN tunnel (Customer Gateway Side).
     */
    tunnel1CgwInsideAddress?: pulumi.Input<string>;
    /**
     * The action to take after DPD timeout occurs for the first VPN tunnel. Specify restart to restart the IKE initiation. Specify clear to end the IKE session. Valid values are `clear | none | restart`.
     */
    tunnel1DpdTimeoutAction?: pulumi.Input<string>;
    /**
     * The number of seconds after which a DPD timeout occurs for the first VPN tunnel. Valid value is equal or higher than `30`.
     */
    tunnel1DpdTimeoutSeconds?: pulumi.Input<number>;
    /**
     * Turn on or off tunnel endpoint lifecycle control feature for the first VPN tunnel. Valid values are `true | false`.
     */
    tunnel1EnableTunnelLifecycleControl?: pulumi.Input<boolean>;
    /**
     * The IKE versions that are permitted for the first VPN tunnel. Valid values are `ikev1 | ikev2`.
     */
    tunnel1IkeVersions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The CIDR block of the inside IP addresses for the first VPN tunnel. Valid value is a size /30 CIDR block from the 169.254.0.0/16 range.
     */
    tunnel1InsideCidr?: pulumi.Input<string>;
    /**
     * The range of inside IPv6 addresses for the first VPN tunnel. Supports only EC2 Transit Gateway. Valid value is a size /126 CIDR block from the local fd00::/8 range.
     */
    tunnel1InsideIpv6Cidr?: pulumi.Input<string>;
    /**
     * Options for logging VPN tunnel activity. See Log Options below for more details.
     */
    tunnel1LogOptions?: pulumi.Input<inputs.ec2.VpnConnectionTunnel1LogOptions>;
    /**
     * List of one or more Diffie-Hellman group numbers that are permitted for the first VPN tunnel for phase 1 IKE negotiations. Valid values are ` 2 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24`.
     */
    tunnel1Phase1DhGroupNumbers?: pulumi.Input<pulumi.Input<number>[]>;
    /**
     * List of one or more encryption algorithms that are permitted for the first VPN tunnel for phase 1 IKE negotiations. Valid values are `AES128 | AES256 | AES128-GCM-16 | AES256-GCM-16`.
     */
    tunnel1Phase1EncryptionAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * One or more integrity algorithms that are permitted for the first VPN tunnel for phase 1 IKE negotiations. Valid values are `SHA1 | SHA2-256 | SHA2-384 | SHA2-512`.
     */
    tunnel1Phase1IntegrityAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The lifetime for phase 1 of the IKE negotiation for the first VPN tunnel, in seconds. Valid value is between `900` and `28800`.
     */
    tunnel1Phase1LifetimeSeconds?: pulumi.Input<number>;
    /**
     * List of one or more Diffie-Hellman group numbers that are permitted for the first VPN tunnel for phase 2 IKE negotiations. Valid values are `2 | 5 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24`.
     */
    tunnel1Phase2DhGroupNumbers?: pulumi.Input<pulumi.Input<number>[]>;
    /**
     * List of one or more encryption algorithms that are permitted for the first VPN tunnel for phase 2 IKE negotiations. Valid values are `AES128 | AES256 | AES128-GCM-16 | AES256-GCM-16`.
     */
    tunnel1Phase2EncryptionAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * List of one or more integrity algorithms that are permitted for the first VPN tunnel for phase 2 IKE negotiations. Valid values are `SHA1 | SHA2-256 | SHA2-384 | SHA2-512`.
     */
    tunnel1Phase2IntegrityAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The lifetime for phase 2 of the IKE negotiation for the first VPN tunnel, in seconds. Valid value is between `900` and `3600`.
     */
    tunnel1Phase2LifetimeSeconds?: pulumi.Input<number>;
    /**
     * The preshared key of the first VPN tunnel. The preshared key must be between 8 and 64 characters in length and cannot start with zero(0). Allowed characters are alphanumeric characters, periods(.) and underscores(_).
     */
    tunnel1PresharedKey?: pulumi.Input<string>;
    /**
     * The percentage of the rekey window for the first VPN tunnel (determined by `tunnel1RekeyMarginTimeSeconds`) during which the rekey time is randomly selected. Valid value is between `0` and `100`.
     */
    tunnel1RekeyFuzzPercentage?: pulumi.Input<number>;
    /**
     * The margin time, in seconds, before the phase 2 lifetime expires, during which the AWS side of the first VPN connection performs an IKE rekey. The exact time of the rekey is randomly selected based on the value for `tunnel1RekeyFuzzPercentage`. Valid value is between `60` and half of `tunnel1Phase2LifetimeSeconds`.
     */
    tunnel1RekeyMarginTimeSeconds?: pulumi.Input<number>;
    /**
     * The number of packets in an IKE replay window for the first VPN tunnel. Valid value is between `64` and `2048`.
     */
    tunnel1ReplayWindowSize?: pulumi.Input<number>;
    /**
     * The action to take when the establishing the tunnel for the first VPN connection. By default, your customer gateway device must initiate the IKE negotiation and bring up the tunnel. Specify start for AWS to initiate the IKE negotiation. Valid values are `add | start`.
     */
    tunnel1StartupAction?: pulumi.Input<string>;
    /**
     * The RFC 6890 link-local address of the first VPN tunnel (VPN Gateway Side).
     */
    tunnel1VgwInsideAddress?: pulumi.Input<string>;
    /**
     * The public IP address of the second VPN tunnel.
     */
    tunnel2Address?: pulumi.Input<string>;
    /**
     * The bgp asn number of the second VPN tunnel.
     */
    tunnel2BgpAsn?: pulumi.Input<string>;
    /**
     * The bgp holdtime of the second VPN tunnel.
     */
    tunnel2BgpHoldtime?: pulumi.Input<number>;
    /**
     * The RFC 6890 link-local address of the second VPN tunnel (Customer Gateway Side).
     */
    tunnel2CgwInsideAddress?: pulumi.Input<string>;
    /**
     * The action to take after DPD timeout occurs for the second VPN tunnel. Specify restart to restart the IKE initiation. Specify clear to end the IKE session. Valid values are `clear | none | restart`.
     */
    tunnel2DpdTimeoutAction?: pulumi.Input<string>;
    /**
     * The number of seconds after which a DPD timeout occurs for the second VPN tunnel. Valid value is equal or higher than `30`.
     */
    tunnel2DpdTimeoutSeconds?: pulumi.Input<number>;
    /**
     * Turn on or off tunnel endpoint lifecycle control feature for the second VPN tunnel. Valid values are `true | false`.
     */
    tunnel2EnableTunnelLifecycleControl?: pulumi.Input<boolean>;
    /**
     * The IKE versions that are permitted for the second VPN tunnel. Valid values are `ikev1 | ikev2`.
     */
    tunnel2IkeVersions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The CIDR block of the inside IP addresses for the second VPN tunnel. Valid value is a size /30 CIDR block from the 169.254.0.0/16 range.
     */
    tunnel2InsideCidr?: pulumi.Input<string>;
    /**
     * The range of inside IPv6 addresses for the second VPN tunnel. Supports only EC2 Transit Gateway. Valid value is a size /126 CIDR block from the local fd00::/8 range.
     */
    tunnel2InsideIpv6Cidr?: pulumi.Input<string>;
    /**
     * Options for logging VPN tunnel activity. See Log Options below for more details.
     */
    tunnel2LogOptions?: pulumi.Input<inputs.ec2.VpnConnectionTunnel2LogOptions>;
    /**
     * List of one or more Diffie-Hellman group numbers that are permitted for the second VPN tunnel for phase 1 IKE negotiations. Valid values are ` 2 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24`.
     */
    tunnel2Phase1DhGroupNumbers?: pulumi.Input<pulumi.Input<number>[]>;
    /**
     * List of one or more encryption algorithms that are permitted for the second VPN tunnel for phase 1 IKE negotiations. Valid values are `AES128 | AES256 | AES128-GCM-16 | AES256-GCM-16`.
     */
    tunnel2Phase1EncryptionAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * One or more integrity algorithms that are permitted for the second VPN tunnel for phase 1 IKE negotiations. Valid values are `SHA1 | SHA2-256 | SHA2-384 | SHA2-512`.
     */
    tunnel2Phase1IntegrityAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The lifetime for phase 1 of the IKE negotiation for the second VPN tunnel, in seconds. Valid value is between `900` and `28800`.
     */
    tunnel2Phase1LifetimeSeconds?: pulumi.Input<number>;
    /**
     * List of one or more Diffie-Hellman group numbers that are permitted for the second VPN tunnel for phase 2 IKE negotiations. Valid values are `2 | 5 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24`.
     */
    tunnel2Phase2DhGroupNumbers?: pulumi.Input<pulumi.Input<number>[]>;
    /**
     * List of one or more encryption algorithms that are permitted for the second VPN tunnel for phase 2 IKE negotiations. Valid values are `AES128 | AES256 | AES128-GCM-16 | AES256-GCM-16`.
     */
    tunnel2Phase2EncryptionAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * List of one or more integrity algorithms that are permitted for the second VPN tunnel for phase 2 IKE negotiations. Valid values are `SHA1 | SHA2-256 | SHA2-384 | SHA2-512`.
     */
    tunnel2Phase2IntegrityAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The lifetime for phase 2 of the IKE negotiation for the second VPN tunnel, in seconds. Valid value is between `900` and `3600`.
     */
    tunnel2Phase2LifetimeSeconds?: pulumi.Input<number>;
    /**
     * The preshared key of the second VPN tunnel. The preshared key must be between 8 and 64 characters in length and cannot start with zero(0). Allowed characters are alphanumeric characters, periods(.) and underscores(_).
     */
    tunnel2PresharedKey?: pulumi.Input<string>;
    /**
     * The percentage of the rekey window for the second VPN tunnel (determined by `tunnel2RekeyMarginTimeSeconds`) during which the rekey time is randomly selected. Valid value is between `0` and `100`.
     */
    tunnel2RekeyFuzzPercentage?: pulumi.Input<number>;
    /**
     * The margin time, in seconds, before the phase 2 lifetime expires, during which the AWS side of the second VPN connection performs an IKE rekey. The exact time of the rekey is randomly selected based on the value for `tunnel2RekeyFuzzPercentage`. Valid value is between `60` and half of `tunnel2Phase2LifetimeSeconds`.
     */
    tunnel2RekeyMarginTimeSeconds?: pulumi.Input<number>;
    /**
     * The number of packets in an IKE replay window for the second VPN tunnel. Valid value is between `64` and `2048`.
     */
    tunnel2ReplayWindowSize?: pulumi.Input<number>;
    /**
     * The action to take when the establishing the tunnel for the second VPN connection. By default, your customer gateway device must initiate the IKE negotiation and bring up the tunnel. Specify start for AWS to initiate the IKE negotiation. Valid values are `add | start`.
     */
    tunnel2StartupAction?: pulumi.Input<string>;
    /**
     * The RFC 6890 link-local address of the second VPN tunnel (VPN Gateway Side).
     */
    tunnel2VgwInsideAddress?: pulumi.Input<string>;
    /**
     * Indicate whether the VPN tunnels process IPv4 or IPv6 traffic. Valid values are `ipv4 | ipv6`. `ipv6` Supports only EC2 Transit Gateway.
     */
    tunnelInsideIpVersion?: pulumi.Input<string>;
    /**
     * The type of VPN connection. The only type AWS supports at this time is "ipsec.1".
     */
    type?: pulumi.Input<string>;
    /**
     * Telemetry for the VPN tunnels. Detailed below.
     */
    vgwTelemetries?: pulumi.Input<pulumi.Input<inputs.ec2.VpnConnectionVgwTelemetry>[]>;
    /**
     * The ID of the Virtual Private Gateway.
     */
    vpnGatewayId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VpnConnection resource.
 */
export interface VpnConnectionArgs {
    /**
     * The ID of the customer gateway.
     */
    customerGatewayId: pulumi.Input<string>;
    /**
     * Indicate whether to enable acceleration for the VPN connection. Supports only EC2 Transit Gateway.
     */
    enableAcceleration?: pulumi.Input<boolean>;
    /**
     * The IPv4 CIDR on the customer gateway (on-premises) side of the VPN connection.
     */
    localIpv4NetworkCidr?: pulumi.Input<string>;
    /**
     * The IPv6 CIDR on the customer gateway (on-premises) side of the VPN connection.
     */
    localIpv6NetworkCidr?: pulumi.Input<string>;
    /**
     * Indicates if a Public S2S VPN or Private S2S VPN over AWS Direct Connect. Valid values are `PublicIpv4 | PrivateIpv4`
     */
    outsideIpAddressType?: pulumi.Input<string>;
    /**
     * Storage mode for the pre-shared key (PSK). Valid values are `Standard` (stored in the Site-to-Site VPN service) or `SecretsManager` (stored in AWS Secrets Manager).
     */
    presharedKeyStorage?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IPv4 CIDR on the AWS side of the VPN connection.
     */
    remoteIpv4NetworkCidr?: pulumi.Input<string>;
    /**
     * The IPv6 CIDR on the AWS side of the VPN connection.
     */
    remoteIpv6NetworkCidr?: pulumi.Input<string>;
    /**
     * Whether the VPN connection uses static routes exclusively. Static routes must be used for devices that don't support BGP.
     */
    staticRoutesOnly?: pulumi.Input<boolean>;
    /**
     * Tags to apply to the connection. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the EC2 Transit Gateway.
     */
    transitGatewayId?: pulumi.Input<string>;
    /**
     * . The attachment ID of the Transit Gateway attachment to Direct Connect Gateway. The ID is obtained through a data source only.
     */
    transportTransitGatewayAttachmentId?: pulumi.Input<string>;
    /**
     * The action to take after DPD timeout occurs for the first VPN tunnel. Specify restart to restart the IKE initiation. Specify clear to end the IKE session. Valid values are `clear | none | restart`.
     */
    tunnel1DpdTimeoutAction?: pulumi.Input<string>;
    /**
     * The number of seconds after which a DPD timeout occurs for the first VPN tunnel. Valid value is equal or higher than `30`.
     */
    tunnel1DpdTimeoutSeconds?: pulumi.Input<number>;
    /**
     * Turn on or off tunnel endpoint lifecycle control feature for the first VPN tunnel. Valid values are `true | false`.
     */
    tunnel1EnableTunnelLifecycleControl?: pulumi.Input<boolean>;
    /**
     * The IKE versions that are permitted for the first VPN tunnel. Valid values are `ikev1 | ikev2`.
     */
    tunnel1IkeVersions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The CIDR block of the inside IP addresses for the first VPN tunnel. Valid value is a size /30 CIDR block from the 169.254.0.0/16 range.
     */
    tunnel1InsideCidr?: pulumi.Input<string>;
    /**
     * The range of inside IPv6 addresses for the first VPN tunnel. Supports only EC2 Transit Gateway. Valid value is a size /126 CIDR block from the local fd00::/8 range.
     */
    tunnel1InsideIpv6Cidr?: pulumi.Input<string>;
    /**
     * Options for logging VPN tunnel activity. See Log Options below for more details.
     */
    tunnel1LogOptions?: pulumi.Input<inputs.ec2.VpnConnectionTunnel1LogOptions>;
    /**
     * List of one or more Diffie-Hellman group numbers that are permitted for the first VPN tunnel for phase 1 IKE negotiations. Valid values are ` 2 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24`.
     */
    tunnel1Phase1DhGroupNumbers?: pulumi.Input<pulumi.Input<number>[]>;
    /**
     * List of one or more encryption algorithms that are permitted for the first VPN tunnel for phase 1 IKE negotiations. Valid values are `AES128 | AES256 | AES128-GCM-16 | AES256-GCM-16`.
     */
    tunnel1Phase1EncryptionAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * One or more integrity algorithms that are permitted for the first VPN tunnel for phase 1 IKE negotiations. Valid values are `SHA1 | SHA2-256 | SHA2-384 | SHA2-512`.
     */
    tunnel1Phase1IntegrityAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The lifetime for phase 1 of the IKE negotiation for the first VPN tunnel, in seconds. Valid value is between `900` and `28800`.
     */
    tunnel1Phase1LifetimeSeconds?: pulumi.Input<number>;
    /**
     * List of one or more Diffie-Hellman group numbers that are permitted for the first VPN tunnel for phase 2 IKE negotiations. Valid values are `2 | 5 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24`.
     */
    tunnel1Phase2DhGroupNumbers?: pulumi.Input<pulumi.Input<number>[]>;
    /**
     * List of one or more encryption algorithms that are permitted for the first VPN tunnel for phase 2 IKE negotiations. Valid values are `AES128 | AES256 | AES128-GCM-16 | AES256-GCM-16`.
     */
    tunnel1Phase2EncryptionAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * List of one or more integrity algorithms that are permitted for the first VPN tunnel for phase 2 IKE negotiations. Valid values are `SHA1 | SHA2-256 | SHA2-384 | SHA2-512`.
     */
    tunnel1Phase2IntegrityAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The lifetime for phase 2 of the IKE negotiation for the first VPN tunnel, in seconds. Valid value is between `900` and `3600`.
     */
    tunnel1Phase2LifetimeSeconds?: pulumi.Input<number>;
    /**
     * The preshared key of the first VPN tunnel. The preshared key must be between 8 and 64 characters in length and cannot start with zero(0). Allowed characters are alphanumeric characters, periods(.) and underscores(_).
     */
    tunnel1PresharedKey?: pulumi.Input<string>;
    /**
     * The percentage of the rekey window for the first VPN tunnel (determined by `tunnel1RekeyMarginTimeSeconds`) during which the rekey time is randomly selected. Valid value is between `0` and `100`.
     */
    tunnel1RekeyFuzzPercentage?: pulumi.Input<number>;
    /**
     * The margin time, in seconds, before the phase 2 lifetime expires, during which the AWS side of the first VPN connection performs an IKE rekey. The exact time of the rekey is randomly selected based on the value for `tunnel1RekeyFuzzPercentage`. Valid value is between `60` and half of `tunnel1Phase2LifetimeSeconds`.
     */
    tunnel1RekeyMarginTimeSeconds?: pulumi.Input<number>;
    /**
     * The number of packets in an IKE replay window for the first VPN tunnel. Valid value is between `64` and `2048`.
     */
    tunnel1ReplayWindowSize?: pulumi.Input<number>;
    /**
     * The action to take when the establishing the tunnel for the first VPN connection. By default, your customer gateway device must initiate the IKE negotiation and bring up the tunnel. Specify start for AWS to initiate the IKE negotiation. Valid values are `add | start`.
     */
    tunnel1StartupAction?: pulumi.Input<string>;
    /**
     * The action to take after DPD timeout occurs for the second VPN tunnel. Specify restart to restart the IKE initiation. Specify clear to end the IKE session. Valid values are `clear | none | restart`.
     */
    tunnel2DpdTimeoutAction?: pulumi.Input<string>;
    /**
     * The number of seconds after which a DPD timeout occurs for the second VPN tunnel. Valid value is equal or higher than `30`.
     */
    tunnel2DpdTimeoutSeconds?: pulumi.Input<number>;
    /**
     * Turn on or off tunnel endpoint lifecycle control feature for the second VPN tunnel. Valid values are `true | false`.
     */
    tunnel2EnableTunnelLifecycleControl?: pulumi.Input<boolean>;
    /**
     * The IKE versions that are permitted for the second VPN tunnel. Valid values are `ikev1 | ikev2`.
     */
    tunnel2IkeVersions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The CIDR block of the inside IP addresses for the second VPN tunnel. Valid value is a size /30 CIDR block from the 169.254.0.0/16 range.
     */
    tunnel2InsideCidr?: pulumi.Input<string>;
    /**
     * The range of inside IPv6 addresses for the second VPN tunnel. Supports only EC2 Transit Gateway. Valid value is a size /126 CIDR block from the local fd00::/8 range.
     */
    tunnel2InsideIpv6Cidr?: pulumi.Input<string>;
    /**
     * Options for logging VPN tunnel activity. See Log Options below for more details.
     */
    tunnel2LogOptions?: pulumi.Input<inputs.ec2.VpnConnectionTunnel2LogOptions>;
    /**
     * List of one or more Diffie-Hellman group numbers that are permitted for the second VPN tunnel for phase 1 IKE negotiations. Valid values are ` 2 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24`.
     */
    tunnel2Phase1DhGroupNumbers?: pulumi.Input<pulumi.Input<number>[]>;
    /**
     * List of one or more encryption algorithms that are permitted for the second VPN tunnel for phase 1 IKE negotiations. Valid values are `AES128 | AES256 | AES128-GCM-16 | AES256-GCM-16`.
     */
    tunnel2Phase1EncryptionAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * One or more integrity algorithms that are permitted for the second VPN tunnel for phase 1 IKE negotiations. Valid values are `SHA1 | SHA2-256 | SHA2-384 | SHA2-512`.
     */
    tunnel2Phase1IntegrityAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The lifetime for phase 1 of the IKE negotiation for the second VPN tunnel, in seconds. Valid value is between `900` and `28800`.
     */
    tunnel2Phase1LifetimeSeconds?: pulumi.Input<number>;
    /**
     * List of one or more Diffie-Hellman group numbers that are permitted for the second VPN tunnel for phase 2 IKE negotiations. Valid values are `2 | 5 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24`.
     */
    tunnel2Phase2DhGroupNumbers?: pulumi.Input<pulumi.Input<number>[]>;
    /**
     * List of one or more encryption algorithms that are permitted for the second VPN tunnel for phase 2 IKE negotiations. Valid values are `AES128 | AES256 | AES128-GCM-16 | AES256-GCM-16`.
     */
    tunnel2Phase2EncryptionAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * List of one or more integrity algorithms that are permitted for the second VPN tunnel for phase 2 IKE negotiations. Valid values are `SHA1 | SHA2-256 | SHA2-384 | SHA2-512`.
     */
    tunnel2Phase2IntegrityAlgorithms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The lifetime for phase 2 of the IKE negotiation for the second VPN tunnel, in seconds. Valid value is between `900` and `3600`.
     */
    tunnel2Phase2LifetimeSeconds?: pulumi.Input<number>;
    /**
     * The preshared key of the second VPN tunnel. The preshared key must be between 8 and 64 characters in length and cannot start with zero(0). Allowed characters are alphanumeric characters, periods(.) and underscores(_).
     */
    tunnel2PresharedKey?: pulumi.Input<string>;
    /**
     * The percentage of the rekey window for the second VPN tunnel (determined by `tunnel2RekeyMarginTimeSeconds`) during which the rekey time is randomly selected. Valid value is between `0` and `100`.
     */
    tunnel2RekeyFuzzPercentage?: pulumi.Input<number>;
    /**
     * The margin time, in seconds, before the phase 2 lifetime expires, during which the AWS side of the second VPN connection performs an IKE rekey. The exact time of the rekey is randomly selected based on the value for `tunnel2RekeyFuzzPercentage`. Valid value is between `60` and half of `tunnel2Phase2LifetimeSeconds`.
     */
    tunnel2RekeyMarginTimeSeconds?: pulumi.Input<number>;
    /**
     * The number of packets in an IKE replay window for the second VPN tunnel. Valid value is between `64` and `2048`.
     */
    tunnel2ReplayWindowSize?: pulumi.Input<number>;
    /**
     * The action to take when the establishing the tunnel for the second VPN connection. By default, your customer gateway device must initiate the IKE negotiation and bring up the tunnel. Specify start for AWS to initiate the IKE negotiation. Valid values are `add | start`.
     */
    tunnel2StartupAction?: pulumi.Input<string>;
    /**
     * Indicate whether the VPN tunnels process IPv4 or IPv6 traffic. Valid values are `ipv4 | ipv6`. `ipv6` Supports only EC2 Transit Gateway.
     */
    tunnelInsideIpVersion?: pulumi.Input<string>;
    /**
     * The type of VPN connection. The only type AWS supports at this time is "ipsec.1".
     */
    type: pulumi.Input<string>;
    /**
     * The ID of the Virtual Private Gateway.
     */
    vpnGatewayId?: pulumi.Input<string>;
}
