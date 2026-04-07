import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS App Mesh gateway route resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appmesh.GatewayRoute("example", {
 *     name: "example-gateway-route",
 *     meshName: "example-service-mesh",
 *     virtualGatewayName: exampleAwsAppmeshVirtualGateway.name,
 *     spec: {
 *         httpRoute: {
 *             action: {
 *                 target: {
 *                     virtualService: {
 *                         virtualServiceName: exampleAwsAppmeshVirtualService.name,
 *                     },
 *                 },
 *             },
 *             match: {
 *                 prefix: "/",
 *             },
 *         },
 *     },
 *     tags: {
 *         Environment: "test",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import App Mesh gateway routes using `mesh_name` and `virtual_gateway_name` together with the gateway route's `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:appmesh/gatewayRoute:GatewayRoute example mesh/gw1/example-gateway-route
 * ```
 */
export declare class GatewayRoute extends pulumi.CustomResource {
    /**
     * Get an existing GatewayRoute resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GatewayRouteState, opts?: pulumi.CustomResourceOptions): GatewayRoute;
    /**
     * Returns true if the given object is an instance of GatewayRoute.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is GatewayRoute;
    /**
     * ARN of the gateway route.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Creation date of the gateway route.
     */
    readonly createdDate: pulumi.Output<string>;
    /**
     * Last update date of the gateway route.
     */
    readonly lastUpdatedDate: pulumi.Output<string>;
    /**
     * Name of the service mesh in which to create the gateway route. Must be between 1 and 255 characters in length.
     */
    readonly meshName: pulumi.Output<string>;
    /**
     * AWS account ID of the service mesh's owner. Defaults to the account ID the AWS provider is currently connected to.
     */
    readonly meshOwner: pulumi.Output<string>;
    /**
     * Name to use for the gateway route. Must be between 1 and 255 characters in length.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Resource owner's AWS account ID.
     */
    readonly resourceOwner: pulumi.Output<string>;
    /**
     * Gateway route specification to apply.
     */
    readonly spec: pulumi.Output<outputs.appmesh.GatewayRouteSpec>;
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
     * Name of the virtual gateway to associate the gateway route with. Must be between 1 and 255 characters in length.
     */
    readonly virtualGatewayName: pulumi.Output<string>;
    /**
     * Create a GatewayRoute resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: GatewayRouteArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering GatewayRoute resources.
 */
export interface GatewayRouteState {
    /**
     * ARN of the gateway route.
     */
    arn?: pulumi.Input<string>;
    /**
     * Creation date of the gateway route.
     */
    createdDate?: pulumi.Input<string>;
    /**
     * Last update date of the gateway route.
     */
    lastUpdatedDate?: pulumi.Input<string>;
    /**
     * Name of the service mesh in which to create the gateway route. Must be between 1 and 255 characters in length.
     */
    meshName?: pulumi.Input<string>;
    /**
     * AWS account ID of the service mesh's owner. Defaults to the account ID the AWS provider is currently connected to.
     */
    meshOwner?: pulumi.Input<string>;
    /**
     * Name to use for the gateway route. Must be between 1 and 255 characters in length.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Resource owner's AWS account ID.
     */
    resourceOwner?: pulumi.Input<string>;
    /**
     * Gateway route specification to apply.
     */
    spec?: pulumi.Input<inputs.appmesh.GatewayRouteSpec>;
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
    /**
     * Name of the virtual gateway to associate the gateway route with. Must be between 1 and 255 characters in length.
     */
    virtualGatewayName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a GatewayRoute resource.
 */
export interface GatewayRouteArgs {
    /**
     * Name of the service mesh in which to create the gateway route. Must be between 1 and 255 characters in length.
     */
    meshName: pulumi.Input<string>;
    /**
     * AWS account ID of the service mesh's owner. Defaults to the account ID the AWS provider is currently connected to.
     */
    meshOwner?: pulumi.Input<string>;
    /**
     * Name to use for the gateway route. Must be between 1 and 255 characters in length.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Gateway route specification to apply.
     */
    spec: pulumi.Input<inputs.appmesh.GatewayRouteSpec>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Name of the virtual gateway to associate the gateway route with. Must be between 1 and 255 characters in length.
     */
    virtualGatewayName: pulumi.Input<string>;
}
