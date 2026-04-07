import * as pulumi from "@pulumi/pulumi";
/**
 * Associates a Direct Connect Gateway with a VGW or transit gateway.
 *
 * To create a cross-account association, create an `aws.directconnect.GatewayAssociationProposal` resource
 * in the AWS account that owns the VGW or transit gateway and then accept the proposal in the AWS account that owns the Direct Connect Gateway
 * by creating an `aws.directconnect.GatewayAssociation` resource with the `proposalId` and `associatedGatewayOwnerAccountId` attributes set.
 *
 * ## Example Usage
 *
 * ### VPN Gateway Association
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.directconnect.Gateway("example", {
 *     name: "example",
 *     amazonSideAsn: "64512",
 * });
 * const exampleVpc = new aws.ec2.Vpc("example", {cidrBlock: "10.255.255.0/28"});
 * const exampleVpnGateway = new aws.ec2.VpnGateway("example", {vpcId: exampleVpc.id});
 * const exampleGatewayAssociation = new aws.directconnect.GatewayAssociation("example", {
 *     dxGatewayId: example.id,
 *     associatedGatewayId: exampleVpnGateway.id,
 * });
 * ```
 *
 * ### Transit Gateway Association
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.directconnect.Gateway("example", {
 *     name: "example",
 *     amazonSideAsn: "64512",
 * });
 * const exampleTransitGateway = new aws.ec2transitgateway.TransitGateway("example", {});
 * const exampleGatewayAssociation = new aws.directconnect.GatewayAssociation("example", {
 *     dxGatewayId: example.id,
 *     associatedGatewayId: exampleTransitGateway.id,
 *     allowedPrefixes: [
 *         "10.255.255.0/30",
 *         "10.255.255.8/30",
 *     ],
 * });
 * ```
 *
 * ### Allowed Prefixes
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.directconnect.Gateway("example", {
 *     name: "example",
 *     amazonSideAsn: "64512",
 * });
 * const exampleVpc = new aws.ec2.Vpc("example", {cidrBlock: "10.255.255.0/28"});
 * const exampleVpnGateway = new aws.ec2.VpnGateway("example", {vpcId: exampleVpc.id});
 * const exampleGatewayAssociation = new aws.directconnect.GatewayAssociation("example", {
 *     dxGatewayId: example.id,
 *     associatedGatewayId: exampleVpnGateway.id,
 *     allowedPrefixes: [
 *         "210.52.109.0/24",
 *         "175.45.176.0/22",
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Direct Connect gateway associations using `dx_gateway_id` together with `associated_gateway_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:directconnect/gatewayAssociation:GatewayAssociation example 345508c3-7215-4aef-9832-07c125d5bd0f/vgw-98765432
 * ```
 */
export declare class GatewayAssociation extends pulumi.CustomResource {
    /**
     * Get an existing GatewayAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GatewayAssociationState, opts?: pulumi.CustomResourceOptions): GatewayAssociation;
    /**
     * Returns true if the given object is an instance of GatewayAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is GatewayAssociation;
    /**
     * VPC prefixes (CIDRs) to advertise to the Direct Connect gateway. Defaults to the CIDR block of the VPC associated with the Virtual Gateway. To enable drift detection, must be configured.
     *
     * > **NOTE:** `dxGatewayId` and `associatedGatewayId` must be specified for single account Direct Connect gateway associations.
     *
     * > **NOTE:** If the `associatedGatewayId` is in another region, an alias in a new provider block for that region should be specified.
     */
    readonly allowedPrefixes: pulumi.Output<string[]>;
    /**
     * The ID of the VGW or transit gateway with which to associate the Direct Connect gateway.
     * Used for single account Direct Connect gateway associations.
     */
    readonly associatedGatewayId: pulumi.Output<string>;
    /**
     * The ID of the AWS account that owns the VGW or transit gateway with which to associate the Direct Connect gateway.
     * Used for cross-account Direct Connect gateway associations.
     */
    readonly associatedGatewayOwnerAccountId: pulumi.Output<string>;
    /**
     * The type of the associated gateway, `transitGateway` or `virtualPrivateGateway`.
     */
    readonly associatedGatewayType: pulumi.Output<string>;
    /**
     * The ID of the Direct Connect gateway association.
     */
    readonly dxGatewayAssociationId: pulumi.Output<string>;
    /**
     * The ID of the Direct Connect gateway.
     */
    readonly dxGatewayId: pulumi.Output<string>;
    /**
     * The ID of the AWS account that owns the Direct Connect gateway.
     */
    readonly dxGatewayOwnerAccountId: pulumi.Output<string>;
    /**
     * The ID of the Direct Connect gateway association proposal.
     * Used for cross-account Direct Connect gateway associations.
     */
    readonly proposalId: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the Transit Gateway Attachment when the type is `transitGateway`.
     */
    readonly transitGatewayAttachmentId: pulumi.Output<string>;
    /**
     * Create a GatewayAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: GatewayAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering GatewayAssociation resources.
 */
export interface GatewayAssociationState {
    /**
     * VPC prefixes (CIDRs) to advertise to the Direct Connect gateway. Defaults to the CIDR block of the VPC associated with the Virtual Gateway. To enable drift detection, must be configured.
     *
     * > **NOTE:** `dxGatewayId` and `associatedGatewayId` must be specified for single account Direct Connect gateway associations.
     *
     * > **NOTE:** If the `associatedGatewayId` is in another region, an alias in a new provider block for that region should be specified.
     */
    allowedPrefixes?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of the VGW or transit gateway with which to associate the Direct Connect gateway.
     * Used for single account Direct Connect gateway associations.
     */
    associatedGatewayId?: pulumi.Input<string>;
    /**
     * The ID of the AWS account that owns the VGW or transit gateway with which to associate the Direct Connect gateway.
     * Used for cross-account Direct Connect gateway associations.
     */
    associatedGatewayOwnerAccountId?: pulumi.Input<string>;
    /**
     * The type of the associated gateway, `transitGateway` or `virtualPrivateGateway`.
     */
    associatedGatewayType?: pulumi.Input<string>;
    /**
     * The ID of the Direct Connect gateway association.
     */
    dxGatewayAssociationId?: pulumi.Input<string>;
    /**
     * The ID of the Direct Connect gateway.
     */
    dxGatewayId?: pulumi.Input<string>;
    /**
     * The ID of the AWS account that owns the Direct Connect gateway.
     */
    dxGatewayOwnerAccountId?: pulumi.Input<string>;
    /**
     * The ID of the Direct Connect gateway association proposal.
     * Used for cross-account Direct Connect gateway associations.
     */
    proposalId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the Transit Gateway Attachment when the type is `transitGateway`.
     */
    transitGatewayAttachmentId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a GatewayAssociation resource.
 */
export interface GatewayAssociationArgs {
    /**
     * VPC prefixes (CIDRs) to advertise to the Direct Connect gateway. Defaults to the CIDR block of the VPC associated with the Virtual Gateway. To enable drift detection, must be configured.
     *
     * > **NOTE:** `dxGatewayId` and `associatedGatewayId` must be specified for single account Direct Connect gateway associations.
     *
     * > **NOTE:** If the `associatedGatewayId` is in another region, an alias in a new provider block for that region should be specified.
     */
    allowedPrefixes?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of the VGW or transit gateway with which to associate the Direct Connect gateway.
     * Used for single account Direct Connect gateway associations.
     */
    associatedGatewayId?: pulumi.Input<string>;
    /**
     * The ID of the AWS account that owns the VGW or transit gateway with which to associate the Direct Connect gateway.
     * Used for cross-account Direct Connect gateway associations.
     */
    associatedGatewayOwnerAccountId?: pulumi.Input<string>;
    /**
     * The ID of the Direct Connect gateway.
     */
    dxGatewayId: pulumi.Input<string>;
    /**
     * The ID of the Direct Connect gateway association proposal.
     * Used for cross-account Direct Connect gateway associations.
     */
    proposalId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
