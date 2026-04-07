import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS App Mesh virtual gateway resource.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appmesh.VirtualGateway("example", {
 *     name: "example-virtual-gateway",
 *     meshName: "example-service-mesh",
 *     spec: {
 *         listeners: [{
 *             portMapping: {
 *                 port: 8080,
 *                 protocol: "http",
 *             },
 *         }],
 *     },
 *     tags: {
 *         Environment: "test",
 *     },
 * });
 * ```
 *
 * ### Access Logs and TLS
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appmesh.VirtualGateway("example", {
 *     name: "example-virtual-gateway",
 *     meshName: "example-service-mesh",
 *     spec: {
 *         listeners: [{
 *             portMapping: {
 *                 port: 8080,
 *                 protocol: "http",
 *             },
 *             tls: {
 *                 certificate: {
 *                     acm: {
 *                         certificateArn: exampleAwsAcmCertificate.arn,
 *                     },
 *                 },
 *                 mode: "STRICT",
 *             },
 *         }],
 *         logging: {
 *             accessLog: {
 *                 file: {
 *                     path: "/var/log/access.log",
 *                 },
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import App Mesh virtual gateway using `mesh_name` together with the virtual gateway's `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:appmesh/virtualGateway:VirtualGateway example mesh/gw1
 * ```
 */
export declare class VirtualGateway extends pulumi.CustomResource {
    /**
     * Get an existing VirtualGateway resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VirtualGatewayState, opts?: pulumi.CustomResourceOptions): VirtualGateway;
    /**
     * Returns true if the given object is an instance of VirtualGateway.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VirtualGateway;
    /**
     * ARN of the virtual gateway.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Creation date of the virtual gateway.
     */
    readonly createdDate: pulumi.Output<string>;
    /**
     * Last update date of the virtual gateway.
     */
    readonly lastUpdatedDate: pulumi.Output<string>;
    /**
     * Name of the service mesh in which to create the virtual gateway. Must be between 1 and 255 characters in length.
     */
    readonly meshName: pulumi.Output<string>;
    /**
     * AWS account ID of the service mesh's owner. Defaults to the account ID the AWS provider is currently connected to.
     */
    readonly meshOwner: pulumi.Output<string>;
    /**
     * Name to use for the virtual gateway. Must be between 1 and 255 characters in length.
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
     * Virtual gateway specification to apply.
     */
    readonly spec: pulumi.Output<outputs.appmesh.VirtualGatewaySpec>;
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
     * Create a VirtualGateway resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VirtualGatewayArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VirtualGateway resources.
 */
export interface VirtualGatewayState {
    /**
     * ARN of the virtual gateway.
     */
    arn?: pulumi.Input<string>;
    /**
     * Creation date of the virtual gateway.
     */
    createdDate?: pulumi.Input<string>;
    /**
     * Last update date of the virtual gateway.
     */
    lastUpdatedDate?: pulumi.Input<string>;
    /**
     * Name of the service mesh in which to create the virtual gateway. Must be between 1 and 255 characters in length.
     */
    meshName?: pulumi.Input<string>;
    /**
     * AWS account ID of the service mesh's owner. Defaults to the account ID the AWS provider is currently connected to.
     */
    meshOwner?: pulumi.Input<string>;
    /**
     * Name to use for the virtual gateway. Must be between 1 and 255 characters in length.
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
     * Virtual gateway specification to apply.
     */
    spec?: pulumi.Input<inputs.appmesh.VirtualGatewaySpec>;
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
 * The set of arguments for constructing a VirtualGateway resource.
 */
export interface VirtualGatewayArgs {
    /**
     * Name of the service mesh in which to create the virtual gateway. Must be between 1 and 255 characters in length.
     */
    meshName: pulumi.Input<string>;
    /**
     * AWS account ID of the service mesh's owner. Defaults to the account ID the AWS provider is currently connected to.
     */
    meshOwner?: pulumi.Input<string>;
    /**
     * Name to use for the virtual gateway. Must be between 1 and 255 characters in length.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Virtual gateway specification to apply.
     */
    spec: pulumi.Input<inputs.appmesh.VirtualGatewaySpec>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
