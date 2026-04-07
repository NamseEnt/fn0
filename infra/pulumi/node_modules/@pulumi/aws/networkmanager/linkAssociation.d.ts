import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Network Manager link association. Associates a link to a device. A device can be associated to multiple links and a link can be associated to multiple devices. The device and link must be in the same global network and the same site.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.networkmanager.LinkAssociation("example", {
 *     globalNetworkId: exampleAwsNetworkmanagerGlobalNetwork.id,
 *     linkId: exampleAwsNetworkmanagerLink.id,
 *     deviceId: exampleAwsNetworkmanagerDevice.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_networkmanager_link_association` using the global network ID, link ID and device ID. For example:
 *
 * ```sh
 * $ pulumi import aws:networkmanager/linkAssociation:LinkAssociation example global-network-0d47f6t230mz46dy4,link-444555aaabbb11223,device-07f6fd08867abc123
 * ```
 */
export declare class LinkAssociation extends pulumi.CustomResource {
    /**
     * Get an existing LinkAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LinkAssociationState, opts?: pulumi.CustomResourceOptions): LinkAssociation;
    /**
     * Returns true if the given object is an instance of LinkAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LinkAssociation;
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
    readonly linkId: pulumi.Output<string>;
    /**
     * Create a LinkAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LinkAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LinkAssociation resources.
 */
export interface LinkAssociationState {
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
}
/**
 * The set of arguments for constructing a LinkAssociation resource.
 */
export interface LinkAssociationArgs {
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
    linkId: pulumi.Input<string>;
}
