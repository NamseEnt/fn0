import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * The ACM certificate resource allows requesting and management of certificates
 * from the Amazon Certificate Manager.
 *
 * ACM certificates can be created in three ways:
 * Amazon-issued, where AWS provides the certificate authority and automatically manages renewal;
 * imported certificates, issued by another certificate authority;
 * and private certificates, issued using an ACM Private Certificate Authority.
 *
 * ## Amazon-Issued Certificates
 *
 * For Amazon-issued certificates, this resource deals with requesting certificates and managing their attributes and life-cycle.
 * This resource does not deal with validation of a certificate but can provide inputs
 * for other resources implementing the validation.
 * It does not wait for a certificate to be issued.
 * Use a `aws.acm.CertificateValidation` resource for this.
 *
 * Most commonly, this resource is used together with `aws.route53.Record` and
 * `aws.acm.CertificateValidation` to request a DNS validated certificate,
 * deploy the required validation records and wait for validation to complete.
 *
 * Domain validation through email is also supported but should be avoided as it requires a manual step outside of this provider.
 *
 * ## Certificates Imported from Other Certificate Authority
 *
 * Imported certificates can be used to make certificates created with an external certificate authority available for AWS services.
 *
 * As they are not managed by AWS, imported certificates are not eligible for automatic renewal.
 * New certificate materials can be supplied to an existing imported certificate to update it in place.
 *
 * ## Private Certificates
 *
 * Private certificates are issued by an ACM Private Certificate Authority, which can be created using the resource type `aws.acmpca.CertificateAuthority`.
 *
 * Private certificates created using this resource are eligible for managed renewal if they have been exported or associated with another AWS service.
 * See [managed renewal documentation](https://docs.aws.amazon.com/acm/latest/userguide/managed-renewal.html) for more information.
 * By default, a certificate is valid for 395 days and the managed renewal process will start 60 days before expiration.
 * To renew the certificate earlier than 60 days before expiration, configure `earlyRenewalDuration`.
 *
 * ## Example Usage
 *
 * ### Custom Domain Validation Options
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const cert = new aws.acm.Certificate("cert", {
 *     domainName: "testing.example.com",
 *     validationMethod: "EMAIL",
 *     validationOptions: [{
 *         domainName: "testing.example.com",
 *         validationDomain: "example.com",
 *     }],
 * });
 * ```
 *
 * ### Existing Certificate Body Import
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as tls from "@pulumi/tls";
 *
 * const example = new tls.PrivateKey("example", {algorithm: "RSA"});
 * const exampleSelfSignedCert = new tls.SelfSignedCert("example", {
 *     keyAlgorithm: "RSA",
 *     privateKeyPem: example.privateKeyPem,
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
 * });
 * const cert = new aws.acm.Certificate("cert", {
 *     privateKey: example.privateKeyPem,
 *     certificateBody: exampleSelfSignedCert.certPem,
 * });
 * ```
 *
 * ### Referencing domainValidationOptions With forEach Based Resources
 *
 * See the `aws.acm.CertificateValidation` resource for a full example of performing DNS validation.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example: aws.route53.Record[] = [];
 * for (const range of Object.entries(.reduce((__obj, dvo) => ({ ...__obj, [dvo.domainName]: {
 *     name: dvo.resourceRecordName,
 *     record: dvo.resourceRecordValue,
 *     type: dvo.resourceRecordType,
 * } }))).map(([k, v]) => ({key: k, value: v}))) {
 *     example.push(new aws.route53.Record(`example-${range.key}`, {
 *         allowOverwrite: true,
 *         name: range.value.name,
 *         records: [range.value.record],
 *         ttl: 60,
 *         type: aws.route53.RecordType[range.value.type],
 *         zoneId: exampleAwsRoute53Zone.zoneId,
 *     }));
 * }
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) ARN of the certificate.
 *
 * Using `pulumi import`, import certificates using their ARN. For example:
 *
 * console
 *
 * % pulumi import aws_acm_certificate.example arn:aws:acm:eu-central-1:123456789012:certificate/7e7a28d2-163f-4b8f-b9cd-822f96c08d6a
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
     * ARN of the certificate
     */
    readonly arn: pulumi.Output<string>;
    readonly certificateAuthorityArn: pulumi.Output<string | undefined>;
    readonly certificateBody: pulumi.Output<string | undefined>;
    readonly certificateChain: pulumi.Output<string | undefined>;
    /**
     * Fully qualified domain name (FQDN) in the certificate.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * Set of domain validation objects which can be used to complete certificate validation.
     * Can have more than one element, e.g., if SANs are defined.
     * Only set if `DNS`-validation was used.
     */
    readonly domainValidationOptions: pulumi.Output<outputs.acm.CertificateDomainValidationOption[]>;
    readonly earlyRenewalDuration: pulumi.Output<string | undefined>;
    readonly keyAlgorithm: pulumi.Output<string>;
    /**
     * Expiration date and time of the certificate.
     */
    readonly notAfter: pulumi.Output<string>;
    /**
     * Start of the validity period of the certificate.
     */
    readonly notBefore: pulumi.Output<string>;
    readonly options: pulumi.Output<outputs.acm.CertificateOptions>;
    /**
     * `true` if a Private certificate eligible for managed renewal is within the `earlyRenewalDuration` period.
     */
    readonly pendingRenewal: pulumi.Output<boolean>;
    readonly privateKey: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     * * Creating an Amazon issued certificate
     */
    readonly region: pulumi.Output<string>;
    /**
     * Whether the certificate is eligible for managed renewal.
     */
    readonly renewalEligibility: pulumi.Output<string>;
    /**
     * Contains information about the status of ACM's [managed renewal](https://docs.aws.amazon.com/acm/latest/userguide/acm-renewal.html) for the certificate.
     */
    readonly renewalSummaries: pulumi.Output<outputs.acm.CertificateRenewalSummary[]>;
    /**
     * Status of the certificate.
     */
    readonly status: pulumi.Output<string>;
    readonly subjectAlternativeNames: pulumi.Output<string[]>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Source of the certificate.
     */
    readonly type: pulumi.Output<string>;
    /**
     * List of addresses that received a validation email. Only set if `EMAIL` validation was used.
     */
    readonly validationEmails: pulumi.Output<string[]>;
    readonly validationMethod: pulumi.Output<string>;
    readonly validationOptions: pulumi.Output<outputs.acm.CertificateValidationOption[] | undefined>;
    /**
     * Create a Certificate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: CertificateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Certificate resources.
 */
export interface CertificateState {
    /**
     * ARN of the certificate
     */
    arn?: pulumi.Input<string>;
    certificateAuthorityArn?: pulumi.Input<string>;
    certificateBody?: pulumi.Input<string>;
    certificateChain?: pulumi.Input<string>;
    /**
     * Fully qualified domain name (FQDN) in the certificate.
     */
    domainName?: pulumi.Input<string>;
    /**
     * Set of domain validation objects which can be used to complete certificate validation.
     * Can have more than one element, e.g., if SANs are defined.
     * Only set if `DNS`-validation was used.
     */
    domainValidationOptions?: pulumi.Input<pulumi.Input<inputs.acm.CertificateDomainValidationOption>[]>;
    earlyRenewalDuration?: pulumi.Input<string>;
    keyAlgorithm?: pulumi.Input<string>;
    /**
     * Expiration date and time of the certificate.
     */
    notAfter?: pulumi.Input<string>;
    /**
     * Start of the validity period of the certificate.
     */
    notBefore?: pulumi.Input<string>;
    options?: pulumi.Input<inputs.acm.CertificateOptions>;
    /**
     * `true` if a Private certificate eligible for managed renewal is within the `earlyRenewalDuration` period.
     */
    pendingRenewal?: pulumi.Input<boolean>;
    privateKey?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     * * Creating an Amazon issued certificate
     */
    region?: pulumi.Input<string>;
    /**
     * Whether the certificate is eligible for managed renewal.
     */
    renewalEligibility?: pulumi.Input<string>;
    /**
     * Contains information about the status of ACM's [managed renewal](https://docs.aws.amazon.com/acm/latest/userguide/acm-renewal.html) for the certificate.
     */
    renewalSummaries?: pulumi.Input<pulumi.Input<inputs.acm.CertificateRenewalSummary>[]>;
    /**
     * Status of the certificate.
     */
    status?: pulumi.Input<string>;
    subjectAlternativeNames?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * Source of the certificate.
     */
    type?: pulumi.Input<string>;
    /**
     * List of addresses that received a validation email. Only set if `EMAIL` validation was used.
     */
    validationEmails?: pulumi.Input<pulumi.Input<string>[]>;
    validationMethod?: pulumi.Input<string>;
    validationOptions?: pulumi.Input<pulumi.Input<inputs.acm.CertificateValidationOption>[]>;
}
/**
 * The set of arguments for constructing a Certificate resource.
 */
export interface CertificateArgs {
    certificateAuthorityArn?: pulumi.Input<string>;
    certificateBody?: pulumi.Input<string>;
    certificateChain?: pulumi.Input<string>;
    /**
     * Fully qualified domain name (FQDN) in the certificate.
     */
    domainName?: pulumi.Input<string>;
    earlyRenewalDuration?: pulumi.Input<string>;
    keyAlgorithm?: pulumi.Input<string>;
    options?: pulumi.Input<inputs.acm.CertificateOptions>;
    privateKey?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     * * Creating an Amazon issued certificate
     */
    region?: pulumi.Input<string>;
    subjectAlternativeNames?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    validationMethod?: pulumi.Input<string>;
    validationOptions?: pulumi.Input<pulumi.Input<inputs.acm.CertificateValidationOption>[]>;
}
