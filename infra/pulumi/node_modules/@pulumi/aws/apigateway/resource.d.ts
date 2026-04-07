import * as pulumi from "@pulumi/pulumi";
import { RestApi } from "./index";
/**
 * Provides an API Gateway Resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const myDemoAPI = new aws.apigateway.RestApi("MyDemoAPI", {
 *     name: "MyDemoAPI",
 *     description: "This is my API for demonstration purposes",
 * });
 * const myDemoResource = new aws.apigateway.Resource("MyDemoResource", {
 *     restApi: myDemoAPI.id,
 *     parentId: myDemoAPI.rootResourceId,
 *     pathPart: "mydemoresource",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_api_gateway_resource` using `REST-API-ID/RESOURCE-ID`. For example:
 *
 * ```sh
 * $ pulumi import aws:apigateway/resource:Resource example 12345abcde/67890fghij
 * ```
 */
export declare class Resource extends pulumi.CustomResource {
    /**
     * Get an existing Resource resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResourceState, opts?: pulumi.CustomResourceOptions): Resource;
    /**
     * Returns true if the given object is an instance of Resource.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Resource;
    /**
     * ID of the parent API resource
     */
    readonly parentId: pulumi.Output<string>;
    /**
     * Complete path for this API resource, including all parent paths.
     */
    readonly path: pulumi.Output<string>;
    /**
     * Last path segment of this API resource.
     */
    readonly pathPart: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID of the associated REST API
     */
    readonly restApi: pulumi.Output<string>;
    /**
     * Create a Resource resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResourceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Resource resources.
 */
export interface ResourceState {
    /**
     * ID of the parent API resource
     */
    parentId?: pulumi.Input<string>;
    /**
     * Complete path for this API resource, including all parent paths.
     */
    path?: pulumi.Input<string>;
    /**
     * Last path segment of this API resource.
     */
    pathPart?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the associated REST API
     */
    restApi?: pulumi.Input<string | RestApi>;
}
/**
 * The set of arguments for constructing a Resource resource.
 */
export interface ResourceArgs {
    /**
     * ID of the parent API resource
     */
    parentId: pulumi.Input<string>;
    /**
     * Last path segment of this API resource.
     */
    pathPart: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the associated REST API
     */
    restApi: pulumi.Input<string | RestApi>;
}
