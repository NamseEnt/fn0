import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS VPC NAT Gateway EIP Association.
 *
 * !> **WARNING:** You should not use the `aws.ec2.NatGatewayEipAssociation` resource in conjunction with an `aws.ec2.NatGateway` resource that has `secondaryAllocationIds` configured. Doing so may cause perpetual differences, and result in associations being overwritten.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.NatGatewayEipAssociation("example", {
 *     allocationId: exampleAwsEip.id,
 *     natGatewayId: exampleAwsNatGateway.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC NAT Gateway EIP Association using the `nat_gateway_id,allocation_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/natGatewayEipAssociation:NatGatewayEipAssociation example nat-1234567890abcdef1,eipalloc-1234567890abcdef1
 * ```
 */
export declare class NatGatewayEipAssociation extends pulumi.CustomResource {
    /**
     * Get an existing NatGatewayEipAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NatGatewayEipAssociationState, opts?: pulumi.CustomResourceOptions): NatGatewayEipAssociation;
    /**
     * Returns true if the given object is an instance of NatGatewayEipAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NatGatewayEipAssociation;
    /**
     * The ID of the Elastic IP Allocation to associate with the NAT Gateway.
     */
    readonly allocationId: pulumi.Output<string>;
    readonly associationId: pulumi.Output<string>;
    /**
     * The ID of the NAT Gateway to associate the Elastic IP Allocation to.
     *
     * The following arguments are optional:
     */
    readonly natGatewayId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.ec2.NatGatewayEipAssociationTimeouts | undefined>;
    /**
     * Create a NatGatewayEipAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NatGatewayEipAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NatGatewayEipAssociation resources.
 */
export interface NatGatewayEipAssociationState {
    /**
     * The ID of the Elastic IP Allocation to associate with the NAT Gateway.
     */
    allocationId?: pulumi.Input<string>;
    associationId?: pulumi.Input<string>;
    /**
     * The ID of the NAT Gateway to associate the Elastic IP Allocation to.
     *
     * The following arguments are optional:
     */
    natGatewayId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.ec2.NatGatewayEipAssociationTimeouts>;
}
/**
 * The set of arguments for constructing a NatGatewayEipAssociation resource.
 */
export interface NatGatewayEipAssociationArgs {
    /**
     * The ID of the Elastic IP Allocation to associate with the NAT Gateway.
     */
    allocationId: pulumi.Input<string>;
    /**
     * The ID of the NAT Gateway to associate the Elastic IP Allocation to.
     *
     * The following arguments are optional:
     */
    natGatewayId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.ec2.NatGatewayEipAssociationTimeouts>;
}
