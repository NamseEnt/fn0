import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a DMS (Data Migration Service) certificate resource. DMS certificates can be created, deleted, and imported.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * // Create a new certificate
 * const test = new aws.dms.Certificate("test", {
 *     certificateId: "test-dms-certificate-tf",
 *     certificatePem: "...",
 *     tags: {
 *         Name: "test",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import certificates using the `certificate_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:dms/certificate:Certificate test test-dms-certificate-tf
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
     * The Amazon Resource Name (ARN) for the certificate.
     */
    readonly certificateArn: pulumi.Output<string>;
    /**
     * The certificate identifier.
     */
    readonly certificateId: pulumi.Output<string>;
    /**
     * The contents of the .pem X.509 certificate file for the certificate. Either `certificatePem` or `certificateWallet` must be set.
     */
    readonly certificatePem: pulumi.Output<string | undefined>;
    /**
     * The contents of the Oracle Wallet certificate for use with SSL, provided as a base64-encoded String. Either `certificatePem` or `certificateWallet` must be set.
     */
    readonly certificateWallet: pulumi.Output<string | undefined>;
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
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
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
     * The Amazon Resource Name (ARN) for the certificate.
     */
    certificateArn?: pulumi.Input<string>;
    /**
     * The certificate identifier.
     */
    certificateId?: pulumi.Input<string>;
    /**
     * The contents of the .pem X.509 certificate file for the certificate. Either `certificatePem` or `certificateWallet` must be set.
     */
    certificatePem?: pulumi.Input<string>;
    /**
     * The contents of the Oracle Wallet certificate for use with SSL, provided as a base64-encoded String. Either `certificatePem` or `certificateWallet` must be set.
     */
    certificateWallet?: pulumi.Input<string>;
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
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Certificate resource.
 */
export interface CertificateArgs {
    /**
     * The certificate identifier.
     */
    certificateId: pulumi.Input<string>;
    /**
     * The contents of the .pem X.509 certificate file for the certificate. Either `certificatePem` or `certificateWallet` must be set.
     */
    certificatePem?: pulumi.Input<string>;
    /**
     * The contents of the Oracle Wallet certificate for use with SSL, provided as a base64-encoded String. Either `certificatePem` or `certificateWallet` must be set.
     */
    certificateWallet?: pulumi.Input<string>;
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
}
