import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Cognito User Identity Provider resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cognito.UserPool("example", {
 *     name: "example-pool",
 *     autoVerifiedAttributes: ["email"],
 * });
 * const exampleProvider = new aws.cognito.IdentityProvider("example_provider", {
 *     userPoolId: example.id,
 *     providerName: "Google",
 *     providerType: "Google",
 *     providerDetails: {
 *         authorize_scopes: "email",
 *         client_id: "your client_id",
 *         client_secret: "your client_secret",
 *     },
 *     attributeMapping: {
 *         email: "email",
 *         username: "sub",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_cognito_identity_provider` resources using their User Pool ID and Provider Name. For example:
 *
 * ```sh
 * $ pulumi import aws:cognito/identityProvider:IdentityProvider example us-west-2_abc123:CorpAD
 * ```
 */
export declare class IdentityProvider extends pulumi.CustomResource {
    /**
     * Get an existing IdentityProvider resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: IdentityProviderState, opts?: pulumi.CustomResourceOptions): IdentityProvider;
    /**
     * Returns true if the given object is an instance of IdentityProvider.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is IdentityProvider;
    /**
     * The map of attribute mapping of user pool attributes. [AttributeMapping in AWS API documentation](https://docs.aws.amazon.com/cognito-user-identity-pools/latest/APIReference/API_CreateIdentityProvider.html#CognitoUserPools-CreateIdentityProvider-request-AttributeMapping)
     */
    readonly attributeMapping: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The list of identity providers.
     */
    readonly idpIdentifiers: pulumi.Output<string[] | undefined>;
    /**
     * The map of identity details, such as access token
     */
    readonly providerDetails: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The provider name
     */
    readonly providerName: pulumi.Output<string>;
    /**
     * The provider type.  [See AWS API for valid values](https://docs.aws.amazon.com/cognito-user-identity-pools/latest/APIReference/API_CreateIdentityProvider.html#CognitoUserPools-CreateIdentityProvider-request-ProviderType)
     */
    readonly providerType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The user pool id
     */
    readonly userPoolId: pulumi.Output<string>;
    /**
     * Create a IdentityProvider resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: IdentityProviderArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering IdentityProvider resources.
 */
export interface IdentityProviderState {
    /**
     * The map of attribute mapping of user pool attributes. [AttributeMapping in AWS API documentation](https://docs.aws.amazon.com/cognito-user-identity-pools/latest/APIReference/API_CreateIdentityProvider.html#CognitoUserPools-CreateIdentityProvider-request-AttributeMapping)
     */
    attributeMapping?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The list of identity providers.
     */
    idpIdentifiers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The map of identity details, such as access token
     */
    providerDetails?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The provider name
     */
    providerName?: pulumi.Input<string>;
    /**
     * The provider type.  [See AWS API for valid values](https://docs.aws.amazon.com/cognito-user-identity-pools/latest/APIReference/API_CreateIdentityProvider.html#CognitoUserPools-CreateIdentityProvider-request-ProviderType)
     */
    providerType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The user pool id
     */
    userPoolId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a IdentityProvider resource.
 */
export interface IdentityProviderArgs {
    /**
     * The map of attribute mapping of user pool attributes. [AttributeMapping in AWS API documentation](https://docs.aws.amazon.com/cognito-user-identity-pools/latest/APIReference/API_CreateIdentityProvider.html#CognitoUserPools-CreateIdentityProvider-request-AttributeMapping)
     */
    attributeMapping?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The list of identity providers.
     */
    idpIdentifiers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The map of identity details, such as access token
     */
    providerDetails: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The provider name
     */
    providerName: pulumi.Input<string>;
    /**
     * The provider type.  [See AWS API for valid values](https://docs.aws.amazon.com/cognito-user-identity-pools/latest/APIReference/API_CreateIdentityProvider.html#CognitoUserPools-CreateIdentityProvider-request-ProviderType)
     */
    providerType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The user pool id
     */
    userPoolId: pulumi.Input<string>;
}
