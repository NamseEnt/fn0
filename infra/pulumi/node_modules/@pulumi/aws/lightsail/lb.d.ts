import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Lightsail load balancer resource.
 *
 * Use this resource to distribute incoming traffic across multiple Lightsail instances to improve application availability and performance.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.Lb("example", {
 *     name: "example-load-balancer",
 *     healthCheckPath: "/",
 *     instancePort: 80,
 *     tags: {
 *         foo: "bar",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_lightsail_lb` using the name attribute. For example:
 *
 * ```sh
 * $ pulumi import aws:lightsail/lb:Lb example example-load-balancer
 * ```
 */
export declare class Lb extends pulumi.CustomResource {
    /**
     * Get an existing Lb resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LbState, opts?: pulumi.CustomResourceOptions): Lb;
    /**
     * Returns true if the given object is an instance of Lb.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Lb;
    /**
     * ARN of the Lightsail load balancer.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Timestamp when the load balancer was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * DNS name of the load balancer.
     */
    readonly dnsName: pulumi.Output<string>;
    /**
     * Health check path of the load balancer. Default value `/`.
     */
    readonly healthCheckPath: pulumi.Output<string | undefined>;
    /**
     * Instance port the load balancer will connect to.
     */
    readonly instancePort: pulumi.Output<number>;
    /**
     * IP address type of the load balancer. Valid values: `dualstack`, `ipv4`. Default value `dualstack`.
     */
    readonly ipAddressType: pulumi.Output<string | undefined>;
    /**
     * Name of the Lightsail load balancer.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Protocol of the load balancer.
     */
    readonly protocol: pulumi.Output<string>;
    /**
     * Public ports of the load balancer.
     */
    readonly publicPorts: pulumi.Output<number[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Support code for the load balancer. Include this code in your email to support when you have questions about a load balancer in Lightsail. This code enables our support team to look up your Lightsail information more easily.
     */
    readonly supportCode: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Lb resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LbArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Lb resources.
 */
export interface LbState {
    /**
     * ARN of the Lightsail load balancer.
     */
    arn?: pulumi.Input<string>;
    /**
     * Timestamp when the load balancer was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * DNS name of the load balancer.
     */
    dnsName?: pulumi.Input<string>;
    /**
     * Health check path of the load balancer. Default value `/`.
     */
    healthCheckPath?: pulumi.Input<string>;
    /**
     * Instance port the load balancer will connect to.
     */
    instancePort?: pulumi.Input<number>;
    /**
     * IP address type of the load balancer. Valid values: `dualstack`, `ipv4`. Default value `dualstack`.
     */
    ipAddressType?: pulumi.Input<string>;
    /**
     * Name of the Lightsail load balancer.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Protocol of the load balancer.
     */
    protocol?: pulumi.Input<string>;
    /**
     * Public ports of the load balancer.
     */
    publicPorts?: pulumi.Input<pulumi.Input<number>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Support code for the load balancer. Include this code in your email to support when you have questions about a load balancer in Lightsail. This code enables our support team to look up your Lightsail information more easily.
     */
    supportCode?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Lb resource.
 */
export interface LbArgs {
    /**
     * Health check path of the load balancer. Default value `/`.
     */
    healthCheckPath?: pulumi.Input<string>;
    /**
     * Instance port the load balancer will connect to.
     */
    instancePort: pulumi.Input<number>;
    /**
     * IP address type of the load balancer. Valid values: `dualstack`, `ipv4`. Default value `dualstack`.
     */
    ipAddressType?: pulumi.Input<string>;
    /**
     * Name of the Lightsail load balancer.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
