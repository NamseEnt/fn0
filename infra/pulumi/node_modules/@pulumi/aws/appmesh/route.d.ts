import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS App Mesh route resource.
 *
 * ## Example Usage
 *
 * ### HTTP Routing
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const serviceb = new aws.appmesh.Route("serviceb", {
 *     name: "serviceB-route",
 *     meshName: simple.id,
 *     virtualRouterName: servicebAwsAppmeshVirtualRouter.name,
 *     spec: {
 *         httpRoute: {
 *             match: {
 *                 prefix: "/",
 *             },
 *             action: {
 *                 weightedTargets: [
 *                     {
 *                         virtualNode: serviceb1.name,
 *                         weight: 90,
 *                     },
 *                     {
 *                         virtualNode: serviceb2.name,
 *                         weight: 10,
 *                     },
 *                 ],
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ### HTTP Header Routing
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const serviceb = new aws.appmesh.Route("serviceb", {
 *     name: "serviceB-route",
 *     meshName: simple.id,
 *     virtualRouterName: servicebAwsAppmeshVirtualRouter.name,
 *     spec: {
 *         httpRoute: {
 *             match: {
 *                 method: "POST",
 *                 prefix: "/",
 *                 scheme: "https",
 *                 headers: [{
 *                     name: "clientRequestId",
 *                     match: {
 *                         prefix: "123",
 *                     },
 *                 }],
 *             },
 *             action: {
 *                 weightedTargets: [{
 *                     virtualNode: servicebAwsAppmeshVirtualNode.name,
 *                     weight: 100,
 *                 }],
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ### Retry Policy
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const serviceb = new aws.appmesh.Route("serviceb", {
 *     name: "serviceB-route",
 *     meshName: simple.id,
 *     virtualRouterName: servicebAwsAppmeshVirtualRouter.name,
 *     spec: {
 *         httpRoute: {
 *             match: {
 *                 prefix: "/",
 *             },
 *             retryPolicy: {
 *                 httpRetryEvents: ["server-error"],
 *                 maxRetries: 1,
 *                 perRetryTimeout: {
 *                     unit: "s",
 *                     value: 15,
 *                 },
 *             },
 *             action: {
 *                 weightedTargets: [{
 *                     virtualNode: servicebAwsAppmeshVirtualNode.name,
 *                     weight: 100,
 *                 }],
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ### TCP Routing
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const serviceb = new aws.appmesh.Route("serviceb", {
 *     name: "serviceB-route",
 *     meshName: simple.id,
 *     virtualRouterName: servicebAwsAppmeshVirtualRouter.name,
 *     spec: {
 *         tcpRoute: {
 *             action: {
 *                 weightedTargets: [{
 *                     virtualNode: serviceb1.name,
 *                     weight: 100,
 *                 }],
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import App Mesh virtual routes using `mesh_name` and `virtual_router_name` together with the route's `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:appmesh/route:Route serviceb simpleapp/serviceB/serviceB-route
 * ```
 */
export declare class Route extends pulumi.CustomResource {
    /**
     * Get an existing Route resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RouteState, opts?: pulumi.CustomResourceOptions): Route;
    /**
     * Returns true if the given object is an instance of Route.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Route;
    /**
     * ARN of the route.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Creation date of the route.
     */
    readonly createdDate: pulumi.Output<string>;
    /**
     * Last update date of the route.
     */
    readonly lastUpdatedDate: pulumi.Output<string>;
    /**
     * Name of the service mesh in which to create the route. Must be between 1 and 255 characters in length.
     */
    readonly meshName: pulumi.Output<string>;
    /**
     * AWS account ID of the service mesh's owner. Defaults to the account ID the AWS provider is currently connected to.
     */
    readonly meshOwner: pulumi.Output<string>;
    /**
     * Name to use for the route. Must be between 1 and 255 characters in length.
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
     * Route specification to apply.
     */
    readonly spec: pulumi.Output<outputs.appmesh.RouteSpec>;
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
     * Name of the virtual router in which to create the route. Must be between 1 and 255 characters in length.
     */
    readonly virtualRouterName: pulumi.Output<string>;
    /**
     * Create a Route resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RouteArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Route resources.
 */
export interface RouteState {
    /**
     * ARN of the route.
     */
    arn?: pulumi.Input<string>;
    /**
     * Creation date of the route.
     */
    createdDate?: pulumi.Input<string>;
    /**
     * Last update date of the route.
     */
    lastUpdatedDate?: pulumi.Input<string>;
    /**
     * Name of the service mesh in which to create the route. Must be between 1 and 255 characters in length.
     */
    meshName?: pulumi.Input<string>;
    /**
     * AWS account ID of the service mesh's owner. Defaults to the account ID the AWS provider is currently connected to.
     */
    meshOwner?: pulumi.Input<string>;
    /**
     * Name to use for the route. Must be between 1 and 255 characters in length.
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
     * Route specification to apply.
     */
    spec?: pulumi.Input<inputs.appmesh.RouteSpec>;
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
     * Name of the virtual router in which to create the route. Must be between 1 and 255 characters in length.
     */
    virtualRouterName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Route resource.
 */
export interface RouteArgs {
    /**
     * Name of the service mesh in which to create the route. Must be between 1 and 255 characters in length.
     */
    meshName: pulumi.Input<string>;
    /**
     * AWS account ID of the service mesh's owner. Defaults to the account ID the AWS provider is currently connected to.
     */
    meshOwner?: pulumi.Input<string>;
    /**
     * Name to use for the route. Must be between 1 and 255 characters in length.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Route specification to apply.
     */
    spec: pulumi.Input<inputs.appmesh.RouteSpec>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Name of the virtual router in which to create the route. Must be between 1 and 255 characters in length.
     */
    virtualRouterName: pulumi.Input<string>;
}
