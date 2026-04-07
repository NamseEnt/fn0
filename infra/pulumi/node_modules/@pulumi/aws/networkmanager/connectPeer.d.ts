import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Network Manager Connect Peer.
 *
 * Use this resource to create a Connect peer in AWS Network Manager. Connect peers establish BGP sessions with your on-premises networks through Connect attachments, enabling dynamic routing between your core network and external networks.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.VpcAttachment("example", {
 *     subnetArns: exampleAwsSubnet.map(__item => __item.arn),
 *     coreNetworkId: exampleAwsccNetworkmanagerCoreNetwork.id,
 *     vpcArn: exampleAwsVpc.arn,
 * });
 * const exampleConnectAttachment = new aws.networkmanager.ConnectAttachment("example", {
 *     coreNetworkId: exampleAwsccNetworkmanagerCoreNetwork.id,
 *     transportAttachmentId: example.id,
 *     edgeLocation: example.edgeLocation,
 *     options: {
 *         protocol: "GRE",
 *     },
 * });
 * const exampleConnectPeer = new aws.networkmanager.ConnectPeer("example", {
 *     connectAttachmentId: exampleConnectAttachment.id,
 *     peerAddress: "127.0.0.1",
 *     bgpOptions: {
 *         peerAsn: 65000,
 *     },
 *     insideCidrBlocks: ["172.16.0.0/16"],
 * });
 * ```
 *
 * ### Usage with attachment accepter
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.VpcAttachment("example", {
 *     subnetArns: exampleAwsSubnet.map(__item => __item.arn),
 *     coreNetworkId: exampleAwsccNetworkmanagerCoreNetwork.id,
 *     vpcArn: exampleAwsVpc.arn,
 * });
 * const exampleAttachmentAccepter = new aws.networkmanager.AttachmentAccepter("example", {
 *     attachmentId: example.id,
 *     attachmentType: example.attachmentType,
 * });
 * const exampleConnectAttachment = new aws.networkmanager.ConnectAttachment("example", {
 *     coreNetworkId: exampleAwsccNetworkmanagerCoreNetwork.id,
 *     transportAttachmentId: example.id,
 *     edgeLocation: example.edgeLocation,
 *     options: {
 *         protocol: "GRE",
 *     },
 * }, {
 *     dependsOn: [exampleAttachmentAccepter],
 * });
 * const example2 = new aws.networkmanager.AttachmentAccepter("example2", {
 *     attachmentId: exampleConnectAttachment.id,
 *     attachmentType: exampleConnectAttachment.attachmentType,
 * });
 * const exampleConnectPeer = new aws.networkmanager.ConnectPeer("example", {
 *     connectAttachmentId: exampleConnectAttachment.id,
 *     peerAddress: "127.0.0.1",
 *     bgpOptions: {
 *         peerAsn: 65500,
 *     },
 *     insideCidrBlocks: ["172.16.0.0/16"],
 * }, {
 *     dependsOn: [example2],
 * });
 * ```
 *
 * ### Usage with a Tunnel-less Connect attachment
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.VpcAttachment("example", {
 *     subnetArns: exampleAwsSubnet.map(__item => __item.arn),
 *     coreNetworkId: exampleAwsccNetworkmanagerCoreNetwork.id,
 *     vpcArn: exampleAwsVpc.arn,
 * });
 * const exampleConnectAttachment = new aws.networkmanager.ConnectAttachment("example", {
 *     coreNetworkId: exampleAwsccNetworkmanagerCoreNetwork.id,
 *     transportAttachmentId: example.id,
 *     edgeLocation: example.edgeLocation,
 *     options: {
 *         protocol: "NO_ENCAP",
 *     },
 * });
 * const exampleConnectPeer = new aws.networkmanager.ConnectPeer("example", {
 *     connectAttachmentId: exampleConnectAttachment.id,
 *     peerAddress: "127.0.0.1",
 *     bgpOptions: {
 *         peerAsn: 65000,
 *     },
 *     subnetArn: example2.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_networkmanager_connect_peer` using the connect peer ID. For example:
 *
 * ```sh
 * $ pulumi import aws:networkmanager/connectPeer:ConnectPeer example connect-peer-061f3e96275db1acc
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
     * ARN of the Connect peer.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Connect peer BGP options. See bgpOptions for more information.
     */
    readonly bgpOptions: pulumi.Output<outputs.networkmanager.ConnectPeerBgpOptions | undefined>;
    /**
     * Configuration of the Connect peer.
     */
    readonly configurations: pulumi.Output<outputs.networkmanager.ConnectPeerConfiguration[]>;
    /**
     * ID of the connection attachment.
     */
    readonly connectAttachmentId: pulumi.Output<string>;
    /**
     * ID of the Connect peer.
     */
    readonly connectPeerId: pulumi.Output<string>;
    /**
     * Connect peer core network address.
     */
    readonly coreNetworkAddress: pulumi.Output<string | undefined>;
    /**
     * ID of a core network.
     */
    readonly coreNetworkId: pulumi.Output<string>;
    /**
     * Timestamp when the Connect peer was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Region where the peer is located.
     */
    readonly edgeLocation: pulumi.Output<string>;
    /**
     * Inside IP addresses used for BGP peering. Required when the Connect attachment protocol is `GRE`. See `aws.networkmanager.ConnectAttachment` for details.
     */
    readonly insideCidrBlocks: pulumi.Output<string[] | undefined>;
    /**
     * Connect peer address.
     *
     * The following arguments are optional:
     */
    readonly peerAddress: pulumi.Output<string>;
    /**
     * State of the Connect peer.
     */
    readonly state: pulumi.Output<string>;
    /**
     * Subnet ARN for the Connect peer. Required when the Connect attachment protocol is `NO_ENCAP`. See `aws.networkmanager.ConnectAttachment` for details.
     */
    readonly subnetArn: pulumi.Output<string | undefined>;
    /**
     * Key-value tags for the attachment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
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
     * ARN of the Connect peer.
     */
    arn?: pulumi.Input<string>;
    /**
     * Connect peer BGP options. See bgpOptions for more information.
     */
    bgpOptions?: pulumi.Input<inputs.networkmanager.ConnectPeerBgpOptions>;
    /**
     * Configuration of the Connect peer.
     */
    configurations?: pulumi.Input<pulumi.Input<inputs.networkmanager.ConnectPeerConfiguration>[]>;
    /**
     * ID of the connection attachment.
     */
    connectAttachmentId?: pulumi.Input<string>;
    /**
     * ID of the Connect peer.
     */
    connectPeerId?: pulumi.Input<string>;
    /**
     * Connect peer core network address.
     */
    coreNetworkAddress?: pulumi.Input<string>;
    /**
     * ID of a core network.
     */
    coreNetworkId?: pulumi.Input<string>;
    /**
     * Timestamp when the Connect peer was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Region where the peer is located.
     */
    edgeLocation?: pulumi.Input<string>;
    /**
     * Inside IP addresses used for BGP peering. Required when the Connect attachment protocol is `GRE`. See `aws.networkmanager.ConnectAttachment` for details.
     */
    insideCidrBlocks?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Connect peer address.
     *
     * The following arguments are optional:
     */
    peerAddress?: pulumi.Input<string>;
    /**
     * State of the Connect peer.
     */
    state?: pulumi.Input<string>;
    /**
     * Subnet ARN for the Connect peer. Required when the Connect attachment protocol is `NO_ENCAP`. See `aws.networkmanager.ConnectAttachment` for details.
     */
    subnetArn?: pulumi.Input<string>;
    /**
     * Key-value tags for the attachment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a ConnectPeer resource.
 */
export interface ConnectPeerArgs {
    /**
     * Connect peer BGP options. See bgpOptions for more information.
     */
    bgpOptions?: pulumi.Input<inputs.networkmanager.ConnectPeerBgpOptions>;
    /**
     * ID of the connection attachment.
     */
    connectAttachmentId: pulumi.Input<string>;
    /**
     * Connect peer core network address.
     */
    coreNetworkAddress?: pulumi.Input<string>;
    /**
     * Inside IP addresses used for BGP peering. Required when the Connect attachment protocol is `GRE`. See `aws.networkmanager.ConnectAttachment` for details.
     */
    insideCidrBlocks?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Connect peer address.
     *
     * The following arguments are optional:
     */
    peerAddress: pulumi.Input<string>;
    /**
     * Subnet ARN for the Connect peer. Required when the Connect attachment protocol is `NO_ENCAP`. See `aws.networkmanager.ConnectAttachment` for details.
     */
    subnetArn?: pulumi.Input<string>;
    /**
     * Key-value tags for the attachment. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
