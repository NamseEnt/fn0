import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Direct Connect Gateway Association Proposal, typically for enabling cross-account associations. For single account associations, see the `aws.directconnect.GatewayAssociation` resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.directconnect.GatewayAssociationProposal("example", {
 *     dxGatewayId: exampleAwsDxGateway.id,
 *     dxGatewayOwnerAccountId: exampleAwsDxGateway.ownerAccountId,
 *     associatedGatewayId: exampleAwsVpnGateway.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using a proposal ID, Direct Connect Gateway ID and associated gateway ID separated by `/`:
 *
 * __With `pulumi import`__, import Direct Connect Gateway Association Proposals using either a proposal ID or proposal ID, Direct Connect Gateway ID and associated gateway ID separated by `/`. For example:
 *
 * Using a proposal ID:
 *
 * ```sh
 * $ pulumi import aws:directconnect/gatewayAssociationProposal:GatewayAssociationProposal example ac90e981-b718-4364-872d-65478c84fafe
 * ```
 * Using a proposal ID, Direct Connect Gateway ID and associated gateway ID separated by `/`:
 *
 * ```sh
 * $ pulumi import aws:directconnect/gatewayAssociationProposal:GatewayAssociationProposal example ac90e981-b718-4364-872d-65478c84fafe/abcd1234-dcba-5678-be23-cdef9876ab45/vgw-12345678
 * ```
 * The latter case is useful when a previous proposal has been accepted and deleted by AWS.
 * The `aws_dx_gateway_association_proposal` resource will then represent a pseudo-proposal for the same Direct Connect Gateway and associated gateway. If no previous proposal is available, use a tool like [`uuidgen`](http://manpages.ubuntu.com/manpages/bionic/man1/uuidgen.1.html) to generate a new random pseudo-proposal ID.
 */
export declare class GatewayAssociationProposal extends pulumi.CustomResource {
    /**
     * Get an existing GatewayAssociationProposal resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GatewayAssociationProposalState, opts?: pulumi.CustomResourceOptions): GatewayAssociationProposal;
    /**
     * Returns true if the given object is an instance of GatewayAssociationProposal.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is GatewayAssociationProposal;
    /**
     * VPC prefixes (CIDRs) to advertise to the Direct Connect gateway. Defaults to the CIDR block of the VPC associated with the Virtual Gateway. To enable drift detection, must be configured.
     */
    readonly allowedPrefixes: pulumi.Output<string[]>;
    /**
     * The ID of the VGW or transit gateway with which to associate the Direct Connect gateway.
     */
    readonly associatedGatewayId: pulumi.Output<string>;
    /**
     * The ID of the AWS account that owns the VGW or transit gateway with which to associate the Direct Connect gateway.
     */
    readonly associatedGatewayOwnerAccountId: pulumi.Output<string>;
    /**
     * The type of the associated gateway, `transitGateway` or `virtualPrivateGateway`.
     */
    readonly associatedGatewayType: pulumi.Output<string>;
    /**
     * Direct Connect Gateway identifier.
     */
    readonly dxGatewayId: pulumi.Output<string>;
    /**
     * AWS Account identifier of the Direct Connect Gateway's owner.
     */
    readonly dxGatewayOwnerAccountId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a GatewayAssociationProposal resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: GatewayAssociationProposalArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering GatewayAssociationProposal resources.
 */
export interface GatewayAssociationProposalState {
    /**
     * VPC prefixes (CIDRs) to advertise to the Direct Connect gateway. Defaults to the CIDR block of the VPC associated with the Virtual Gateway. To enable drift detection, must be configured.
     */
    allowedPrefixes?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of the VGW or transit gateway with which to associate the Direct Connect gateway.
     */
    associatedGatewayId?: pulumi.Input<string>;
    /**
     * The ID of the AWS account that owns the VGW or transit gateway with which to associate the Direct Connect gateway.
     */
    associatedGatewayOwnerAccountId?: pulumi.Input<string>;
    /**
     * The type of the associated gateway, `transitGateway` or `virtualPrivateGateway`.
     */
    associatedGatewayType?: pulumi.Input<string>;
    /**
     * Direct Connect Gateway identifier.
     */
    dxGatewayId?: pulumi.Input<string>;
    /**
     * AWS Account identifier of the Direct Connect Gateway's owner.
     */
    dxGatewayOwnerAccountId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a GatewayAssociationProposal resource.
 */
export interface GatewayAssociationProposalArgs {
    /**
     * VPC prefixes (CIDRs) to advertise to the Direct Connect gateway. Defaults to the CIDR block of the VPC associated with the Virtual Gateway. To enable drift detection, must be configured.
     */
    allowedPrefixes?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ID of the VGW or transit gateway with which to associate the Direct Connect gateway.
     */
    associatedGatewayId: pulumi.Input<string>;
    /**
     * Direct Connect Gateway identifier.
     */
    dxGatewayId: pulumi.Input<string>;
    /**
     * AWS Account identifier of the Direct Connect Gateway's owner.
     */
    dxGatewayOwnerAccountId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
