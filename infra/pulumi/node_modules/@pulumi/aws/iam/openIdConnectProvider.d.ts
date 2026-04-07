import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an IAM OpenID Connect provider.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.iam.OpenIdConnectProvider("default", {
 *     url: "https://accounts.google.com",
 *     clientIdLists: ["266362248691-342342xasdasdasda-apps.googleusercontent.com"],
 *     thumbprintLists: ["cf23df2207d99a74fbe169e3eba035e633b65d94"],
 * });
 * ```
 *
 * ### Without A Thumbprint
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.iam.OpenIdConnectProvider("default", {
 *     url: "https://accounts.google.com",
 *     clientIdLists: ["266362248691-342342xasdasdasda-apps.googleusercontent.com"],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the IAM OpenID Connect provider.
 *
 * Using `pulumi import`, import IAM OpenID Connect Providers using the `arn`. For example:
 *
 * console
 *
 * % pulumi import aws_iam_openid_connect_provider.default arn:aws:iam::123456789012:oidc-provider/accounts.google.com
 */
export declare class OpenIdConnectProvider extends pulumi.CustomResource {
    /**
     * Get an existing OpenIdConnectProvider resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OpenIdConnectProviderState, opts?: pulumi.CustomResourceOptions): OpenIdConnectProvider;
    /**
     * Returns true if the given object is an instance of OpenIdConnectProvider.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OpenIdConnectProvider;
    /**
     * ARN assigned by AWS for this provider.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * List of client IDs (audiences) that identify the application registered with the OpenID Connect provider. This is the value sent as the `clientId` parameter in OAuth requests.
     */
    readonly clientIdLists: pulumi.Output<string[]>;
    /**
     * Map of resource tags for the IAM OIDC provider. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly thumbprintLists: pulumi.Output<string[]>;
    /**
     * URL of the identity provider, corresponding to the `iss` claim.
     */
    readonly url: pulumi.Output<string>;
    /**
     * Create a OpenIdConnectProvider resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OpenIdConnectProviderArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OpenIdConnectProvider resources.
 */
export interface OpenIdConnectProviderState {
    /**
     * ARN assigned by AWS for this provider.
     */
    arn?: pulumi.Input<string>;
    /**
     * List of client IDs (audiences) that identify the application registered with the OpenID Connect provider. This is the value sent as the `clientId` parameter in OAuth requests.
     */
    clientIdLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Map of resource tags for the IAM OIDC provider. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    thumbprintLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * URL of the identity provider, corresponding to the `iss` claim.
     */
    url?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a OpenIdConnectProvider resource.
 */
export interface OpenIdConnectProviderArgs {
    /**
     * List of client IDs (audiences) that identify the application registered with the OpenID Connect provider. This is the value sent as the `clientId` parameter in OAuth requests.
     */
    clientIdLists: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Map of resource tags for the IAM OIDC provider. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    thumbprintLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * URL of the identity provider, corresponding to the `iss` claim.
     */
    url: pulumi.Input<string>;
}
