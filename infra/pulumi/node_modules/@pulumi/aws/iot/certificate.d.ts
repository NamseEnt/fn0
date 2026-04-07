import * as pulumi from "@pulumi/pulumi";
/**
 * Creates and manages an AWS IoT certificate.
 *
 * ## Example Usage
 *
 * ### With CSR
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const cert = new aws.iot.Certificate("cert", {
 *     csr: std.file({
 *         input: "/my/csr.pem",
 *     }).then(invoke => invoke.result),
 *     active: true,
 * });
 * ```
 *
 * ### Without CSR
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const cert = new aws.iot.Certificate("cert", {active: true});
 * ```
 *
 * ### From existing certificate without a CA
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const cert = new aws.iot.Certificate("cert", {
 *     certificatePem: std.file({
 *         input: "/my/cert.pem",
 *     }).then(invoke => invoke.result),
 *     active: true,
 * });
 * ```
 */
export declare class Certificate extends pulumi.CustomResource {
    /**
     * Get an existing Certificate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CertificateState, opts?: pulumi.CustomResourceOptions): Certificate;
    /**
     * Returns true if the given object is an instance of Certificate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Certificate;
    /**
     * Boolean flag to indicate if the certificate should be active
     */
    readonly active: pulumi.Output<boolean>;
    /**
     * The ARN of the created certificate.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The certificate ID of the CA certificate used to sign the certificate.
     */
    readonly caCertificateId: pulumi.Output<string>;
    /**
     * The CA certificate for the certificate to be registered. If this is set, the CA needs to be registered with AWS IoT beforehand.
     */
    readonly caPem: pulumi.Output<string | undefined>;
    /**
     * The certificate to be registered. If `caPem` is unspecified, review
     * [RegisterCertificateWithoutCA](https://docs.aws.amazon.com/iot/latest/apireference/API_RegisterCertificateWithoutCA.html).
     * If `caPem` is specified, review
     * [RegisterCertificate](https://docs.aws.amazon.com/iot/latest/apireference/API_RegisterCertificate.html)
     * for more information on registering a certificate.
     */
    readonly certificatePem: pulumi.Output<string>;
    /**
     * The certificate signing request. Review
     * [CreateCertificateFromCsr](https://docs.aws.amazon.com/iot/latest/apireference/API_CreateCertificateFromCsr.html)
     * for more information on generating a certificate from a certificate signing request (CSR).
     * If none is specified both the certificate and keys will be generated, review [CreateKeysAndCertificate](https://docs.aws.amazon.com/iot/latest/apireference/API_CreateKeysAndCertificate.html)
     * for more information on generating keys and a certificate.
     */
    readonly csr: pulumi.Output<string | undefined>;
    /**
     * When neither CSR nor certificate is provided, the private key.
     */
    readonly privateKey: pulumi.Output<string>;
    /**
     * When neither CSR nor certificate is provided, the public key.
     */
    readonly publicKey: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a Certificate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CertificateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Certificate resources.
 */
export interface CertificateState {
    /**
     * Boolean flag to indicate if the certificate should be active
     */
    active?: pulumi.Input<boolean>;
    /**
     * The ARN of the created certificate.
     */
    arn?: pulumi.Input<string>;
    /**
     * The certificate ID of the CA certificate used to sign the certificate.
     */
    caCertificateId?: pulumi.Input<string>;
    /**
     * The CA certificate for the certificate to be registered. If this is set, the CA needs to be registered with AWS IoT beforehand.
     */
    caPem?: pulumi.Input<string>;
    /**
     * The certificate to be registered. If `caPem` is unspecified, review
     * [RegisterCertificateWithoutCA](https://docs.aws.amazon.com/iot/latest/apireference/API_RegisterCertificateWithoutCA.html).
     * If `caPem` is specified, review
     * [RegisterCertificate](https://docs.aws.amazon.com/iot/latest/apireference/API_RegisterCertificate.html)
     * for more information on registering a certificate.
     */
    certificatePem?: pulumi.Input<string>;
    /**
     * The certificate signing request. Review
     * [CreateCertificateFromCsr](https://docs.aws.amazon.com/iot/latest/apireference/API_CreateCertificateFromCsr.html)
     * for more information on generating a certificate from a certificate signing request (CSR).
     * If none is specified both the certificate and keys will be generated, review [CreateKeysAndCertificate](https://docs.aws.amazon.com/iot/latest/apireference/API_CreateKeysAndCertificate.html)
     * for more information on generating keys and a certificate.
     */
    csr?: pulumi.Input<string>;
    /**
     * When neither CSR nor certificate is provided, the private key.
     */
    privateKey?: pulumi.Input<string>;
    /**
     * When neither CSR nor certificate is provided, the public key.
     */
    publicKey?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Certificate resource.
 */
export interface CertificateArgs {
    /**
     * Boolean flag to indicate if the certificate should be active
     */
    active: pulumi.Input<boolean>;
    /**
     * The CA certificate for the certificate to be registered. If this is set, the CA needs to be registered with AWS IoT beforehand.
     */
    caPem?: pulumi.Input<string>;
    /**
     * The certificate to be registered. If `caPem` is unspecified, review
     * [RegisterCertificateWithoutCA](https://docs.aws.amazon.com/iot/latest/apireference/API_RegisterCertificateWithoutCA.html).
     * If `caPem` is specified, review
     * [RegisterCertificate](https://docs.aws.amazon.com/iot/latest/apireference/API_RegisterCertificate.html)
     * for more information on registering a certificate.
     */
    certificatePem?: pulumi.Input<string>;
    /**
     * The certificate signing request. Review
     * [CreateCertificateFromCsr](https://docs.aws.amazon.com/iot/latest/apireference/API_CreateCertificateFromCsr.html)
     * for more information on generating a certificate from a certificate signing request (CSR).
     * If none is specified both the certificate and keys will be generated, review [CreateKeysAndCertificate](https://docs.aws.amazon.com/iot/latest/apireference/API_CreateKeysAndCertificate.html)
     * for more information on generating keys and a certificate.
     */
    csr?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
