import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource for managing a VPC (Virtual Private Cloud) Route Server Peer.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.vpc.RouteServerPeer("test", {
 *     routeServerEndpointId: example.routeServerEndpointId,
 *     peerAddress: "10.0.1.250",
 *     bgpOptions: {
 *         peerAsn: 65200,
 *     },
 *     tags: {
 *         Name: "Appliance 1",
 *     },
 * });
 * ```
 *
 * ### Complete Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.vpc.RouteServer("test", {
 *     amazonSideAsn: 4294967294,
 *     tags: {
 *         Name: "Test",
 *     },
 * });
 * const testVpcRouteServerAssociation = new aws.index.VpcRouteServerAssociation("test", {
 *     routeServerId: test.routeServerId,
 *     vpcId: testAwsVpc.id,
 * });
 * const testRouteServerEndpoint = new aws.vpc.RouteServerEndpoint("test", {
 *     routeServerId: test.routeServerId,
 *     subnetId: testAwsSubnet.id,
 *     tags: {
 *         Name: "Test Endpoint",
 *     },
 * }, {
 *     dependsOn: [testVpcRouteServerAssociation],
 * });
 * const testRouteServerPropagation = new aws.vpc.RouteServerPropagation("test", {
 *     routeServerId: test.routeServerId,
 *     routeTableId: testAwsRouteTable.id,
 * }, {
 *     dependsOn: [testVpcRouteServerAssociation],
 * });
 * const testRouteServerPeer = new aws.vpc.RouteServerPeer("test", {
 *     routeServerEndpointId: testRouteServerEndpoint.routeServerEndpointId,
 *     peerAddress: "10.0.1.250",
 *     bgpOptions: {
 *         peerAsn: 65000,
 *         peerLivenessDetection: "bgp-keepalive",
 *     },
 *     tags: {
 *         Name: "Test Appliance",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC (Virtual Private Cloud) Route Server using the `route_server_peer_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:vpc/routeServerPeer:RouteServerPeer example rsp-12345678
 * ```
 */
export declare class RouteServerPeer extends pulumi.CustomResource {
    /**
     * Get an existing RouteServerPeer resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RouteServerPeerState, opts?: pulumi.CustomResourceOptions): RouteServerPeer;
    /**
     * Returns true if the given object is an instance of RouteServerPeer.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RouteServerPeer;
    /**
     * The ARN of the route server peer.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The BGP options for the peer, including ASN (Autonomous System Number) and BFD (Bidrectional Forwarding Detection) settings. Configuration block with BGP Options configuration Detailed below
     */
    readonly bgpOptions: pulumi.Output<outputs.vpc.RouteServerPeerBgpOptions | undefined>;
    /**
     * The IP address of the Elastic network interface for the route server endpoint.
     */
    readonly endpointEniAddress: pulumi.Output<string>;
    /**
     * The ID of the Elastic network interface for the route server endpoint.
     */
    readonly endpointEniId: pulumi.Output<string>;
    /**
     * The IPv4 address of the peer device.
     */
    readonly peerAddress: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the route server endpoint for which to create a peer.
     *
     * The following arguments are optional:
     */
    readonly routeServerEndpointId: pulumi.Output<string>;
    /**
     * The ID of the route server associated with this peer.
     */
    readonly routeServerId: pulumi.Output<string>;
    /**
     * The unique identifier of the route server peer.
     */
    readonly routeServerPeerId: pulumi.Output<string>;
    /**
     * The ID of the subnet containing the route server peer.
     */
    readonly subnetId: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timeouts: pulumi.Output<outputs.vpc.RouteServerPeerTimeouts | undefined>;
    /**
     * The ID of the VPC containing the route server peer.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a RouteServerPeer resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RouteServerPeerArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RouteServerPeer resources.
 */
export interface RouteServerPeerState {
    /**
     * The ARN of the route server peer.
     */
    arn?: pulumi.Input<string>;
    /**
     * The BGP options for the peer, including ASN (Autonomous System Number) and BFD (Bidrectional Forwarding Detection) settings. Configuration block with BGP Options configuration Detailed below
     */
    bgpOptions?: pulumi.Input<inputs.vpc.RouteServerPeerBgpOptions>;
    /**
     * The IP address of the Elastic network interface for the route server endpoint.
     */
    endpointEniAddress?: pulumi.Input<string>;
    /**
     * The ID of the Elastic network interface for the route server endpoint.
     */
    endpointEniId?: pulumi.Input<string>;
    /**
     * The IPv4 address of the peer device.
     */
    peerAddress?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the route server endpoint for which to create a peer.
     *
     * The following arguments are optional:
     */
    routeServerEndpointId?: pulumi.Input<string>;
    /**
     * The ID of the route server associated with this peer.
     */
    routeServerId?: pulumi.Input<string>;
    /**
     * The unique identifier of the route server peer.
     */
    routeServerPeerId?: pulumi.Input<string>;
    /**
     * The ID of the subnet containing the route server peer.
     */
    subnetId?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timeouts?: pulumi.Input<inputs.vpc.RouteServerPeerTimeouts>;
    /**
     * The ID of the VPC containing the route server peer.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RouteServerPeer resource.
 */
export interface RouteServerPeerArgs {
    /**
     * The BGP options for the peer, including ASN (Autonomous System Number) and BFD (Bidrectional Forwarding Detection) settings. Configuration block with BGP Options configuration Detailed below
     */
    bgpOptions?: pulumi.Input<inputs.vpc.RouteServerPeerBgpOptions>;
    /**
     * The IPv4 address of the peer device.
     */
    peerAddress: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the route server endpoint for which to create a peer.
     *
     * The following arguments are optional:
     */
    routeServerEndpointId: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.vpc.RouteServerPeerTimeouts>;
}
