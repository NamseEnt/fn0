import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS App Mesh virtual service resource.
 *
 * ## Example Usage
 *
 * ### Virtual Node Provider
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const servicea = new aws.appmesh.VirtualService("servicea", {
 *     name: "servicea.simpleapp.local",
 *     meshName: simple.id,
 *     spec: {
 *         provider: {
 *             virtualNode: {
 *                 virtualNodeName: serviceb1.name,
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ### Virtual Router Provider
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const servicea = new aws.appmesh.VirtualService("servicea", {
 *     name: "servicea.simpleapp.local",
 *     meshName: simple.id,
 *     spec: {
 *         provider: {
 *             virtualRouter: {
 *                 virtualRouterName: serviceb.name,
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import App Mesh virtual services using `mesh_name` together with the virtual service's `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:appmesh/virtualService:VirtualService servicea simpleapp/servicea.simpleapp.local
 * ```
 */
export declare class VirtualService extends pulumi.CustomResource {
    /**
     * Get an existing VirtualService resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VirtualServiceState, opts?: pulumi.CustomResourceOptions): VirtualService;
    /**
     * Returns true if the given object is an instance of VirtualService.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VirtualService;
    /**
     * ARN of the virtual service.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Creation date of the virtual service.
     */
    readonly createdDate: pulumi.Output<string>;
    /**
     * Last update date of the virtual service.
     */
    readonly lastUpdatedDate: pulumi.Output<string>;
    /**
     * Name of the service mesh in which to create the virtual service. Must be between 1 and 255 characters in length.
     */
    readonly meshName: pulumi.Output<string>;
    /**
     * AWS account ID of the service mesh's owner. Defaults to the account ID the AWS provider is currently connected to.
     */
    readonly meshOwner: pulumi.Output<string>;
    /**
     * Name to use for the virtual service. Must be between 1 and 255 characters in length.
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
     * Virtual service specification to apply.
     */
    readonly spec: pulumi.Output<outputs.appmesh.VirtualServiceSpec>;
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
     * Create a VirtualService resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VirtualServiceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VirtualService resources.
 */
export interface VirtualServiceState {
    /**
     * ARN of the virtual service.
     */
    arn?: pulumi.Input<string>;
    /**
     * Creation date of the virtual service.
     */
    createdDate?: pulumi.Input<string>;
    /**
     * Last update date of the virtual service.
     */
    lastUpdatedDate?: pulumi.Input<string>;
    /**
     * Name of the service mesh in which to create the virtual service. Must be between 1 and 255 characters in length.
     */
    meshName?: pulumi.Input<string>;
    /**
     * AWS account ID of the service mesh's owner. Defaults to the account ID the AWS provider is currently connected to.
     */
    meshOwner?: pulumi.Input<string>;
    /**
     * Name to use for the virtual service. Must be between 1 and 255 characters in length.
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
     * Virtual service specification to apply.
     */
    spec?: pulumi.Input<inputs.appmesh.VirtualServiceSpec>;
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
 * The set of arguments for constructing a VirtualService resource.
 */
export interface VirtualServiceArgs {
    /**
     * Name of the service mesh in which to create the virtual service. Must be between 1 and 255 characters in length.
     */
    meshName: pulumi.Input<string>;
    /**
     * AWS account ID of the service mesh's owner. Defaults to the account ID the AWS provider is currently connected to.
     */
    meshOwner?: pulumi.Input<string>;
    /**
     * Name to use for the virtual service. Must be between 1 and 255 characters in length.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Virtual service specification to apply.
     */
    spec: pulumi.Input<inputs.appmesh.VirtualServiceSpec>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
