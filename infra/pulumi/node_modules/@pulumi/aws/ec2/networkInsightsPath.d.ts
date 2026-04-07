import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Network Insights Path resource. Part of the "Reachability Analyzer" service in the AWS VPC console.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.ec2.NetworkInsightsPath("test", {
 *     source: source.id,
 *     destination: destination.id,
 *     protocol: "tcp",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Network Insights Paths using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/networkInsightsPath:NetworkInsightsPath test nip-00edfba169923aefd
 * ```
 */
export declare class NetworkInsightsPath extends pulumi.CustomResource {
    /**
     * Get an existing NetworkInsightsPath resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NetworkInsightsPathState, opts?: pulumi.CustomResourceOptions): NetworkInsightsPath;
    /**
     * Returns true if the given object is an instance of NetworkInsightsPath.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is NetworkInsightsPath;
    /**
     * ARN of the Network Insights Path.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ID or ARN of the resource which is the destination of the path. Can be an Instance, Internet Gateway, Network Interface, Transit Gateway, VPC Endpoint, VPC Peering Connection or VPN Gateway. If the resource is in another account, you must specify an ARN. Either the `destination` argument or the `destinationAddress` argument in the `filterAtSource` block must be specified.
     */
    readonly destination: pulumi.Output<string | undefined>;
    /**
     * ARN of the destination.
     */
    readonly destinationArn: pulumi.Output<string>;
    /**
     * IP address of the destination resource.
     */
    readonly destinationIp: pulumi.Output<string | undefined>;
    /**
     * Destination port to analyze access to.
     */
    readonly destinationPort: pulumi.Output<number | undefined>;
    readonly filterAtDestination: pulumi.Output<outputs.ec2.NetworkInsightsPathFilterAtDestination>;
    readonly filterAtSource: pulumi.Output<outputs.ec2.NetworkInsightsPathFilterAtSource>;
    /**
     * Protocol to use for analysis. Valid options are `tcp` or `udp`.
     *
     * The following arguments are optional:
     */
    readonly protocol: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID or ARN of the resource which is the source of the path. Can be an Instance, Internet Gateway, Network Interface, Transit Gateway, VPC Endpoint, VPC Peering Connection or VPN Gateway. If the resource is in another account, you must specify an ARN.
     */
    readonly source: pulumi.Output<string>;
    /**
     * ARN of the source.
     */
    readonly sourceArn: pulumi.Output<string>;
    /**
     * IP address of the source resource.
     */
    readonly sourceIp: pulumi.Output<string | undefined>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a NetworkInsightsPath resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NetworkInsightsPathArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering NetworkInsightsPath resources.
 */
export interface NetworkInsightsPathState {
    /**
     * ARN of the Network Insights Path.
     */
    arn?: pulumi.Input<string>;
    /**
     * ID or ARN of the resource which is the destination of the path. Can be an Instance, Internet Gateway, Network Interface, Transit Gateway, VPC Endpoint, VPC Peering Connection or VPN Gateway. If the resource is in another account, you must specify an ARN. Either the `destination` argument or the `destinationAddress` argument in the `filterAtSource` block must be specified.
     */
    destination?: pulumi.Input<string>;
    /**
     * ARN of the destination.
     */
    destinationArn?: pulumi.Input<string>;
    /**
     * IP address of the destination resource.
     */
    destinationIp?: pulumi.Input<string>;
    /**
     * Destination port to analyze access to.
     */
    destinationPort?: pulumi.Input<number>;
    filterAtDestination?: pulumi.Input<inputs.ec2.NetworkInsightsPathFilterAtDestination>;
    filterAtSource?: pulumi.Input<inputs.ec2.NetworkInsightsPathFilterAtSource>;
    /**
     * Protocol to use for analysis. Valid options are `tcp` or `udp`.
     *
     * The following arguments are optional:
     */
    protocol?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID or ARN of the resource which is the source of the path. Can be an Instance, Internet Gateway, Network Interface, Transit Gateway, VPC Endpoint, VPC Peering Connection or VPN Gateway. If the resource is in another account, you must specify an ARN.
     */
    source?: pulumi.Input<string>;
    /**
     * ARN of the source.
     */
    sourceArn?: pulumi.Input<string>;
    /**
     * IP address of the source resource.
     */
    sourceIp?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a NetworkInsightsPath resource.
 */
export interface NetworkInsightsPathArgs {
    /**
     * ID or ARN of the resource which is the destination of the path. Can be an Instance, Internet Gateway, Network Interface, Transit Gateway, VPC Endpoint, VPC Peering Connection or VPN Gateway. If the resource is in another account, you must specify an ARN. Either the `destination` argument or the `destinationAddress` argument in the `filterAtSource` block must be specified.
     */
    destination?: pulumi.Input<string>;
    /**
     * IP address of the destination resource.
     */
    destinationIp?: pulumi.Input<string>;
    /**
     * Destination port to analyze access to.
     */
    destinationPort?: pulumi.Input<number>;
    filterAtDestination?: pulumi.Input<inputs.ec2.NetworkInsightsPathFilterAtDestination>;
    filterAtSource?: pulumi.Input<inputs.ec2.NetworkInsightsPathFilterAtSource>;
    /**
     * Protocol to use for analysis. Valid options are `tcp` or `udp`.
     *
     * The following arguments are optional:
     */
    protocol: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID or ARN of the resource which is the source of the path. Can be an Instance, Internet Gateway, Network Interface, Transit Gateway, VPC Endpoint, VPC Peering Connection or VPN Gateway. If the resource is in another account, you must specify an ARN.
     */
    source: pulumi.Input<string>;
    /**
     * IP address of the source resource.
     */
    sourceIp?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
