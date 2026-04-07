import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a subnet CIDR reservation resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.SubnetCidrReservation("example", {
 *     cidrBlock: "10.0.0.16/28",
 *     reservationType: "prefix",
 *     subnetId: exampleAwsSubnet.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Existing CIDR reservations using `SUBNET_ID:RESERVATION_ID`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/subnetCidrReservation:SubnetCidrReservation example subnet-01llsxvsxabqiymcz:scr-4mnvz6wb7otksjcs9
 * ```
 */
export declare class SubnetCidrReservation extends pulumi.CustomResource {
    /**
     * Get an existing SubnetCidrReservation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SubnetCidrReservationState, opts?: pulumi.CustomResourceOptions): SubnetCidrReservation;
    /**
     * Returns true if the given object is an instance of SubnetCidrReservation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SubnetCidrReservation;
    /**
     * The CIDR block for the reservation.
     */
    readonly cidrBlock: pulumi.Output<string>;
    /**
     * A brief description of the reservation.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * ID of the AWS account that owns this CIDR reservation.
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The type of reservation to create. Valid values: `explicit`, `prefix`
     */
    readonly reservationType: pulumi.Output<string>;
    /**
     * The ID of the subnet to create the reservation for.
     */
    readonly subnetId: pulumi.Output<string>;
    /**
     * Create a SubnetCidrReservation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SubnetCidrReservationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SubnetCidrReservation resources.
 */
export interface SubnetCidrReservationState {
    /**
     * The CIDR block for the reservation.
     */
    cidrBlock?: pulumi.Input<string>;
    /**
     * A brief description of the reservation.
     */
    description?: pulumi.Input<string>;
    /**
     * ID of the AWS account that owns this CIDR reservation.
     */
    ownerId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The type of reservation to create. Valid values: `explicit`, `prefix`
     */
    reservationType?: pulumi.Input<string>;
    /**
     * The ID of the subnet to create the reservation for.
     */
    subnetId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SubnetCidrReservation resource.
 */
export interface SubnetCidrReservationArgs {
    /**
     * The CIDR block for the reservation.
     */
    cidrBlock: pulumi.Input<string>;
    /**
     * A brief description of the reservation.
     */
    description?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The type of reservation to create. Valid values: `explicit`, `prefix`
     */
    reservationType: pulumi.Input<string>;
    /**
     * The ID of the subnet to create the reservation for.
     */
    subnetId: pulumi.Input<string>;
}
