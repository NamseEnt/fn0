import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * The App Mesh Virtual Router data source allows details of an App Mesh Virtual Service to be retrieved by its name and mesh_name.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.appmesh.getVirtualRouter({
 *     name: "example-router-name",
 *     meshName: "example-mesh-name",
 * });
 * ```
 */
export declare function getVirtualRouter(args: GetVirtualRouterArgs, opts?: pulumi.InvokeOptions): Promise<GetVirtualRouterResult>;
/**
 * A collection of arguments for invoking getVirtualRouter.
 */
export interface GetVirtualRouterArgs {
    /**
     * Name of the mesh in which the virtual router exists
     */
    meshName: string;
    meshOwner?: string;
    /**
     * Name of the virtual router.
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
}
/**
 * A collection of values returned by getVirtualRouter.
 */
export interface GetVirtualRouterResult {
    /**
     * ARN of the virtual router.
     */
    readonly arn: string;
    /**
     * Creation date of the virtual router.
     */
    readonly createdDate: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Last update date of the virtual router.
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
     * Virtual routers specification. See the `aws.appmesh.VirtualRouter` resource for details.
     */
    readonly specs: outputs.appmesh.GetVirtualRouterSpec[];
    /**
     * Map of tags.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * The App Mesh Virtual Router data source allows details of an App Mesh Virtual Service to be retrieved by its name and mesh_name.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.appmesh.getVirtualRouter({
 *     name: "example-router-name",
 *     meshName: "example-mesh-name",
 * });
 * ```
 */
export declare function getVirtualRouterOutput(args: GetVirtualRouterOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetVirtualRouterResult>;
/**
 * A collection of arguments for invoking getVirtualRouter.
 */
export interface GetVirtualRouterOutputArgs {
    /**
     * Name of the mesh in which the virtual router exists
     */
    meshName: pulumi.Input<string>;
    meshOwner?: pulumi.Input<string>;
    /**
     * Name of the virtual router.
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
}
