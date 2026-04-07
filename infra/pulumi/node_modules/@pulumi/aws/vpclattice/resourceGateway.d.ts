import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS VPC Lattice Resource Gateway.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.ResourceGateway("example", {
 *     name: "Example",
 *     vpcId: exampleAwsVpc.id,
 *     subnetIds: [exampleAwsSubnet.id],
 *     tags: {
 *         Environment: "Example",
 *     },
 * });
 * ```
 *
 * ### Specifying IP address type
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.ResourceGateway("example", {
 *     name: "Example",
 *     vpcId: exampleAwsVpc.id,
 *     subnetIds: [exampleAwsSubnet.id],
 *     ipAddressType: "DUALSTACK",
 *     tags: {
 *         Environment: "Example",
 *     },
 * });
 * ```
 *
 * ### With security groups
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.ResourceGateway("example", {
 *     name: "Example",
 *     vpcId: exampleAwsVpc.id,
 *     securityGroupIds: [test.id],
 *     subnetIds: [exampleAwsSubnet.id],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC Lattice Resource Gateway using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:vpclattice/resourceGateway:ResourceGateway example rgw-0a1b2c3d4e5f
 * ```
 */
export declare class ResourceGateway extends pulumi.CustomResource {
    /**
     * Get an existing ResourceGateway resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResourceGatewayState, opts?: pulumi.CustomResourceOptions): ResourceGateway;
    /**
     * Returns true if the given object is an instance of ResourceGateway.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResourceGateway;
    /**
     * ARN of the resource gateway.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * IP address type used by the resource gateway. Valid values are `IPV4`, `IPV6`, and `DUALSTACK`. The IP address type of a resource gateway must be compatible with the subnets of the resource gateway and the IP address type of the resource.
     */
    readonly ipAddressType: pulumi.Output<string>;
    /**
     * The number of IPv4 addresses per ENI for your resource. This argument is only applicable to `IPV4` and `DUALSTACK` IP address types. Defaults to `16`.
     */
    readonly ipv4AddressesPerEni: pulumi.Output<number>;
    /**
     * Name of the resource gateway.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Security group IDs associated with the resource gateway. The security groups must be in the same VPC.
     */
    readonly securityGroupIds: pulumi.Output<string[]>;
    /**
     * Status of the resource gateway.
     */
    readonly status: pulumi.Output<string>;
    /**
     * IDs of the VPC subnets in which to create the resource gateway.
     */
    readonly subnetIds: pulumi.Output<string[]>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timeouts: pulumi.Output<outputs.vpclattice.ResourceGatewayTimeouts | undefined>;
    /**
     * ID of the VPC for the resource gateway.
     *
     * The following arguments are optional:
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a ResourceGateway resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResourceGatewayArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResourceGateway resources.
 */
export interface ResourceGatewayState {
    /**
     * ARN of the resource gateway.
     */
    arn?: pulumi.Input<string>;
    /**
     * IP address type used by the resource gateway. Valid values are `IPV4`, `IPV6`, and `DUALSTACK`. The IP address type of a resource gateway must be compatible with the subnets of the resource gateway and the IP address type of the resource.
     */
    ipAddressType?: pulumi.Input<string>;
    /**
     * The number of IPv4 addresses per ENI for your resource. This argument is only applicable to `IPV4` and `DUALSTACK` IP address types. Defaults to `16`.
     */
    ipv4AddressesPerEni?: pulumi.Input<number>;
    /**
     * Name of the resource gateway.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Security group IDs associated with the resource gateway. The security groups must be in the same VPC.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Status of the resource gateway.
     */
    status?: pulumi.Input<string>;
    /**
     * IDs of the VPC subnets in which to create the resource gateway.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timeouts?: pulumi.Input<inputs.vpclattice.ResourceGatewayTimeouts>;
    /**
     * ID of the VPC for the resource gateway.
     *
     * The following arguments are optional:
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ResourceGateway resource.
 */
export interface ResourceGatewayArgs {
    /**
     * IP address type used by the resource gateway. Valid values are `IPV4`, `IPV6`, and `DUALSTACK`. The IP address type of a resource gateway must be compatible with the subnets of the resource gateway and the IP address type of the resource.
     */
    ipAddressType?: pulumi.Input<string>;
    /**
     * The number of IPv4 addresses per ENI for your resource. This argument is only applicable to `IPV4` and `DUALSTACK` IP address types. Defaults to `16`.
     */
    ipv4AddressesPerEni?: pulumi.Input<number>;
    /**
     * Name of the resource gateway.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Security group IDs associated with the resource gateway. The security groups must be in the same VPC.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * IDs of the VPC subnets in which to create the resource gateway.
     */
    subnetIds: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.vpclattice.ResourceGatewayTimeouts>;
    /**
     * ID of the VPC for the resource gateway.
     *
     * The following arguments are optional:
     */
    vpcId: pulumi.Input<string>;
}
