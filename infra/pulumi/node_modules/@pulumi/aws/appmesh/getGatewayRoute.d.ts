import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * The App Mesh Gateway Route data source allows details of an App Mesh Gateway Route to be retrieved by its name, mesh_name, virtual_gateway_name, and optionally the mesh_owner.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.appmesh.getGatewayRoute({
 *     name: "test-route",
 *     meshName: "test-mesh",
 *     virtualGatewayName: "test-gateway",
 * });
 * ```
 */
export declare function getGatewayRoute(args: GetGatewayRouteArgs, opts?: pulumi.InvokeOptions): Promise<GetGatewayRouteResult>;
/**
 * A collection of arguments for invoking getGatewayRoute.
 */
export interface GetGatewayRouteArgs {
    /**
     * Name of the service mesh in which the virtual gateway exists.
     */
    meshName: string;
    /**
     * AWS account ID of the service mesh's owner.
     */
    meshOwner?: string;
    /**
     * Name of the gateway route.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags.
     */
    tags?: {
        [key: string]: string;
    };
    /**
     * Name of the virtual gateway in which the route exists.
     */
    virtualGatewayName: string;
}
/**
 * A collection of values returned by getGatewayRoute.
 */
export interface GetGatewayRouteResult {
    /**
     * ARN of the gateway route.
     */
    readonly arn: string;
    /**
     * Creation date of the gateway route.
     */
    readonly createdDate: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Last update date of the gateway route.
     */
    readonly lastUpdatedDate: string;
    readonly meshName: string;
    readonly meshOwner: string;
    readonly name: string;
    readonly region: string;
    /**
     * Resource owner's AWS account ID.
     */
    readonly resourceOwner: string;
    /**
     * Gateway route specification. See the `aws.appmesh.GatewayRoute` resource for details.
     */
    readonly specs: outputs.appmesh.GetGatewayRouteSpec[];
    /**
     * Map of tags.
     */
    readonly tags: {
        [key: string]: string;
    };
    readonly virtualGatewayName: string;
}
/**
 * The App Mesh Gateway Route data source allows details of an App Mesh Gateway Route to be retrieved by its name, mesh_name, virtual_gateway_name, and optionally the mesh_owner.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.appmesh.getGatewayRoute({
 *     name: "test-route",
 *     meshName: "test-mesh",
 *     virtualGatewayName: "test-gateway",
 * });
 * ```
 */
export declare function getGatewayRouteOutput(args: GetGatewayRouteOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetGatewayRouteResult>;
/**
 * A collection of arguments for invoking getGatewayRoute.
 */
export interface GetGatewayRouteOutputArgs {
    /**
     * Name of the service mesh in which the virtual gateway exists.
     */
    meshName: pulumi.Input<string>;
    /**
     * AWS account ID of the service mesh's owner.
     */
    meshOwner?: pulumi.Input<string>;
    /**
     * Name of the gateway route.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Name of the virtual gateway in which the route exists.
     */
    virtualGatewayName: pulumi.Input<string>;
}
