import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to issue a certificate using AWS Certificate Manager Private Certificate Authority (ACM PCA).
 *
 * Certificates created using `aws.acmpca.Certificate` are not eligible for automatic renewal,
 * and must be replaced instead.
 * To issue a renewable certificate using an ACM PCA, create a `aws.acm.Certificate`
 * with the parameter `certificateAuthorityArn`.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as tls from "@pulumi/tls";
 *
 * const exampleCertificateAuthority = new aws.acmpca.CertificateAuthority("example", {
 *     certificateAuthorityConfiguration: {
 *         keyAlgorithm: "RSA_4096",
 *         signingAlgorithm: "SHA512WITHRSA",
 *         subject: {
 *             commonName: "example.com",
 *         },
 *     },
 *     permanentDeletionTimeInDays: 7,
 * });
 * const key = new tls.PrivateKey("key", {algorithm: "RSA"});
 * const csr = new tls.CertRequest("csr", {
 *     privateKeyPem: key.privateKeyPem,
 *     subject: [{
 *         commonName: "example",
 *     }],
 * });
 * const example = new aws.acmpca.Certificate("example", {
 *     certificateAuthorityArn: exampleCertificateAuthority.arn,
 *     certificateSigningRequest: csr.certRequestPem,
 *     signingAlgorithm: "SHA256WITHRSA",
 *     validity: {
 *         type: "YEARS",
 *         value: "1",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the ACM PCA certificate.
 *
 * Using `pulumi import`, import ACM PCA Certificates using their ARN. For example:
 *
 * console
 *
 * % pulumi import aws_acmpca_certificate.cert arn:aws:acm-pca:eu-west-1:675225743824:certificate-authority/08319ede-83g9-1400-8f21-c7d12b2b6edb/certificate/a4e9c2aa4bcfab625g1b9136464cd3a
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
     * Specifies X.509 certificate information to be included in the issued certificate. To use with API Passthrough templates
     */
    readonly apiPassthrough: pulumi.Output<string | undefined>;
    /**
     * ARN of the certificate.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * PEM-encoded certificate value.
     */
    readonly certificate: pulumi.Output<string>;
    /**
     * ARN of the certificate authority.
     */
    readonly certificateAuthorityArn: pulumi.Output<string>;
    /**
     * PEM-encoded certificate chain that includes any intermediate certificates and chains up to root CA.
     */
    readonly certificateChain: pulumi.Output<string>;
    /**
     * Certificate Signing Request in PEM format.
     */
    readonly certificateSigningRequest: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Algorithm to use to sign certificate requests. Valid values: `SHA256WITHRSA`, `SHA256WITHECDSA`, `SHA384WITHRSA`, `SHA384WITHECDSA`, `SHA512WITHRSA`, `SHA512WITHECDSA`.
     */
    readonly signingAlgorithm: pulumi.Output<string>;
    /**
     * Template to use when issuing a certificate.
     * See [ACM PCA Documentation](https://docs.aws.amazon.com/privateca/latest/userguide/UsingTemplates.html) for more information.
     */
    readonly templateArn: pulumi.Output<string | undefined>;
    /**
     * Configures end of the validity period for the certificate. See validity block below.
     */
    readonly validity: pulumi.Output<outputs.acmpca.CertificateValidity>;
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
     * Specifies X.509 certificate information to be included in the issued certificate. To use with API Passthrough templates
     */
    apiPassthrough?: pulumi.Input<string>;
    /**
     * ARN of the certificate.
     */
    arn?: pulumi.Input<string>;
    /**
     * PEM-encoded certificate value.
     */
    certificate?: pulumi.Input<string>;
    /**
     * ARN of the certificate authority.
     */
    certificateAuthorityArn?: pulumi.Input<string>;
    /**
     * PEM-encoded certificate chain that includes any intermediate certificates and chains up to root CA.
     */
    certificateChain?: pulumi.Input<string>;
    /**
     * Certificate Signing Request in PEM format.
     */
    certificateSigningRequest?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Algorithm to use to sign certificate requests. Valid values: `SHA256WITHRSA`, `SHA256WITHECDSA`, `SHA384WITHRSA`, `SHA384WITHECDSA`, `SHA512WITHRSA`, `SHA512WITHECDSA`.
     */
    signingAlgorithm?: pulumi.Input<string>;
    /**
     * Template to use when issuing a certificate.
     * See [ACM PCA Documentation](https://docs.aws.amazon.com/privateca/latest/userguide/UsingTemplates.html) for more information.
     */
    templateArn?: pulumi.Input<string>;
    /**
     * Configures end of the validity period for the certificate. See validity block below.
     */
    validity?: pulumi.Input<inputs.acmpca.CertificateValidity>;
}
/**
 * The set of arguments for constructing a Certificate resource.
 */
export interface CertificateArgs {
    /**
     * Specifies X.509 certificate information to be included in the issued certificate. To use with API Passthrough templates
     */
    apiPassthrough?: pulumi.Input<string>;
    /**
     * ARN of the certificate authority.
     */
    certificateAuthorityArn: pulumi.Input<string>;
    /**
     * Certificate Signing Request in PEM format.
     */
    certificateSigningRequest: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Algorithm to use to sign certificate requests. Valid values: `SHA256WITHRSA`, `SHA256WITHECDSA`, `SHA384WITHRSA`, `SHA384WITHECDSA`, `SHA512WITHRSA`, `SHA512WITHECDSA`.
     */
    signingAlgorithm: pulumi.Input<string>;
    /**
     * Template to use when issuing a certificate.
     * See [ACM PCA Documentation](https://docs.aws.amazon.com/privateca/latest/userguide/UsingTemplates.html) for more information.
     */
    templateArn?: pulumi.Input<string>;
    /**
     * Configures end of the validity period for the certificate. See validity block below.
     */
    validity: pulumi.Input<inputs.acmpca.CertificateValidity>;
}
