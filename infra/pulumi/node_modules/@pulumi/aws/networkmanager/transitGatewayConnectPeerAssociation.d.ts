import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Network Manager transit gateway Connect peer association. Associates a transit gateway Connect peer with a device, and optionally, with a link. If you specify a link, it must be associated with the specified device.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.TransitGatewayConnectPeerAssociation("example", {
 *     globalNetworkId: exampleAwsNetworkmanagerGlobalNetwork.id,
 *     deviceId: exampleAwsNetworkmanagerDevice.id,
 *     transitGatewayConnectPeerArn: exampleAwsEc2TransitGatewayConnectPeer.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_networkmanager_transit_gateway_connect_peer_association` using the global network ID and Connect peer ARN. For example:
 *
 * ```sh
 * $ pulumi import aws:networkmanager/transitGatewayConnectPeerAssociation:TransitGatewayConnectPeerAssociation example global-network-0d47f6t230mz46dy4,arn:aws:ec2:us-west-2:123456789012:transit-gateway-connect-peer/tgw-connect-peer-12345678
 * ```
 */
export declare class TransitGatewayConnectPeerAssociation extends pulumi.CustomResource {
    /**
     * Get an existing TransitGatewayConnectPeerAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TransitGatewayConnectPeerAssociationState, opts?: pulumi.CustomResourceOptions): TransitGatewayConnectPeerAssociation;
    /**
     * Returns true if the given object is an instance of TransitGatewayConnectPeerAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TransitGatewayConnectPeerAssociation;
    /**
     * ID of the device.
     */
    readonly deviceId: pulumi.Output<string>;
    /**
     * ID of the global network.
     */
    readonly globalNetworkId: pulumi.Output<string>;
    /**
     * ID of the link.
     */
    readonly linkId: pulumi.Output<string | undefined>;
    /**
     * ARN of the Connect peer.
     *
     * The following arguments are optional:
     */
    readonly transitGatewayConnectPeerArn: pulumi.Output<string>;
    /**
     * Create a TransitGatewayConnectPeerAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TransitGatewayConnectPeerAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TransitGatewayConnectPeerAssociation resources.
 */
export interface TransitGatewayConnectPeerAssociationState {
    /**
     * ID of the device.
     */
    deviceId?: pulumi.Input<string>;
    /**
     * ID of the global network.
     */
    globalNetworkId?: pulumi.Input<string>;
    /**
     * ID of the link.
     */
    linkId?: pulumi.Input<string>;
    /**
     * ARN of the Connect peer.
     *
     * The following arguments are optional:
     */
    transitGatewayConnectPeerArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TransitGatewayConnectPeerAssociation resource.
 */
export interface TransitGatewayConnectPeerAssociationArgs {
    /**
     * ID of the device.
     */
    deviceId: pulumi.Input<string>;
    /**
     * ID of the global network.
     */
    globalNetworkId: pulumi.Input<string>;
    /**
     * ID of the link.
     */
    linkId?: pulumi.Input<string>;
    /**
     * ARN of the Connect peer.
     *
     * The following arguments are optional:
     */
    transitGatewayConnectPeerArn: pulumi.Input<string>;
}
