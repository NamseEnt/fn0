import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a AWS Transfer AS2 Certificate resource.
 *
 * ## Example Usage
 *
 * ## Import
 *
 * Using `pulumi import`, import Transfer AS2 Certificate using the `certificate_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:transfer/certificate:Certificate example c-4221a88afd5f4362a
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
     * An date when the certificate becomes active
     */
    readonly activeDate: pulumi.Output<string>;
    /**
     * The ARN of the certificate
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The valid certificate file required for the transfer.
     */
    readonly certificate: pulumi.Output<string>;
    /**
     * The optional list of certificate that make up the chain for the certificate that is being imported.
     */
    readonly certificateChain: pulumi.Output<string | undefined>;
    /**
     * The unique identifier for the AS2 certificate
     */
    readonly certificateId: pulumi.Output<string>;
    /**
     * A short description that helps identify the certificate.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * An date when the certificate becomes inactive
     */
    readonly inactiveDate: pulumi.Output<string>;
    /**
     * The private key associated with the certificate being imported.
     */
    readonly privateKey: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Specifies if a certificate is being used for signing or encryption. The valid values are SIGNING and ENCRYPTION.
     */
    readonly usage: pulumi.Output<string>;
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
     * An date when the certificate becomes active
     */
    activeDate?: pulumi.Input<string>;
    /**
     * The ARN of the certificate
     */
    arn?: pulumi.Input<string>;
    /**
     * The valid certificate file required for the transfer.
     */
    certificate?: pulumi.Input<string>;
    /**
     * The optional list of certificate that make up the chain for the certificate that is being imported.
     */
    certificateChain?: pulumi.Input<string>;
    /**
     * The unique identifier for the AS2 certificate
     */
    certificateId?: pulumi.Input<string>;
    /**
     * A short description that helps identify the certificate.
     */
    description?: pulumi.Input<string>;
    /**
     * An date when the certificate becomes inactive
     */
    inactiveDate?: pulumi.Input<string>;
    /**
     * The private key associated with the certificate being imported.
     */
    privateKey?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Specifies if a certificate is being used for signing or encryption. The valid values are SIGNING and ENCRYPTION.
     */
    usage?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Certificate resource.
 */
export interface CertificateArgs {
    /**
     * The valid certificate file required for the transfer.
     */
    certificate: pulumi.Input<string>;
    /**
     * The optional list of certificate that make up the chain for the certificate that is being imported.
     */
    certificateChain?: pulumi.Input<string>;
    /**
     * A short description that helps identify the certificate.
     */
    description?: pulumi.Input<string>;
    /**
     * The private key associated with the certificate being imported.
     */
    privateKey?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Specifies if a certificate is being used for signing or encryption. The valid values are SIGNING and ENCRYPTION.
     */
    usage: pulumi.Input<string>;
}
