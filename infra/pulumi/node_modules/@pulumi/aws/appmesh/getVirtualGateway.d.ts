import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS App Mesh Virtual Gateway.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.appmesh.getVirtualGateway({
 *     meshName: "mesh-gateway",
 *     name: "example-mesh",
 * });
 * ```
 */
export declare function getVirtualGateway(args: GetVirtualGatewayArgs, opts?: pulumi.InvokeOptions): Promise<GetVirtualGatewayResult>;
/**
 * A collection of arguments for invoking getVirtualGateway.
 */
export interface GetVirtualGatewayArgs {
    /**
     * Name of the service mesh in which the virtual gateway exists.
     */
    meshName: string;
    /**
     * Name of the virtual gateway.
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
 * A collection of values returned by getVirtualGateway.
 */
export interface GetVirtualGatewayResult {
    /**
     * ARN of the virtual gateway.
     */
    readonly arn: string;
    /**
     * Creation date of the virtual gateway.
     */
    readonly createdDate: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Last update date of the virtual gateway.
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
     * Virtual gateway specification. See the `aws.appmesh.VirtualGateway` resource for details.
     */
    readonly specs: outputs.appmesh.GetVirtualGatewaySpec[];
    /**
     * Map of tags.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Data source for managing an AWS App Mesh Virtual Gateway.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.appmesh.getVirtualGateway({
 *     meshName: "mesh-gateway",
 *     name: "example-mesh",
 * });
 * ```
 */
export declare function getVirtualGatewayOutput(args: GetVirtualGatewayOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetVirtualGatewayResult>;
/**
 * A collection of arguments for invoking getVirtualGateway.
 */
export interface GetVirtualGatewayOutputArgs {
    /**
     * Name of the service mesh in which the virtual gateway exists.
     */
    meshName: pulumi.Input<string>;
    /**
     * Name of the virtual gateway.
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
