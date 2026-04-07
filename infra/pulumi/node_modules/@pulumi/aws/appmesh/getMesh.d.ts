import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * The App Mesh Mesh data source allows details of an App Mesh Mesh to be retrieved by its name and optionally the mesh_owner.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const simple = aws.appmesh.getMesh({
 *     name: "simpleapp",
 * });
 * ```
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const simple = current.then(current => aws.appmesh.getMesh({
 *     name: "simpleapp",
 *     meshOwner: current.accountId,
 * }));
 * ```
 */
export declare function getMesh(args: GetMeshArgs, opts?: pulumi.InvokeOptions): Promise<GetMeshResult>;
/**
 * A collection of arguments for invoking getMesh.
 */
export interface GetMeshArgs {
    /**
     * AWS account ID of the service mesh's owner.
     */
    meshOwner?: string;
    /**
     * Name of the service mesh.
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
 * A collection of values returned by getMesh.
 */
export interface GetMeshResult {
    /**
     * ARN of the service mesh.
     */
    readonly arn: string;
    /**
     * Creation date of the service mesh.
     */
    readonly createdDate: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Last update date of the service mesh.
     */
    readonly lastUpdatedDate: string;
    readonly meshOwner: string;
    readonly name: string;
    readonly region: string;
    /**
     * Resource owner's AWS account ID.
     */
    readonly resourceOwner: string;
    /**
     * Service mesh specification. See the `aws.appmesh.Mesh` resource for details.
     */
    readonly specs: outputs.appmesh.GetMeshSpec[];
    /**
     * Map of tags.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * The App Mesh Mesh data source allows details of an App Mesh Mesh to be retrieved by its name and optionally the mesh_owner.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const simple = aws.appmesh.getMesh({
 *     name: "simpleapp",
 * });
 * ```
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const simple = current.then(current => aws.appmesh.getMesh({
 *     name: "simpleapp",
 *     meshOwner: current.accountId,
 * }));
 * ```
 */
export declare function getMeshOutput(args: GetMeshOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetMeshResult>;
/**
 * A collection of arguments for invoking getMesh.
 */
export interface GetMeshOutputArgs {
    /**
     * AWS account ID of the service mesh's owner.
     */
    meshOwner?: pulumi.Input<string>;
    /**
     * Name of the service mesh.
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
