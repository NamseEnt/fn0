import * as pulumi from "@pulumi/pulumi";
/**
 * Manages HTTPS redirection for a Lightsail Load Balancer.
 *
 * Use this resource to configure automatic redirection of HTTP traffic to HTTPS on a Lightsail Load Balancer. A valid certificate must be attached to the load balancer before enabling HTTPS redirection.
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
 * const exampleLbCertificate = new aws.lightsail.LbCertificate("example", {
 *     name: "example-load-balancer-certificate",
 *     lbName: example.id,
 *     domainName: "example.com",
 * });
 * const exampleLbCertificateAttachment = new aws.lightsail.LbCertificateAttachment("example", {
 *     lbName: example.name,
 *     certificateName: exampleLbCertificate.name,
 * });
 * const exampleLbHttpsRedirectionPolicy = new aws.lightsail.LbHttpsRedirectionPolicy("example", {
 *     lbName: example.name,
 *     enabled: true,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_lightsail_lb_https_redirection_policy` using the `lb_name` attribute. For example:
 *
 * ```sh
 * $ pulumi import aws:lightsail/lbHttpsRedirectionPolicy:LbHttpsRedirectionPolicy example example-load-balancer
 * ```
 */
export declare class LbHttpsRedirectionPolicy extends pulumi.CustomResource {
    /**
     * Get an existing LbHttpsRedirectionPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LbHttpsRedirectionPolicyState, opts?: pulumi.CustomResourceOptions): LbHttpsRedirectionPolicy;
    /**
     * Returns true if the given object is an instance of LbHttpsRedirectionPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LbHttpsRedirectionPolicy;
    /**
     * Whether to enable HTTP to HTTPS redirection. `true` to activate HTTP to HTTPS redirection or `false` to deactivate HTTP to HTTPS redirection.
     */
    readonly enabled: pulumi.Output<boolean>;
    /**
     * Name of the load balancer to which you want to enable HTTP to HTTPS redirection.
     *
     * The following arguments are optional:
     */
    readonly lbName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a LbHttpsRedirectionPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LbHttpsRedirectionPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LbHttpsRedirectionPolicy resources.
 */
export interface LbHttpsRedirectionPolicyState {
    /**
     * Whether to enable HTTP to HTTPS redirection. `true` to activate HTTP to HTTPS redirection or `false` to deactivate HTTP to HTTPS redirection.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Name of the load balancer to which you want to enable HTTP to HTTPS redirection.
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
 * The set of arguments for constructing a LbHttpsRedirectionPolicy resource.
 */
export interface LbHttpsRedirectionPolicyArgs {
    /**
     * Whether to enable HTTP to HTTPS redirection. `true` to activate HTTP to HTTPS redirection or `false` to deactivate HTTP to HTTPS redirection.
     */
    enabled: pulumi.Input<boolean>;
    /**
     * Name of the load balancer to which you want to enable HTTP to HTTPS redirection.
     *
     * The following arguments are optional:
     */
    lbName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
