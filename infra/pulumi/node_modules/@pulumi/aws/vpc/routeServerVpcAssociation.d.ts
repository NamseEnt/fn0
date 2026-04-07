import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource for managing association between VPC (Virtual Private Cloud) route server and a VPC.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpc.RouteServerVpcAssociation("example", {
 *     routeServerId: exampleAwsVpcRouteServer.routeServerId,
 *     vpcId: exampleAwsVpc.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, to  to import VPC (Virtual Private Cloud) Route Server Association using the associated resource ID and VPC Id separated by a comma (,). For example:
 *
 * ```sh
 * $ pulumi import aws:vpc/routeServerVpcAssociation:RouteServerVpcAssociation example rs-12345678,vpc-0f001273ec18911b1
 * ```
 */
export declare class RouteServerVpcAssociation extends pulumi.CustomResource {
    /**
     * Get an existing RouteServerVpcAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RouteServerVpcAssociationState, opts?: pulumi.CustomResourceOptions): RouteServerVpcAssociation;
    /**
     * Returns true if the given object is an instance of RouteServerVpcAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RouteServerVpcAssociation;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The unique identifier for the route server to be associated.
     */
    readonly routeServerId: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.vpc.RouteServerVpcAssociationTimeouts | undefined>;
    /**
     * The ID of the VPC to associate with the route server.
     *
     * The following arguments are optional:
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a RouteServerVpcAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RouteServerVpcAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RouteServerVpcAssociation resources.
 */
export interface RouteServerVpcAssociationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The unique identifier for the route server to be associated.
     */
    routeServerId?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.vpc.RouteServerVpcAssociationTimeouts>;
    /**
     * The ID of the VPC to associate with the route server.
     *
     * The following arguments are optional:
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RouteServerVpcAssociation resource.
 */
export interface RouteServerVpcAssociationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The unique identifier for the route server to be associated.
     */
    routeServerId: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.vpc.RouteServerVpcAssociationTimeouts>;
    /**
     * The ID of the VPC to associate with the route server.
     *
     * The following arguments are optional:
     */
    vpcId: pulumi.Input<string>;
}
