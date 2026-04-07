import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource for managing propagation between VPC (Virtual Private Cloud) route server and a route table.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpc.RouteServerPropagation("example", {
 *     routeServerId: exampleAwsVpcRouteServer.routeServerId,
 *     routeTableId: exampleAwsRouteTable.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, to  to import VPC (Virtual Private Cloud) Route Server Propagation using the associated resource ID and route table ID separated by a comma (,). For example:
 *
 * ```sh
 * $ pulumi import aws:vpc/routeServerPropagation:RouteServerPropagation example rs-12345678,rtb-656c65616e6f72
 * ```
 */
export declare class RouteServerPropagation extends pulumi.CustomResource {
    /**
     * Get an existing RouteServerPropagation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RouteServerPropagationState, opts?: pulumi.CustomResourceOptions): RouteServerPropagation;
    /**
     * Returns true if the given object is an instance of RouteServerPropagation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RouteServerPropagation;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The unique identifier for the route server to be associated.
     */
    readonly routeServerId: pulumi.Output<string>;
    /**
     * The ID of the route table to which route server will propagate routes.
     *
     * The following arguments are optional:
     */
    readonly routeTableId: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.vpc.RouteServerPropagationTimeouts | undefined>;
    /**
     * Create a RouteServerPropagation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RouteServerPropagationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RouteServerPropagation resources.
 */
export interface RouteServerPropagationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The unique identifier for the route server to be associated.
     */
    routeServerId?: pulumi.Input<string>;
    /**
     * The ID of the route table to which route server will propagate routes.
     *
     * The following arguments are optional:
     */
    routeTableId?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.vpc.RouteServerPropagationTimeouts>;
}
/**
 * The set of arguments for constructing a RouteServerPropagation resource.
 */
export interface RouteServerPropagationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The unique identifier for the route server to be associated.
     */
    routeServerId: pulumi.Input<string>;
    /**
     * The ID of the route table to which route server will propagate routes.
     *
     * The following arguments are optional:
     */
    routeTableId: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.vpc.RouteServerPropagationTimeouts>;
}
