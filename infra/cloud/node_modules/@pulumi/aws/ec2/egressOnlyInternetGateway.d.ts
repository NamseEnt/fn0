import * as pulumi from "@pulumi/pulumi";
/**
 * [IPv6 only] Creates an egress-only Internet gateway for your VPC.
 * An egress-only Internet gateway is used to enable outbound communication
 * over IPv6 from instances in your VPC to the Internet, and prevents hosts
 * outside of your VPC from initiating an IPv6 connection with your instance.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.Vpc("example", {
 *     cidrBlock: "10.1.0.0/16",
 *     assignGeneratedIpv6CidrBlock: true,
 * });
 * const exampleEgressOnlyInternetGateway = new aws.ec2.EgressOnlyInternetGateway("example", {
 *     vpcId: example.id,
 *     tags: {
 *         Name: "main",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Egress-only Internet gateways using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/egressOnlyInternetGateway:EgressOnlyInternetGateway example eigw-015e0e244e24dfe8a
 * ```
 */
export declare class EgressOnlyInternetGateway extends pulumi.CustomResource {
    /**
     * Get an existing EgressOnlyInternetGateway resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EgressOnlyInternetGatewayState, opts?: pulumi.CustomResourceOptions): EgressOnlyInternetGateway;
    /**
     * Returns true if the given object is an instance of EgressOnlyInternetGateway.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EgressOnlyInternetGateway;
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
     * The VPC ID to create in.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a EgressOnlyInternetGateway resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EgressOnlyInternetGatewayArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EgressOnlyInternetGateway resources.
 */
export interface EgressOnlyInternetGatewayState {
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
     * The VPC ID to create in.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EgressOnlyInternetGateway resource.
 */
export interface EgressOnlyInternetGatewayArgs {
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
     * The VPC ID to create in.
     */
    vpcId: pulumi.Input<string>;
}
