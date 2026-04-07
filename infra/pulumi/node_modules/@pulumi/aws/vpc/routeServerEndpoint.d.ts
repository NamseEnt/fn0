import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource for managing a VPC (Virtual Private Cloud) Route Server Endpoint.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.vpc.RouteServerEndpoint("test", {
 *     routeServerId: example.routeServerId,
 *     subnetId: main.id,
 *     tags: {
 *         Name: "Endpoint A",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC (Virtual Private Cloud) Route Server Endpoint using the `route_server_endpoint_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:vpc/routeServerEndpoint:RouteServerEndpoint example rse-12345678
 * ```
 */
export declare class RouteServerEndpoint extends pulumi.CustomResource {
    /**
     * Get an existing RouteServerEndpoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RouteServerEndpointState, opts?: pulumi.CustomResourceOptions): RouteServerEndpoint;
    /**
     * Returns true if the given object is an instance of RouteServerEndpoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RouteServerEndpoint;
    /**
     * The ARN of the route server endpoint.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The IP address of the Elastic network interface for the endpoint.
     */
    readonly eniAddress: pulumi.Output<string>;
    /**
     * The ID of the Elastic network interface for the endpoint.
     */
    readonly eniId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The unique identifier of the route server endpoint.
     */
    readonly routeServerEndpointId: pulumi.Output<string>;
    /**
     * The ID of the route server for which to create an endpoint.
     */
    readonly routeServerId: pulumi.Output<string>;
    /**
     * The ID of the subnet in which to create the route server endpoint.
     *
     * The following arguments are optional:
     */
    readonly subnetId: pulumi.Output<string>;
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
    readonly timeouts: pulumi.Output<outputs.vpc.RouteServerEndpointTimeouts | undefined>;
    /**
     * The ID of the VPC containing the endpoint.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a RouteServerEndpoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RouteServerEndpointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RouteServerEndpoint resources.
 */
export interface RouteServerEndpointState {
    /**
     * The ARN of the route server endpoint.
     */
    arn?: pulumi.Input<string>;
    /**
     * The IP address of the Elastic network interface for the endpoint.
     */
    eniAddress?: pulumi.Input<string>;
    /**
     * The ID of the Elastic network interface for the endpoint.
     */
    eniId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The unique identifier of the route server endpoint.
     */
    routeServerEndpointId?: pulumi.Input<string>;
    /**
     * The ID of the route server for which to create an endpoint.
     */
    routeServerId?: pulumi.Input<string>;
    /**
     * The ID of the subnet in which to create the route server endpoint.
     *
     * The following arguments are optional:
     */
    subnetId?: pulumi.Input<string>;
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
    timeouts?: pulumi.Input<inputs.vpc.RouteServerEndpointTimeouts>;
    /**
     * The ID of the VPC containing the endpoint.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RouteServerEndpoint resource.
 */
export interface RouteServerEndpointArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the route server for which to create an endpoint.
     */
    routeServerId: pulumi.Input<string>;
    /**
     * The ID of the subnet in which to create the route server endpoint.
     *
     * The following arguments are optional:
     */
    subnetId: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.vpc.RouteServerEndpointTimeouts>;
}
