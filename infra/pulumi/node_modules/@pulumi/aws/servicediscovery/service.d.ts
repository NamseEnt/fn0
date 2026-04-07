import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Service Discovery Service resource.
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
 *     name: "example.mydomain.local",
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
 *     healthCheckConfig: {
 *         failureThreshold: 1,
 *     },
 * });
 * ```
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicediscovery.PublicDnsNamespace("example", {
 *     name: "example.mydomain.com",
 *     description: "example",
 * });
 * const exampleService = new aws.servicediscovery.Service("example", {
 *     name: "example",
 *     dnsConfig: {
 *         namespaceId: example.id,
 *         dnsRecords: [{
 *             ttl: 10,
 *             type: "A",
 *         }],
 *     },
 *     healthCheckConfig: {
 *         failureThreshold: 10,
 *         resourcePath: "path",
 *         type: "HTTP",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Service Discovery Service using the service ID. For example:
 *
 * ```sh
 * $ pulumi import aws:servicediscovery/service:Service example 0123456789
 * ```
 */
export declare class Service extends pulumi.CustomResource {
    /**
     * Get an existing Service resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServiceState, opts?: pulumi.CustomResourceOptions): Service;
    /**
     * Returns true if the given object is an instance of Service.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Service;
    /**
     * The ARN of the service.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The description of the service.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * A complex type that contains information about the resource record sets that you want Amazon Route 53 to create when you register an instance. See `dnsConfig` Block for details.
     */
    readonly dnsConfig: pulumi.Output<outputs.servicediscovery.ServiceDnsConfig | undefined>;
    /**
     * A boolean that indicates all instances should be deleted from the service so that the service can be destroyed without error. These instances are not recoverable. Defaults to `false`.
     */
    readonly forceDestroy: pulumi.Output<boolean | undefined>;
    /**
     * A complex type that contains settings for an optional health check. Only for Public DNS namespaces. See `healthCheckConfig` Block for details.
     */
    readonly healthCheckConfig: pulumi.Output<outputs.servicediscovery.ServiceHealthCheckConfig | undefined>;
    /**
     * Please use `healthCheckConfig` instead. See `healthCheckCustomConfig` Block for details.
     */
    readonly healthCheckCustomConfig: pulumi.Output<outputs.servicediscovery.ServiceHealthCheckCustomConfig | undefined>;
    /**
     * The name of the service.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The ID of the namespace that you want to use to create the service.
     */
    readonly namespaceId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the service. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * If present, specifies that the service instances are only discoverable using the `DiscoverInstances` API operation. No DNS records is registered for the service instances. The only valid value is `HTTP`.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a Service resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ServiceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Service resources.
 */
export interface ServiceState {
    /**
     * The ARN of the service.
     */
    arn?: pulumi.Input<string>;
    /**
     * The description of the service.
     */
    description?: pulumi.Input<string>;
    /**
     * A complex type that contains information about the resource record sets that you want Amazon Route 53 to create when you register an instance. See `dnsConfig` Block for details.
     */
    dnsConfig?: pulumi.Input<inputs.servicediscovery.ServiceDnsConfig>;
    /**
     * A boolean that indicates all instances should be deleted from the service so that the service can be destroyed without error. These instances are not recoverable. Defaults to `false`.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * A complex type that contains settings for an optional health check. Only for Public DNS namespaces. See `healthCheckConfig` Block for details.
     */
    healthCheckConfig?: pulumi.Input<inputs.servicediscovery.ServiceHealthCheckConfig>;
    /**
     * Please use `healthCheckConfig` instead. See `healthCheckCustomConfig` Block for details.
     */
    healthCheckCustomConfig?: pulumi.Input<inputs.servicediscovery.ServiceHealthCheckCustomConfig>;
    /**
     * The name of the service.
     */
    name?: pulumi.Input<string>;
    /**
     * The ID of the namespace that you want to use to create the service.
     */
    namespaceId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the service. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * If present, specifies that the service instances are only discoverable using the `DiscoverInstances` API operation. No DNS records is registered for the service instances. The only valid value is `HTTP`.
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Service resource.
 */
export interface ServiceArgs {
    /**
     * The description of the service.
     */
    description?: pulumi.Input<string>;
    /**
     * A complex type that contains information about the resource record sets that you want Amazon Route 53 to create when you register an instance. See `dnsConfig` Block for details.
     */
    dnsConfig?: pulumi.Input<inputs.servicediscovery.ServiceDnsConfig>;
    /**
     * A boolean that indicates all instances should be deleted from the service so that the service can be destroyed without error. These instances are not recoverable. Defaults to `false`.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * A complex type that contains settings for an optional health check. Only for Public DNS namespaces. See `healthCheckConfig` Block for details.
     */
    healthCheckConfig?: pulumi.Input<inputs.servicediscovery.ServiceHealthCheckConfig>;
    /**
     * Please use `healthCheckConfig` instead. See `healthCheckCustomConfig` Block for details.
     */
    healthCheckCustomConfig?: pulumi.Input<inputs.servicediscovery.ServiceHealthCheckCustomConfig>;
    /**
     * The name of the service.
     */
    name?: pulumi.Input<string>;
    /**
     * The ID of the namespace that you want to use to create the service.
     */
    namespaceId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the service. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * If present, specifies that the service instances are only discoverable using the `DiscoverInstances` API operation. No DNS records is registered for the service instances. The only valid value is `HTTP`.
     */
    type?: pulumi.Input<string>;
}
