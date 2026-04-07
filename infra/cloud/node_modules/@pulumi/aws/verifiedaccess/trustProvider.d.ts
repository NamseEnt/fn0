import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing a Verified Access Trust Provider.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.verifiedaccess.TrustProvider("example", {
 *     policyReferenceName: "example",
 *     trustProviderType: "user",
 *     userTrustProviderType: "iam-identity-center",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Transfer Workflows using the  `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:verifiedaccess/trustProvider:TrustProvider example vatp-8012925589
 * ```
 */
export declare class TrustProvider extends pulumi.CustomResource {
    /**
     * Get an existing TrustProvider resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TrustProviderState, opts?: pulumi.CustomResourceOptions): TrustProvider;
    /**
     * Returns true if the given object is an instance of TrustProvider.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TrustProvider;
    /**
     * A description for the AWS Verified Access trust provider.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * A block of options for device identity based trust providers.
     */
    readonly deviceOptions: pulumi.Output<outputs.verifiedaccess.TrustProviderDeviceOptions | undefined>;
    /**
     * The type of device-based trust provider.
     */
    readonly deviceTrustProviderType: pulumi.Output<string | undefined>;
    /**
     * The OpenID Connect details for an Native Application OIDC, user-identity based trust provider.
     */
    readonly nativeApplicationOidcOptions: pulumi.Output<outputs.verifiedaccess.TrustProviderNativeApplicationOidcOptions | undefined>;
    /**
     * The OpenID Connect details for an oidc-type, user-identity based trust provider.
     */
    readonly oidcOptions: pulumi.Output<outputs.verifiedaccess.TrustProviderOidcOptions | undefined>;
    /**
     * The identifier to be used when working with policy rules.
     */
    readonly policyReferenceName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly sseSpecification: pulumi.Output<outputs.verifiedaccess.TrustProviderSseSpecification>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The type of trust provider can be either user or device-based.
     *
     * The following arguments are optional:
     */
    readonly trustProviderType: pulumi.Output<string>;
    /**
     * The type of user-based trust provider.
     */
    readonly userTrustProviderType: pulumi.Output<string | undefined>;
    /**
     * Create a TrustProvider resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TrustProviderArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TrustProvider resources.
 */
export interface TrustProviderState {
    /**
     * A description for the AWS Verified Access trust provider.
     */
    description?: pulumi.Input<string>;
    /**
     * A block of options for device identity based trust providers.
     */
    deviceOptions?: pulumi.Input<inputs.verifiedaccess.TrustProviderDeviceOptions>;
    /**
     * The type of device-based trust provider.
     */
    deviceTrustProviderType?: pulumi.Input<string>;
    /**
     * The OpenID Connect details for an Native Application OIDC, user-identity based trust provider.
     */
    nativeApplicationOidcOptions?: pulumi.Input<inputs.verifiedaccess.TrustProviderNativeApplicationOidcOptions>;
    /**
     * The OpenID Connect details for an oidc-type, user-identity based trust provider.
     */
    oidcOptions?: pulumi.Input<inputs.verifiedaccess.TrustProviderOidcOptions>;
    /**
     * The identifier to be used when working with policy rules.
     */
    policyReferenceName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    sseSpecification?: pulumi.Input<inputs.verifiedaccess.TrustProviderSseSpecification>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The type of trust provider can be either user or device-based.
     *
     * The following arguments are optional:
     */
    trustProviderType?: pulumi.Input<string>;
    /**
     * The type of user-based trust provider.
     */
    userTrustProviderType?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TrustProvider resource.
 */
export interface TrustProviderArgs {
    /**
     * A description for the AWS Verified Access trust provider.
     */
    description?: pulumi.Input<string>;
    /**
     * A block of options for device identity based trust providers.
     */
    deviceOptions?: pulumi.Input<inputs.verifiedaccess.TrustProviderDeviceOptions>;
    /**
     * The type of device-based trust provider.
     */
    deviceTrustProviderType?: pulumi.Input<string>;
    /**
     * The OpenID Connect details for an Native Application OIDC, user-identity based trust provider.
     */
    nativeApplicationOidcOptions?: pulumi.Input<inputs.verifiedaccess.TrustProviderNativeApplicationOidcOptions>;
    /**
     * The OpenID Connect details for an oidc-type, user-identity based trust provider.
     */
    oidcOptions?: pulumi.Input<inputs.verifiedaccess.TrustProviderOidcOptions>;
    /**
     * The identifier to be used when working with policy rules.
     */
    policyReferenceName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    sseSpecification?: pulumi.Input<inputs.verifiedaccess.TrustProviderSseSpecification>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The type of trust provider can be either user or device-based.
     *
     * The following arguments are optional:
     */
    trustProviderType: pulumi.Input<string>;
    /**
     * The type of user-based trust provider.
     */
    userTrustProviderType?: pulumi.Input<string>;
}
