import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AWS Cognito Identity Principal Mapping.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.cognito.UserPool("example", {
 *     name: "user pool",
 *     autoVerifiedAttributes: ["email"],
 * });
 * const exampleUserPoolClient = new aws.cognito.UserPoolClient("example", {
 *     name: "client",
 *     userPoolId: example.id,
 *     supportedIdentityProviders: std.compact({
 *         input: ["COGNITO"],
 *     }).then(invoke => invoke.result),
 * });
 * const exampleIdentityPool = new aws.cognito.IdentityPool("example", {
 *     identityPoolName: "identity pool",
 *     allowUnauthenticatedIdentities: false,
 *     cognitoIdentityProviders: [{
 *         clientId: exampleUserPoolClient.id,
 *         providerName: example.endpoint,
 *         serverSideTokenCheck: false,
 *     }],
 * });
 * const exampleIdentityPoolProviderPrincipalTag = new aws.cognito.IdentityPoolProviderPrincipalTag("example", {
 *     identityPoolId: exampleIdentityPool.id,
 *     identityProviderName: example.endpoint,
 *     useDefaults: false,
 *     principalTags: {
 *         test: "value",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Cognito Identity Pool Roles Attachment using the Identity Pool ID and provider name. For example:
 *
 * ```sh
 * $ pulumi import aws:cognito/identityPoolProviderPrincipalTag:IdentityPoolProviderPrincipalTag example us-west-2_abc123:CorpAD
 * ```
 */
export declare class IdentityPoolProviderPrincipalTag extends pulumi.CustomResource {
    /**
     * Get an existing IdentityPoolProviderPrincipalTag resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: IdentityPoolProviderPrincipalTagState, opts?: pulumi.CustomResourceOptions): IdentityPoolProviderPrincipalTag;
    /**
     * Returns true if the given object is an instance of IdentityPoolProviderPrincipalTag.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is IdentityPoolProviderPrincipalTag;
    /**
     * An identity pool ID.
     */
    readonly identityPoolId: pulumi.Output<string>;
    /**
     * The name of the identity provider.
     */
    readonly identityProviderName: pulumi.Output<string>;
    /**
     * String to string map of variables.
     */
    readonly principalTags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * use default (username and clientID) attribute mappings.
     */
    readonly useDefaults: pulumi.Output<boolean | undefined>;
    /**
     * Create a IdentityPoolProviderPrincipalTag resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: IdentityPoolProviderPrincipalTagArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering IdentityPoolProviderPrincipalTag resources.
 */
export interface IdentityPoolProviderPrincipalTagState {
    /**
     * An identity pool ID.
     */
    identityPoolId?: pulumi.Input<string>;
    /**
     * The name of the identity provider.
     */
    identityProviderName?: pulumi.Input<string>;
    /**
     * String to string map of variables.
     */
    principalTags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * use default (username and clientID) attribute mappings.
     */
    useDefaults?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a IdentityPoolProviderPrincipalTag resource.
 */
export interface IdentityPoolProviderPrincipalTagArgs {
    /**
     * An identity pool ID.
     */
    identityPoolId: pulumi.Input<string>;
    /**
     * The name of the identity provider.
     */
    identityProviderName: pulumi.Input<string>;
    /**
     * String to string map of variables.
     */
    principalTags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * use default (username and clientID) attribute mappings.
     */
    useDefaults?: pulumi.Input<boolean>;
}
