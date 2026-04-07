import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing a Roles Anywhere Trust Anchor.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.acmpca.CertificateAuthority("example", {
 *     permanentDeletionTimeInDays: 7,
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
 * const test = new aws.acmpca.Certificate("test", {
 *     certificateAuthorityArn: example.arn,
 *     certificateSigningRequest: example.certificateSigningRequest,
 *     signingAlgorithm: "SHA512WITHRSA",
 *     templateArn: current.then(current => `arn:${current.partition}:acm-pca:::template/RootCACertificate/V1`),
 *     validity: {
 *         type: "YEARS",
 *         value: "1",
 *     },
 * });
 * const exampleCertificateAuthorityCertificate = new aws.acmpca.CertificateAuthorityCertificate("example", {
 *     certificateAuthorityArn: example.arn,
 *     certificate: exampleAwsAcmpcaCertificate.certificate,
 *     certificateChain: exampleAwsAcmpcaCertificate.certificateChain,
 * });
 * const testTrustAnchor = new aws.rolesanywhere.TrustAnchor("test", {
 *     name: "example",
 *     source: {
 *         sourceData: {
 *             acmPcaArn: example.arn,
 *         },
 *         sourceType: "AWS_ACM_PCA",
 *     },
 * }, {
 *     dependsOn: [exampleCertificateAuthorityCertificate],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_rolesanywhere_trust_anchor` using its `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:rolesanywhere/trustAnchor:TrustAnchor example 92b2fbbb-984d-41a3-a765-e3cbdb69ebb1
 * ```
 */
export declare class TrustAnchor extends pulumi.CustomResource {
    /**
     * Get an existing TrustAnchor resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TrustAnchorState, opts?: pulumi.CustomResourceOptions): TrustAnchor;
    /**
     * Returns true if the given object is an instance of TrustAnchor.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TrustAnchor;
    /**
     * Amazon Resource Name (ARN) of the Trust Anchor
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Whether or not the Trust Anchor should be enabled.
     */
    readonly enabled: pulumi.Output<boolean>;
    /**
     * The name of the Trust Anchor.
     */
    readonly name: pulumi.Output<string>;
    readonly notificationSettings: pulumi.Output<outputs.rolesanywhere.TrustAnchorNotificationSetting[]>;
    /**
     * The source of trust, documented below
     */
    readonly source: pulumi.Output<outputs.rolesanywhere.TrustAnchorSource>;
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
     * Create a TrustAnchor resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TrustAnchorArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TrustAnchor resources.
 */
export interface TrustAnchorState {
    /**
     * Amazon Resource Name (ARN) of the Trust Anchor
     */
    arn?: pulumi.Input<string>;
    /**
     * Whether or not the Trust Anchor should be enabled.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * The name of the Trust Anchor.
     */
    name?: pulumi.Input<string>;
    notificationSettings?: pulumi.Input<pulumi.Input<inputs.rolesanywhere.TrustAnchorNotificationSetting>[]>;
    /**
     * The source of trust, documented below
     */
    source?: pulumi.Input<inputs.rolesanywhere.TrustAnchorSource>;
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
 * The set of arguments for constructing a TrustAnchor resource.
 */
export interface TrustAnchorArgs {
    /**
     * Whether or not the Trust Anchor should be enabled.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * The name of the Trust Anchor.
     */
    name?: pulumi.Input<string>;
    notificationSettings?: pulumi.Input<pulumi.Input<inputs.rolesanywhere.TrustAnchorNotificationSetting>[]>;
    /**
     * The source of trust, documented below
     */
    source: pulumi.Input<inputs.rolesanywhere.TrustAnchorSource>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
