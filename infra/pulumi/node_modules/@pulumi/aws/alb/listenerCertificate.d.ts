import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Load Balancer Listener Certificate resource.
 *
 * This resource is for additional certificates and does not replace the default certificate on the listener.
 *
 * > **Note:** `aws.alb.ListenerCertificate` is known as `aws.lb.ListenerCertificate`. The functionality is identical.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.acm.Certificate("example", {});
 * const frontEnd = new aws.lb.LoadBalancer("front_end", {});
 * const frontEndListener = new aws.lb.Listener("front_end", {});
 * const exampleListenerCertificate = new aws.lb.ListenerCertificate("example", {
 *     listenerArn: frontEndListener.arn,
 *     certificateArn: example.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Listener Certificates using the listener arn and certificate arn, separated by an underscore (`_`). For example:
 *
 * ```sh
 * $ pulumi import aws:alb/listenerCertificate:ListenerCertificate example arn:aws:elasticloadbalancing:us-west-2:123456789012:listener/app/test/8e4497da625e2d8a/9ab28ade35828f96/67b3d2d36dd7c26b_arn:aws:iam::123456789012:server-certificate/tf-acc-test-6453083910015726063
 * ```
 */
export declare class ListenerCertificate extends pulumi.CustomResource {
    /**
     * Get an existing ListenerCertificate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ListenerCertificateState, opts?: pulumi.CustomResourceOptions): ListenerCertificate;
    /**
     * Returns true if the given object is an instance of ListenerCertificate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ListenerCertificate;
    /**
     * The ARN of the certificate to attach to the listener.
     */
    readonly certificateArn: pulumi.Output<string>;
    /**
     * The ARN of the listener to which to attach the certificate.
     */
    readonly listenerArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a ListenerCertificate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ListenerCertificateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ListenerCertificate resources.
 */
export interface ListenerCertificateState {
    /**
     * The ARN of the certificate to attach to the listener.
     */
    certificateArn?: pulumi.Input<string>;
    /**
     * The ARN of the listener to which to attach the certificate.
     */
    listenerArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ListenerCertificate resource.
 */
export interface ListenerCertificateArgs {
    /**
     * The ARN of the certificate to attach to the listener.
     */
    certificateArn: pulumi.Input<string>;
    /**
     * The ARN of the listener to which to attach the certificate.
     */
    listenerArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
