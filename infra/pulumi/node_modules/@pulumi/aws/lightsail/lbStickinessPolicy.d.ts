import * as pulumi from "@pulumi/pulumi";
/**
 * Manages session stickiness for a Lightsail Load Balancer.
 *
 * Use this resource to configure session stickiness to ensure that user sessions are consistently routed to the same backend instance. This helps maintain session state for applications that store session data locally on the server.
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
 * const exampleLbStickinessPolicy = new aws.lightsail.LbStickinessPolicy("example", {
 *     lbName: example.name,
 *     cookieDuration: 900,
 *     enabled: true,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_lightsail_lb_stickiness_policy` using the `lb_name` attribute. For example:
 *
 * ```sh
 * $ pulumi import aws:lightsail/lbStickinessPolicy:LbStickinessPolicy example example-load-balancer
 * ```
 */
export declare class LbStickinessPolicy extends pulumi.CustomResource {
    /**
     * Get an existing LbStickinessPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LbStickinessPolicyState, opts?: pulumi.CustomResourceOptions): LbStickinessPolicy;
    /**
     * Returns true if the given object is an instance of LbStickinessPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LbStickinessPolicy;
    /**
     * Cookie duration in seconds. This determines the length of the session stickiness.
     */
    readonly cookieDuration: pulumi.Output<number>;
    /**
     * Whether to enable session stickiness for the load balancer.
     */
    readonly enabled: pulumi.Output<boolean>;
    /**
     * Name of the load balancer to which you want to enable session stickiness.
     *
     * The following arguments are optional:
     */
    readonly lbName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a LbStickinessPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LbStickinessPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LbStickinessPolicy resources.
 */
export interface LbStickinessPolicyState {
    /**
     * Cookie duration in seconds. This determines the length of the session stickiness.
     */
    cookieDuration?: pulumi.Input<number>;
    /**
     * Whether to enable session stickiness for the load balancer.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Name of the load balancer to which you want to enable session stickiness.
     *
     * The following arguments are optional:
     */
    lbName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LbStickinessPolicy resource.
 */
export interface LbStickinessPolicyArgs {
    /**
     * Cookie duration in seconds. This determines the length of the session stickiness.
     */
    cookieDuration: pulumi.Input<number>;
    /**
     * Whether to enable session stickiness for the load balancer.
     */
    enabled: pulumi.Input<boolean>;
    /**
     * Name of the load balancer to which you want to enable session stickiness.
     *
     * The following arguments are optional:
     */
    lbName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
