import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS Client VPN endpoint for OpenVPN clients. For more information on usage, please see the
 * [AWS Client VPN Administrator's Guide](https://docs.aws.amazon.com/vpn/latest/clientvpn-admin/what-is.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2clientvpn.Endpoint("example", {
 *     description: "clientvpn-example",
 *     serverCertificateArn: cert.arn,
 *     clientCidrBlock: "10.0.0.0/16",
 *     authenticationOptions: [{
 *         type: "certificate-authentication",
 *         rootCertificateChainArn: rootCert.arn,
 *     }],
 *     connectionLogOptions: {
 *         enabled: true,
 *         cloudwatchLogGroup: lg.name,
 *         cloudwatchLogStream: ls.name,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AWS Client VPN endpoints using the `id` value found via `aws ec2 describe-client-vpn-endpoints`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2clientvpn/endpoint:Endpoint example cvpn-endpoint-0ac3a1abbccddd666
 * ```
 */
export declare class Endpoint extends pulumi.CustomResource {
    /**
     * Get an existing Endpoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EndpointState, opts?: pulumi.CustomResourceOptions): Endpoint;
    /**
     * Returns true if the given object is an instance of Endpoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Endpoint;
    /**
     * The ARN of the Client VPN endpoint.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Information about the authentication method to be used to authenticate clients.
     */
    readonly authenticationOptions: pulumi.Output<outputs.ec2clientvpn.EndpointAuthenticationOption[]>;
    /**
     * The IPv4 address range, in CIDR notation, from which to assign client IP addresses. The address range cannot overlap with the local CIDR of the VPC in which the associated subnet is located, or the routes that you add manually. The address range cannot be changed after the Client VPN endpoint has been created. The CIDR block should be /22 or greater. When `trafficIpAddressType` is set to `ipv6`, it must not be specified. Otherwise, it is required.
     */
    readonly clientCidrBlock: pulumi.Output<string | undefined>;
    /**
     * The options for managing connection authorization for new client connections.
     */
    readonly clientConnectOptions: pulumi.Output<outputs.ec2clientvpn.EndpointClientConnectOptions>;
    /**
     * Options for enabling a customizable text banner that will be displayed on AWS provided clients when a VPN session is established.
     */
    readonly clientLoginBannerOptions: pulumi.Output<outputs.ec2clientvpn.EndpointClientLoginBannerOptions>;
    /**
     * Options for enforce administrator defined routes on devices connected through the VPN.
     */
    readonly clientRouteEnforcementOptions: pulumi.Output<outputs.ec2clientvpn.EndpointClientRouteEnforcementOptions>;
    /**
     * Information about the client connection logging options.
     */
    readonly connectionLogOptions: pulumi.Output<outputs.ec2clientvpn.EndpointConnectionLogOptions>;
    /**
     * A brief description of the Client VPN endpoint.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Indicates whether the client VPN session is disconnected after the maximum `sessionTimeoutHours` is reached. If `true`, users are prompted to reconnect client VPN. If `false`, client VPN attempts to reconnect automatically. The default value is `false`.
     */
    readonly disconnectOnSessionTimeout: pulumi.Output<boolean>;
    /**
     * The DNS name to be used by clients when establishing their VPN session.
     */
    readonly dnsName: pulumi.Output<string>;
    /**
     * Information about the DNS servers to be used for DNS resolution. A Client VPN endpoint can have up to two DNS servers. If no DNS server is specified, the DNS address of the connecting device is used.
     */
    readonly dnsServers: pulumi.Output<string[] | undefined>;
    /**
     * IP address type for the Client VPN endpoint. Valid values are `ipv4`, `ipv6`, or `dual-stack`. Defaults to `ipv4`.
     */
    readonly endpointIpAddressType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The IDs of one or more security groups to apply to the target network. You must also specify the ID of the VPC that contains the security groups.
     */
    readonly securityGroupIds: pulumi.Output<string[]>;
    /**
     * Specify whether to enable the self-service portal for the Client VPN endpoint. Values can be `enabled` or `disabled`. Default value is `disabled`.
     */
    readonly selfServicePortal: pulumi.Output<string | undefined>;
    /**
     * The URL of the self-service portal.
     */
    readonly selfServicePortalUrl: pulumi.Output<string>;
    /**
     * The ARN of the ACM server certificate.
     */
    readonly serverCertificateArn: pulumi.Output<string>;
    /**
     * The maximum session duration is a trigger by which end-users are required to re-authenticate prior to establishing a VPN session. Default value is `24` - Valid values: `8 | 10 | 12 | 24`
     */
    readonly sessionTimeoutHours: pulumi.Output<number | undefined>;
    /**
     * Indicates whether split-tunnel is enabled on VPN endpoint. Default value is `false`.
     */
    readonly splitTunnel: pulumi.Output<boolean | undefined>;
    /**
     * A mapping of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * IP address type for traffic within the Client VPN tunnel. Valid values are `ipv4`, `ipv6`, or `dual-stack`. Defaults to `ipv4`. When it is set to `ipv6`, `clientCidrBlock` must not be specified.
     */
    readonly trafficIpAddressType: pulumi.Output<string>;
    /**
     * The transport protocol to be used by the VPN session. Default value is `udp`.
     */
    readonly transportProtocol: pulumi.Output<string | undefined>;
    /**
     * The ID of the VPC to associate with the Client VPN endpoint. If no security group IDs are specified in the request, the default security group for the VPC is applied.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * The port number for the Client VPN endpoint. Valid values are `443` and `1194`. Default value is `443`.
     */
    readonly vpnPort: pulumi.Output<number | undefined>;
    /**
     * Create a Endpoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EndpointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Endpoint resources.
 */
export interface EndpointState {
    /**
     * The ARN of the Client VPN endpoint.
     */
    arn?: pulumi.Input<string>;
    /**
     * Information about the authentication method to be used to authenticate clients.
     */
    authenticationOptions?: pulumi.Input<pulumi.Input<inputs.ec2clientvpn.EndpointAuthenticationOption>[]>;
    /**
     * The IPv4 address range, in CIDR notation, from which to assign client IP addresses. The address range cannot overlap with the local CIDR of the VPC in which the associated subnet is located, or the routes that you add manually. The address range cannot be changed after the Client VPN endpoint has been created. The CIDR block should be /22 or greater. When `trafficIpAddressType` is set to `ipv6`, it must not be specified. Otherwise, it is required.
     */
    clientCidrBlock?: pulumi.Input<string>;
    /**
     * The options for managing connection authorization for new client connections.
     */
    clientConnectOptions?: pulumi.Input<inputs.ec2clientvpn.EndpointClientConnectOptions>;
    /**
     * Options for enabling a customizable text banner that will be displayed on AWS provided clients when a VPN session is established.
     */
    clientLoginBannerOptions?: pulumi.Input<inputs.ec2clientvpn.EndpointClientLoginBannerOptions>;
    /**
     * Options for enforce administrator defined routes on devices connected through the VPN.
     */
    clientRouteEnforcementOptions?: pulumi.Input<inputs.ec2clientvpn.EndpointClientRouteEnforcementOptions>;
    /**
     * Information about the client connection logging options.
     */
    connectionLogOptions?: pulumi.Input<inputs.ec2clientvpn.EndpointConnectionLogOptions>;
    /**
     * A brief description of the Client VPN endpoint.
     */
    description?: pulumi.Input<string>;
    /**
     * Indicates whether the client VPN session is disconnected after the maximum `sessionTimeoutHours` is reached. If `true`, users are prompted to reconnect client VPN. If `false`, client VPN attempts to reconnect automatically. The default value is `false`.
     */
    disconnectOnSessionTimeout?: pulumi.Input<boolean>;
    /**
     * The DNS name to be used by clients when establishing their VPN session.
     */
    dnsName?: pulumi.Input<string>;
    /**
     * Information about the DNS servers to be used for DNS resolution. A Client VPN endpoint can have up to two DNS servers. If no DNS server is specified, the DNS address of the connecting device is used.
     */
    dnsServers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * IP address type for the Client VPN endpoint. Valid values are `ipv4`, `ipv6`, or `dual-stack`. Defaults to `ipv4`.
     */
    endpointIpAddressType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IDs of one or more security groups to apply to the target network. You must also specify the ID of the VPC that contains the security groups.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Specify whether to enable the self-service portal for the Client VPN endpoint. Values can be `enabled` or `disabled`. Default value is `disabled`.
     */
    selfServicePortal?: pulumi.Input<string>;
    /**
     * The URL of the self-service portal.
     */
    selfServicePortalUrl?: pulumi.Input<string>;
    /**
     * The ARN of the ACM server certificate.
     */
    serverCertificateArn?: pulumi.Input<string>;
    /**
     * The maximum session duration is a trigger by which end-users are required to re-authenticate prior to establishing a VPN session. Default value is `24` - Valid values: `8 | 10 | 12 | 24`
     */
    sessionTimeoutHours?: pulumi.Input<number>;
    /**
     * Indicates whether split-tunnel is enabled on VPN endpoint. Default value is `false`.
     */
    splitTunnel?: pulumi.Input<boolean>;
    /**
     * A mapping of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * IP address type for traffic within the Client VPN tunnel. Valid values are `ipv4`, `ipv6`, or `dual-stack`. Defaults to `ipv4`. When it is set to `ipv6`, `clientCidrBlock` must not be specified.
     */
    trafficIpAddressType?: pulumi.Input<string>;
    /**
     * The transport protocol to be used by the VPN session. Default value is `udp`.
     */
    transportProtocol?: pulumi.Input<string>;
    /**
     * The ID of the VPC to associate with the Client VPN endpoint. If no security group IDs are specified in the request, the default security group for the VPC is applied.
     */
    vpcId?: pulumi.Input<string>;
    /**
     * The port number for the Client VPN endpoint. Valid values are `443` and `1194`. Default value is `443`.
     */
    vpnPort?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a Endpoint resource.
 */
export interface EndpointArgs {
    /**
     * Information about the authentication method to be used to authenticate clients.
     */
    authenticationOptions: pulumi.Input<pulumi.Input<inputs.ec2clientvpn.EndpointAuthenticationOption>[]>;
    /**
     * The IPv4 address range, in CIDR notation, from which to assign client IP addresses. The address range cannot overlap with the local CIDR of the VPC in which the associated subnet is located, or the routes that you add manually. The address range cannot be changed after the Client VPN endpoint has been created. The CIDR block should be /22 or greater. When `trafficIpAddressType` is set to `ipv6`, it must not be specified. Otherwise, it is required.
     */
    clientCidrBlock?: pulumi.Input<string>;
    /**
     * The options for managing connection authorization for new client connections.
     */
    clientConnectOptions?: pulumi.Input<inputs.ec2clientvpn.EndpointClientConnectOptions>;
    /**
     * Options for enabling a customizable text banner that will be displayed on AWS provided clients when a VPN session is established.
     */
    clientLoginBannerOptions?: pulumi.Input<inputs.ec2clientvpn.EndpointClientLoginBannerOptions>;
    /**
     * Options for enforce administrator defined routes on devices connected through the VPN.
     */
    clientRouteEnforcementOptions?: pulumi.Input<inputs.ec2clientvpn.EndpointClientRouteEnforcementOptions>;
    /**
     * Information about the client connection logging options.
     */
    connectionLogOptions: pulumi.Input<inputs.ec2clientvpn.EndpointConnectionLogOptions>;
    /**
     * A brief description of the Client VPN endpoint.
     */
    description?: pulumi.Input<string>;
    /**
     * Indicates whether the client VPN session is disconnected after the maximum `sessionTimeoutHours` is reached. If `true`, users are prompted to reconnect client VPN. If `false`, client VPN attempts to reconnect automatically. The default value is `false`.
     */
    disconnectOnSessionTimeout?: pulumi.Input<boolean>;
    /**
     * Information about the DNS servers to be used for DNS resolution. A Client VPN endpoint can have up to two DNS servers. If no DNS server is specified, the DNS address of the connecting device is used.
     */
    dnsServers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * IP address type for the Client VPN endpoint. Valid values are `ipv4`, `ipv6`, or `dual-stack`. Defaults to `ipv4`.
     */
    endpointIpAddressType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IDs of one or more security groups to apply to the target network. You must also specify the ID of the VPC that contains the security groups.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Specify whether to enable the self-service portal for the Client VPN endpoint. Values can be `enabled` or `disabled`. Default value is `disabled`.
     */
    selfServicePortal?: pulumi.Input<string>;
    /**
     * The ARN of the ACM server certificate.
     */
    serverCertificateArn: pulumi.Input<string>;
    /**
     * The maximum session duration is a trigger by which end-users are required to re-authenticate prior to establishing a VPN session. Default value is `24` - Valid values: `8 | 10 | 12 | 24`
     */
    sessionTimeoutHours?: pulumi.Input<number>;
    /**
     * Indicates whether split-tunnel is enabled on VPN endpoint. Default value is `false`.
     */
    splitTunnel?: pulumi.Input<boolean>;
    /**
     * A mapping of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * IP address type for traffic within the Client VPN tunnel. Valid values are `ipv4`, `ipv6`, or `dual-stack`. Defaults to `ipv4`. When it is set to `ipv6`, `clientCidrBlock` must not be specified.
     */
    trafficIpAddressType?: pulumi.Input<string>;
    /**
     * The transport protocol to be used by the VPN session. Default value is `udp`.
     */
    transportProtocol?: pulumi.Input<string>;
    /**
     * The ID of the VPC to associate with the Client VPN endpoint. If no security group IDs are specified in the request, the default security group for the VPC is applied.
     */
    vpcId?: pulumi.Input<string>;
    /**
     * The port number for the Client VPN endpoint. Valid values are `443` and `1194`. Default value is `443`.
     */
    vpnPort?: pulumi.Input<number>;
}
