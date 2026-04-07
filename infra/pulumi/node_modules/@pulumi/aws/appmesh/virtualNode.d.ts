import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS App Mesh virtual node resource.
 *
 * ## Breaking Changes
 *
 * Because of backward incompatible API changes (read [here](https://github.com/awslabs/aws-app-mesh-examples/issues/92)), `aws.appmesh.VirtualNode` resource definitions created with provider versions earlier than v2.3.0 will need to be modified:
 *
 * * Rename the `serviceName` attribute of the `dns` object to `hostname`.
 *
 * * Replace the `backends` attribute of the `spec` object with one or more `backend` configuration blocks,
 * setting `virtualServiceName` to the name of the service.
 *
 * The state associated with existing resources will automatically be migrated.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const serviceb1 = new aws.appmesh.VirtualNode("serviceb1", {
 *     name: "serviceBv1",
 *     meshName: simple.id,
 *     spec: {
 *         backends: [{
 *             virtualService: {
 *                 virtualServiceName: "servicea.simpleapp.local",
 *             },
 *         }],
 *         listeners: [{
 *             portMapping: {
 *                 port: 8080,
 *                 protocol: "http",
 *             },
 *         }],
 *         serviceDiscovery: {
 *             dns: {
 *                 hostname: "serviceb.simpleapp.local",
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ### AWS Cloud Map Service Discovery
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicediscovery.HttpNamespace("example", {name: "example-ns"});
 * const serviceb1 = new aws.appmesh.VirtualNode("serviceb1", {
 *     name: "serviceBv1",
 *     meshName: simple.id,
 *     spec: {
 *         backends: [{
 *             virtualService: {
 *                 virtualServiceName: "servicea.simpleapp.local",
 *             },
 *         }],
 *         listeners: [{
 *             portMapping: {
 *                 port: 8080,
 *                 protocol: "http",
 *             },
 *         }],
 *         serviceDiscovery: {
 *             awsCloudMap: {
 *                 attributes: {
 *                     stack: "blue",
 *                 },
 *                 serviceName: "serviceb1",
 *                 namespaceName: example.name,
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ### Listener Health Check
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const serviceb1 = new aws.appmesh.VirtualNode("serviceb1", {
 *     name: "serviceBv1",
 *     meshName: simple.id,
 *     spec: {
 *         backends: [{
 *             virtualService: {
 *                 virtualServiceName: "servicea.simpleapp.local",
 *             },
 *         }],
 *         listeners: [{
 *             portMapping: {
 *                 port: 8080,
 *                 protocol: "http",
 *             },
 *             healthCheck: {
 *                 protocol: "http",
 *                 path: "/ping",
 *                 healthyThreshold: 2,
 *                 unhealthyThreshold: 2,
 *                 timeoutMillis: 2000,
 *                 intervalMillis: 5000,
 *             },
 *         }],
 *         serviceDiscovery: {
 *             dns: {
 *                 hostname: "serviceb.simpleapp.local",
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ### Logging
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const serviceb1 = new aws.appmesh.VirtualNode("serviceb1", {
 *     name: "serviceBv1",
 *     meshName: simple.id,
 *     spec: {
 *         backends: [{
 *             virtualService: {
 *                 virtualServiceName: "servicea.simpleapp.local",
 *             },
 *         }],
 *         listeners: [{
 *             portMapping: {
 *                 port: 8080,
 *                 protocol: "http",
 *             },
 *         }],
 *         serviceDiscovery: {
 *             dns: {
 *                 hostname: "serviceb.simpleapp.local",
 *             },
 *         },
 *         logging: {
 *             accessLog: {
 *                 file: {
 *                     path: "/dev/stdout",
 *                 },
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import App Mesh virtual nodes using `mesh_name` together with the virtual node's `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:appmesh/virtualNode:VirtualNode serviceb1 simpleapp/serviceBv1
 * ```
 */
export declare class VirtualNode extends pulumi.CustomResource {
    /**
     * Get an existing VirtualNode resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VirtualNodeState, opts?: pulumi.CustomResourceOptions): VirtualNode;
    /**
     * Returns true if the given object is an instance of VirtualNode.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VirtualNode;
    /**
     * ARN of the virtual node.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Creation date of the virtual node.
     */
    readonly createdDate: pulumi.Output<string>;
    /**
     * Last update date of the virtual node.
     */
    readonly lastUpdatedDate: pulumi.Output<string>;
    /**
     * Name of the service mesh in which to create the virtual node. Must be between 1 and 255 characters in length.
     */
    readonly meshName: pulumi.Output<string>;
    /**
     * AWS account ID of the service mesh's owner. Defaults to the account ID the AWS provider is currently connected to.
     */
    readonly meshOwner: pulumi.Output<string>;
    /**
     * Name to use for the virtual node. Must be between 1 and 255 characters in length.
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
     * Virtual node specification to apply.
     */
    readonly spec: pulumi.Output<outputs.appmesh.VirtualNodeSpec>;
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
     * Create a VirtualNode resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VirtualNodeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VirtualNode resources.
 */
export interface VirtualNodeState {
    /**
     * ARN of the virtual node.
     */
    arn?: pulumi.Input<string>;
    /**
     * Creation date of the virtual node.
     */
    createdDate?: pulumi.Input<string>;
    /**
     * Last update date of the virtual node.
     */
    lastUpdatedDate?: pulumi.Input<string>;
    /**
     * Name of the service mesh in which to create the virtual node. Must be between 1 and 255 characters in length.
     */
    meshName?: pulumi.Input<string>;
    /**
     * AWS account ID of the service mesh's owner. Defaults to the account ID the AWS provider is currently connected to.
     */
    meshOwner?: pulumi.Input<string>;
    /**
     * Name to use for the virtual node. Must be between 1 and 255 characters in length.
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
     * Virtual node specification to apply.
     */
    spec?: pulumi.Input<inputs.appmesh.VirtualNodeSpec>;
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
 * The set of arguments for constructing a VirtualNode resource.
 */
export interface VirtualNodeArgs {
    /**
     * Name of the service mesh in which to create the virtual node. Must be between 1 and 255 characters in length.
     */
    meshName: pulumi.Input<string>;
    /**
     * AWS account ID of the service mesh's owner. Defaults to the account ID the AWS provider is currently connected to.
     */
    meshOwner?: pulumi.Input<string>;
    /**
     * Name to use for the virtual node. Must be between 1 and 255 characters in length.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Virtual node specification to apply.
     */
    spec: pulumi.Input<inputs.appmesh.VirtualNodeSpec>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
