import * as pulumi from "@pulumi/pulumi";
/**
 * Associates a certificate with an AWS Certificate Manager Private Certificate Authority (ACM PCA Certificate Authority). An ACM PCA Certificate Authority is unable to issue certificates until it has a certificate associated with it. A root level ACM PCA Certificate Authority is able to self-sign its own root certificate.
 *
 * ## Example Usage
 *
 * ### Self-Signed Root Certificate Authority Certificate
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleCertificateAuthority = new aws.acmpca.CertificateAuthority("example", {
 *     type: "ROOT",
 *     certificateAuthorityConfiguration: {
 *         keyAlgorithm: "RSA_4096",
 *         signingAlgorithm: "SHA512WITHRSA",
 *         subject: {
 *             commonName: "example.com",
 *         },
 *     },
 * });
 * const current = aws.getPartition({});
 * const exampleCertificate = new aws.acmpca.Certificate("example", {
 *     certificateAuthorityArn: exampleCertificateAuthority.arn,
 *     certificateSigningRequest: exampleCertificateAuthority.certificateSigningRequest,
 *     signingAlgorithm: "SHA512WITHRSA",
 *     templateArn: current.then(current => `arn:${current.partition}:acm-pca:::template/RootCACertificate/V1`),
 *     validity: {
 *         type: "YEARS",
 *         value: "1",
 *     },
 * });
 * const example = new aws.acmpca.CertificateAuthorityCertificate("example", {
 *     certificateAuthorityArn: exampleCertificateAuthority.arn,
 *     certificate: exampleCertificate.certificate,
 *     certificateChain: exampleCertificate.certificateChain,
 * });
 * ```
 *
 * ### Certificate for Subordinate Certificate Authority
 *
 * Note that the certificate for the subordinate certificate authority must be issued by the root certificate authority using a signing request from the subordinate certificate authority.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const subordinateCertificateAuthority = new aws.acmpca.CertificateAuthority("subordinate", {
 *     type: "SUBORDINATE",
 *     certificateAuthorityConfiguration: {
 *         keyAlgorithm: "RSA_2048",
 *         signingAlgorithm: "SHA512WITHRSA",
 *         subject: {
 *             commonName: "sub.example.com",
 *         },
 *     },
 * });
 * const root = new aws.acmpca.CertificateAuthority("root", {});
 * const current = aws.getPartition({});
 * const subordinateCertificate = new aws.acmpca.Certificate("subordinate", {
 *     certificateAuthorityArn: root.arn,
 *     certificateSigningRequest: subordinateCertificateAuthority.certificateSigningRequest,
 *     signingAlgorithm: "SHA512WITHRSA",
 *     templateArn: current.then(current => `arn:${current.partition}:acm-pca:::template/SubordinateCACertificate_PathLen0/V1`),
 *     validity: {
 *         type: "YEARS",
 *         value: "1",
 *     },
 * });
 * const subordinate = new aws.acmpca.CertificateAuthorityCertificate("subordinate", {
 *     certificateAuthorityArn: subordinateCertificateAuthority.arn,
 *     certificate: subordinateCertificate.certificate,
 *     certificateChain: subordinateCertificate.certificateChain,
 * });
 * const rootCertificateAuthorityCertificate = new aws.acmpca.CertificateAuthorityCertificate("root", {});
 * const rootCertificate = new aws.acmpca.Certificate("root", {});
 * ```
 */
export declare class CertificateAuthorityCertificate extends pulumi.CustomResource {
    /**
     * Get an existing CertificateAuthorityCertificate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CertificateAuthorityCertificateState, opts?: pulumi.CustomResourceOptions): CertificateAuthorityCertificate;
    /**
     * Returns true if the given object is an instance of CertificateAuthorityCertificate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CertificateAuthorityCertificate;
    /**
     * PEM-encoded certificate for the Certificate Authority.
     */
    readonly certificate: pulumi.Output<string>;
    /**
     * ARN of the Certificate Authority.
     */
    readonly certificateAuthorityArn: pulumi.Output<string>;
    /**
     * PEM-encoded certificate chain that includes any intermediate certificates and chains up to root CA. Required for subordinate Certificate Authorities. Not allowed for root Certificate Authorities.
     */
    readonly certificateChain: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a CertificateAuthorityCertificate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CertificateAuthorityCertificateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CertificateAuthorityCertificate resources.
 */
export interface CertificateAuthorityCertificateState {
    /**
     * PEM-encoded certificate for the Certificate Authority.
     */
    certificate?: pulumi.Input<string>;
    /**
     * ARN of the Certificate Authority.
     */
    certificateAuthorityArn?: pulumi.Input<string>;
    /**
     * PEM-encoded certificate chain that includes any intermediate certificates and chains up to root CA. Required for subordinate Certificate Authorities. Not allowed for root Certificate Authorities.
     */
    certificateChain?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a CertificateAuthorityCertificate resource.
 */
export interface CertificateAuthorityCertificateArgs {
    /**
     * PEM-encoded certificate for the Certificate Authority.
     */
    certificate: pulumi.Input<string>;
    /**
     * ARN of the Certificate Authority.
     */
    certificateAuthorityArn: pulumi.Input<string>;
    /**
     * PEM-encoded certificate chain that includes any intermediate certificates and chains up to root CA. Required for subordinate Certificate Authorities. Not allowed for root Certificate Authorities.
     */
    certificateChain?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
