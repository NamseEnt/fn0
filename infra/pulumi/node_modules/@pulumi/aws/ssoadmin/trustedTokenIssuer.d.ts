import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS SSO Admin Trusted Token Issuer.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getInstances({});
 * const exampleTrustedTokenIssuer = new aws.ssoadmin.TrustedTokenIssuer("example", {
 *     name: "example",
 *     instanceArn: example.then(example => example.arns?.[0]),
 *     trustedTokenIssuerType: "OIDC_JWT",
 *     trustedTokenIssuerConfiguration: {
 *         oidcJwtConfiguration: {
 *             claimAttributePath: "email",
 *             identityStoreAttributePath: "emails.value",
 *             issuerUrl: "https://example.com",
 *             jwksRetrievalOption: "OPEN_ID_DISCOVERY",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SSO Admin Trusted Token Issuer using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ssoadmin/trustedTokenIssuer:TrustedTokenIssuer example arn:aws:sso::123456789012:trustedTokenIssuer/ssoins-lu1ye3gew4mbc7ju/tti-2657c556-9707-11ee-b9d1-0242ac120002
 * ```
 */
export declare class TrustedTokenIssuer extends pulumi.CustomResource {
    /**
     * Get an existing TrustedTokenIssuer resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TrustedTokenIssuerState, opts?: pulumi.CustomResourceOptions): TrustedTokenIssuer;
    /**
     * Returns true if the given object is an instance of TrustedTokenIssuer.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TrustedTokenIssuer;
    /**
     * ARN of the trusted token issuer.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A unique, case-sensitive ID that you provide to ensure the idempotency of the request. AWS generates a random value when not provided.
     */
    readonly clientToken: pulumi.Output<string | undefined>;
    /**
     * ARN of the instance of IAM Identity Center.
     */
    readonly instanceArn: pulumi.Output<string>;
    /**
     * Name of the trusted token issuer.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * A block that specifies settings that apply to the trusted token issuer, these change depending on the type you specify in `trustedTokenIssuerType`. Documented below.
     */
    readonly trustedTokenIssuerConfiguration: pulumi.Output<outputs.ssoadmin.TrustedTokenIssuerTrustedTokenIssuerConfiguration | undefined>;
    /**
     * Specifies the type of the trusted token issuer. Valid values are `OIDC_JWT`
     *
     * The following arguments are optional:
     */
    readonly trustedTokenIssuerType: pulumi.Output<string>;
    /**
     * Create a TrustedTokenIssuer resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TrustedTokenIssuerArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TrustedTokenIssuer resources.
 */
export interface TrustedTokenIssuerState {
    /**
     * ARN of the trusted token issuer.
     */
    arn?: pulumi.Input<string>;
    /**
     * A unique, case-sensitive ID that you provide to ensure the idempotency of the request. AWS generates a random value when not provided.
     */
    clientToken?: pulumi.Input<string>;
    /**
     * ARN of the instance of IAM Identity Center.
     */
    instanceArn?: pulumi.Input<string>;
    /**
     * Name of the trusted token issuer.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * A block that specifies settings that apply to the trusted token issuer, these change depending on the type you specify in `trustedTokenIssuerType`. Documented below.
     */
    trustedTokenIssuerConfiguration?: pulumi.Input<inputs.ssoadmin.TrustedTokenIssuerTrustedTokenIssuerConfiguration>;
    /**
     * Specifies the type of the trusted token issuer. Valid values are `OIDC_JWT`
     *
     * The following arguments are optional:
     */
    trustedTokenIssuerType?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TrustedTokenIssuer resource.
 */
export interface TrustedTokenIssuerArgs {
    /**
     * A unique, case-sensitive ID that you provide to ensure the idempotency of the request. AWS generates a random value when not provided.
     */
    clientToken?: pulumi.Input<string>;
    /**
     * ARN of the instance of IAM Identity Center.
     */
    instanceArn: pulumi.Input<string>;
    /**
     * Name of the trusted token issuer.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A block that specifies settings that apply to the trusted token issuer, these change depending on the type you specify in `trustedTokenIssuerType`. Documented below.
     */
    trustedTokenIssuerConfiguration?: pulumi.Input<inputs.ssoadmin.TrustedTokenIssuerTrustedTokenIssuerConfiguration>;
    /**
     * Specifies the type of the trusted token issuer. Valid values are `OIDC_JWT`
     *
     * The following arguments are optional:
     */
    trustedTokenIssuerType: pulumi.Input<string>;
}
