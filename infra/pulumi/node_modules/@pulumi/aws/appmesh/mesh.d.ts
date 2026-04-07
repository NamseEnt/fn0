import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS App Mesh service mesh resource.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const simple = new aws.appmesh.Mesh("simple", {name: "simpleapp"});
 * ```
 *
 * ### Egress Filter
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const simple = new aws.appmesh.Mesh("simple", {
 *     name: "simpleapp",
 *     spec: {
 *         egressFilter: {
 *             type: "ALLOW_ALL",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import App Mesh service meshes using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:appmesh/mesh:Mesh simple simpleapp
 * ```
 */
export declare class Mesh extends pulumi.CustomResource {
    /**
     * Get an existing Mesh resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MeshState, opts?: pulumi.CustomResourceOptions): Mesh;
    /**
     * Returns true if the given object is an instance of Mesh.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Mesh;
    /**
     * ARN of the service mesh.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Creation date of the service mesh.
     */
    readonly createdDate: pulumi.Output<string>;
    /**
     * Last update date of the service mesh.
     */
    readonly lastUpdatedDate: pulumi.Output<string>;
    /**
     * AWS account ID of the service mesh's owner.
     */
    readonly meshOwner: pulumi.Output<string>;
    /**
     * Name to use for the service mesh. Must be between 1 and 255 characters in length.
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
     * Service mesh specification to apply.
     */
    readonly spec: pulumi.Output<outputs.appmesh.MeshSpec | undefined>;
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
     * Create a Mesh resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: MeshArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Mesh resources.
 */
export interface MeshState {
    /**
     * ARN of the service mesh.
     */
    arn?: pulumi.Input<string>;
    /**
     * Creation date of the service mesh.
     */
    createdDate?: pulumi.Input<string>;
    /**
     * Last update date of the service mesh.
     */
    lastUpdatedDate?: pulumi.Input<string>;
    /**
     * AWS account ID of the service mesh's owner.
     */
    meshOwner?: pulumi.Input<string>;
    /**
     * Name to use for the service mesh. Must be between 1 and 255 characters in length.
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
     * Service mesh specification to apply.
     */
    spec?: pulumi.Input<inputs.appmesh.MeshSpec>;
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
 * The set of arguments for constructing a Mesh resource.
 */
export interface MeshArgs {
    /**
     * Name to use for the service mesh. Must be between 1 and 255 characters in length.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Service mesh specification to apply.
     */
    spec?: pulumi.Input<inputs.appmesh.MeshSpec>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
