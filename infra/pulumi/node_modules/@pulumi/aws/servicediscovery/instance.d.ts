import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Service Discovery Instance resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.Vpc("example", {
 *     cidrBlock: "10.0.0.0/16",
 *     enableDnsSupport: true,
 *     enableDnsHostnames: true,
 * });
 * const examplePrivateDnsNamespace = new aws.servicediscovery.PrivateDnsNamespace("example", {
 *     name: "example.domain.local",
 *     description: "example",
 *     vpc: example.id,
 * });
 * const exampleService = new aws.servicediscovery.Service("example", {
 *     name: "example",
 *     dnsConfig: {
 *         namespaceId: examplePrivateDnsNamespace.id,
 *         dnsRecords: [{
 *             ttl: 10,
 *             type: "A",
 *         }],
 *         routingPolicy: "MULTIVALUE",
 *     },
 *     healthCheckCustomConfig: {
 *         failureThreshold: 1,
 *     },
 * });
 * const exampleInstance = new aws.servicediscovery.Instance("example", {
 *     instanceId: "example-instance-id",
 *     serviceId: exampleService.id,
 *     attributes: {
 *         AWS_INSTANCE_IPV4: "172.18.0.1",
 *         custom_attribute: "custom",
 *     },
 * });
 * ```
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicediscovery.HttpNamespace("example", {
 *     name: "example.domain.test",
 *     description: "example",
 * });
 * const exampleService = new aws.servicediscovery.Service("example", {
 *     name: "example",
 *     namespaceId: example.id,
 * });
 * const exampleInstance = new aws.servicediscovery.Instance("example", {
 *     instanceId: "example-instance-id",
 *     serviceId: exampleService.id,
 *     attributes: {
 *         AWS_EC2_INSTANCE_ID: "i-0abdg374kd892cj6dl",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Service Discovery Instance using the service ID and instance ID. For example:
 *
 * ```sh
 * $ pulumi import aws:servicediscovery/instance:Instance example 0123456789/i-0123
 * ```
 */
export declare class Instance extends pulumi.CustomResource {
    /**
     * Get an existing Instance resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InstanceState, opts?: pulumi.CustomResourceOptions): Instance;
    /**
     * Returns true if the given object is an instance of Instance.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Instance;
    /**
     * A map contains the attributes of the instance. Check the [doc](https://docs.aws.amazon.com/cloud-map/latest/api/API_RegisterInstance.html#API_RegisterInstance_RequestSyntax) for the supported attributes and syntax.
     */
    readonly attributes: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The ID of the service instance.
     */
    readonly instanceId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the service that you want to use to create the instance.
     */
    readonly serviceId: pulumi.Output<string>;
    /**
     * Create a Instance resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InstanceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Instance resources.
 */
export interface InstanceState {
    /**
     * A map contains the attributes of the instance. Check the [doc](https://docs.aws.amazon.com/cloud-map/latest/api/API_RegisterInstance.html#API_RegisterInstance_RequestSyntax) for the supported attributes and syntax.
     */
    attributes?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the service instance.
     */
    instanceId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the service that you want to use to create the instance.
     */
    serviceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Instance resource.
 */
export interface InstanceArgs {
    /**
     * A map contains the attributes of the instance. Check the [doc](https://docs.aws.amazon.com/cloud-map/latest/api/API_RegisterInstance.html#API_RegisterInstance_RequestSyntax) for the supported attributes and syntax.
     */
    attributes: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the service instance.
     */
    instanceId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the service that you want to use to create the instance.
     */
    serviceId: pulumi.Input<string>;
}
