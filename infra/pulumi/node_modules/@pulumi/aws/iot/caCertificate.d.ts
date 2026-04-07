import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Creates and manages an AWS IoT CA Certificate.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as tls from "@pulumi/tls";
 *
 * const caPrivateKey = new tls.PrivateKey("ca", {algorithm: "RSA"});
 * const ca = new tls.SelfSignedCert("ca", {
 *     privateKeyPem: caPrivateKey.privateKeyPem,
 *     subject: [{
 *         commonName: "example.com",
 *         organization: "ACME Examples, Inc",
 *     }],
 *     validityPeriodHours: 12,
 *     allowedUses: [
 *         "key_encipherment",
 *         "digital_signature",
 *         "server_auth",
 *     ],
 *     isCaCertificate: true,
 * });
 * const verificationPrivateKey = new tls.PrivateKey("verification", {algorithm: "RSA"});
 * const example = aws.iot.getRegistrationCode({});
 * const verification = new tls.CertRequest("verification", {
 *     privateKeyPem: verificationPrivateKey.privateKeyPem,
 *     subject: [{
 *         commonName: example.then(example => example.registrationCode),
 *     }],
 * });
 * const verificationLocallySignedCert = new tls.LocallySignedCert("verification", {
 *     certRequestPem: verification.certRequestPem,
 *     caPrivateKeyPem: caPrivateKey.privateKeyPem,
 *     caCertPem: ca.certPem,
 *     validityPeriodHours: 12,
 *     allowedUses: [
 *         "key_encipherment",
 *         "digital_signature",
 *         "server_auth",
 *     ],
 * });
 * const exampleCaCertificate = new aws.iot.CaCertificate("example", {
 *     active: true,
 *     caCertificatePem: ca.certPem,
 *     verificationCertificatePem: verificationLocallySignedCert.certPem,
 *     allowAutoRegistration: true,
 * });
 * ```
 */
export declare class CaCertificate extends pulumi.CustomResource {
    /**
     * Get an existing CaCertificate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CaCertificateState, opts?: pulumi.CustomResourceOptions): CaCertificate;
    /**
     * Returns true if the given object is an instance of CaCertificate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CaCertificate;
    /**
     * Boolean flag to indicate if the certificate should be active for device authentication.
     */
    readonly active: pulumi.Output<boolean>;
    /**
     * Boolean flag to indicate if the certificate should be active for device regisration.
     */
    readonly allowAutoRegistration: pulumi.Output<boolean>;
    /**
     * The ARN of the created CA certificate.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * PEM encoded CA certificate.
     */
    readonly caCertificatePem: pulumi.Output<string>;
    /**
     * The certificate mode in which the CA will be registered. Valid values: `DEFAULT` and `SNI_ONLY`. Default: `DEFAULT`.
     */
    readonly certificateMode: pulumi.Output<string | undefined>;
    /**
     * The customer version of the CA certificate.
     */
    readonly customerVersion: pulumi.Output<number>;
    /**
     * The generation ID of the CA certificate.
     */
    readonly generationId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Information about the registration configuration. See below.
     */
    readonly registrationConfig: pulumi.Output<outputs.iot.CaCertificateRegistrationConfig | undefined>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * When the CA certificate is valid.
     */
    readonly validities: pulumi.Output<outputs.iot.CaCertificateValidity[]>;
    /**
     * PEM encoded verification certificate containing the common name of a registration code. Review
     * [CreateVerificationCSR](https://docs.aws.amazon.com/iot/latest/developerguide/register-CA-cert.html). Required if `certificateMode` is `DEFAULT`.
     */
    readonly verificationCertificatePem: pulumi.Output<string | undefined>;
    /**
     * Create a CaCertificate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CaCertificateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CaCertificate resources.
 */
export interface CaCertificateState {
    /**
     * Boolean flag to indicate if the certificate should be active for device authentication.
     */
    active?: pulumi.Input<boolean>;
    /**
     * Boolean flag to indicate if the certificate should be active for device regisration.
     */
    allowAutoRegistration?: pulumi.Input<boolean>;
    /**
     * The ARN of the created CA certificate.
     */
    arn?: pulumi.Input<string>;
    /**
     * PEM encoded CA certificate.
     */
    caCertificatePem?: pulumi.Input<string>;
    /**
     * The certificate mode in which the CA will be registered. Valid values: `DEFAULT` and `SNI_ONLY`. Default: `DEFAULT`.
     */
    certificateMode?: pulumi.Input<string>;
    /**
     * The customer version of the CA certificate.
     */
    customerVersion?: pulumi.Input<number>;
    /**
     * The generation ID of the CA certificate.
     */
    generationId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Information about the registration configuration. See below.
     */
    registrationConfig?: pulumi.Input<inputs.iot.CaCertificateRegistrationConfig>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * When the CA certificate is valid.
     */
    validities?: pulumi.Input<pulumi.Input<inputs.iot.CaCertificateValidity>[]>;
    /**
     * PEM encoded verification certificate containing the common name of a registration code. Review
     * [CreateVerificationCSR](https://docs.aws.amazon.com/iot/latest/developerguide/register-CA-cert.html). Required if `certificateMode` is `DEFAULT`.
     */
    verificationCertificatePem?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a CaCertificate resource.
 */
export interface CaCertificateArgs {
    /**
     * Boolean flag to indicate if the certificate should be active for device authentication.
     */
    active: pulumi.Input<boolean>;
    /**
     * Boolean flag to indicate if the certificate should be active for device regisration.
     */
    allowAutoRegistration: pulumi.Input<boolean>;
    /**
     * PEM encoded CA certificate.
     */
    caCertificatePem: pulumi.Input<string>;
    /**
     * The certificate mode in which the CA will be registered. Valid values: `DEFAULT` and `SNI_ONLY`. Default: `DEFAULT`.
     */
    certificateMode?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Information about the registration configuration. See below.
     */
    registrationConfig?: pulumi.Input<inputs.iot.CaCertificateRegistrationConfig>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * PEM encoded verification certificate containing the common name of a registration code. Review
     * [CreateVerificationCSR](https://docs.aws.amazon.com/iot/latest/developerguide/register-CA-cert.html). Required if `certificateMode` is `DEFAULT`.
     */
    verificationCertificatePem?: pulumi.Input<string>;
}
