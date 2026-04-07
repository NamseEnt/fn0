import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an EC2 Carrier Gateway. See the AWS [documentation](https://docs.aws.amazon.com/vpc/latest/userguide/Carrier_Gateway.html) for more information.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.CarrierGateway("example", {
 *     vpcId: exampleAwsVpc.id,
 *     tags: {
 *         Name: "example-carrier-gateway",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_ec2_carrier_gateway` using the carrier gateway's ID. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/carrierGateway:CarrierGateway example cgw-12345
 * ```
 */
export declare class CarrierGateway extends pulumi.CustomResource {
    /**
     * Get an existing CarrierGateway resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CarrierGatewayState, opts?: pulumi.CustomResourceOptions): CarrierGateway;
    /**
     * Returns true if the given object is an instance of CarrierGateway.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CarrierGateway;
    /**
     * The ARN of the carrier gateway.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The AWS account ID of the owner of the carrier gateway.
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
    /**
     * The ID of the VPC to associate with the carrier gateway.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a CarrierGateway resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CarrierGatewayArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CarrierGateway resources.
 */
export interface CarrierGatewayState {
    /**
     * The ARN of the carrier gateway.
     */
    arn?: pulumi.Input<string>;
    /**
     * The AWS account ID of the owner of the carrier gateway.
     */
    ownerId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
    /**
     * The ID of the VPC to associate with the carrier gateway.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a CarrierGateway resource.
 */
export interface CarrierGatewayArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the VPC to associate with the carrier gateway.
     */
    vpcId: pulumi.Input<string>;
}
