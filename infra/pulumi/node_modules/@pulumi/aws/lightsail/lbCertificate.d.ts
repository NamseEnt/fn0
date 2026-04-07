import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Lightsail Load Balancer Certificate.
 *
 * Use this resource to create and manage SSL/TLS certificates for Lightsail Load Balancers. The certificate must be validated before it can be attached to a load balancer to enable HTTPS traffic.
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
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_lightsail_lb_certificate` using the id attribute. For example:
 *
 * ```sh
 * $ pulumi import aws:lightsail/lbCertificate:LbCertificate example example-load-balancer,example-load-balancer-certificate
 * ```
 */
export declare class LbCertificate extends pulumi.CustomResource {
    /**
     * Get an existing LbCertificate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LbCertificateState, opts?: pulumi.CustomResourceOptions): LbCertificate;
    /**
     * Returns true if the given object is an instance of LbCertificate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LbCertificate;
    /**
     * ARN of the lightsail certificate.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Timestamp when the instance was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Domain name (e.g., example.com) for your SSL/TLS certificate.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * Set of domain validation objects which can be used to complete certificate validation. Can have more than one element, e.g., if SANs are defined.
     */
    readonly domainValidationRecords: pulumi.Output<outputs.lightsail.LbCertificateDomainValidationRecord[]>;
    /**
     * Load balancer name where you want to create the SSL/TLS certificate.
     */
    readonly lbName: pulumi.Output<string>;
    /**
     * SSL/TLS certificate name.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Set of domains that should be SANs in the issued certificate. `domainName` attribute is automatically added as a Subject Alternative Name.
     */
    readonly subjectAlternativeNames: pulumi.Output<string[]>;
    /**
     * Support code for the certificate.
     */
    readonly supportCode: pulumi.Output<string>;
    /**
     * Create a LbCertificate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LbCertificateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LbCertificate resources.
 */
export interface LbCertificateState {
    /**
     * ARN of the lightsail certificate.
     */
    arn?: pulumi.Input<string>;
    /**
     * Timestamp when the instance was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Domain name (e.g., example.com) for your SSL/TLS certificate.
     */
    domainName?: pulumi.Input<string>;
    /**
     * Set of domain validation objects which can be used to complete certificate validation. Can have more than one element, e.g., if SANs are defined.
     */
    domainValidationRecords?: pulumi.Input<pulumi.Input<inputs.lightsail.LbCertificateDomainValidationRecord>[]>;
    /**
     * Load balancer name where you want to create the SSL/TLS certificate.
     */
    lbName?: pulumi.Input<string>;
    /**
     * SSL/TLS certificate name.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Set of domains that should be SANs in the issued certificate. `domainName` attribute is automatically added as a Subject Alternative Name.
     */
    subjectAlternativeNames?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Support code for the certificate.
     */
    supportCode?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LbCertificate resource.
 */
export interface LbCertificateArgs {
    /**
     * Domain name (e.g., example.com) for your SSL/TLS certificate.
     */
    domainName?: pulumi.Input<string>;
    /**
     * Load balancer name where you want to create the SSL/TLS certificate.
     */
    lbName: pulumi.Input<string>;
    /**
     * SSL/TLS certificate name.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Set of domains that should be SANs in the issued certificate. `domainName` attribute is automatically added as a Subject Alternative Name.
     */
    subjectAlternativeNames?: pulumi.Input<pulumi.Input<string>[]>;
}
