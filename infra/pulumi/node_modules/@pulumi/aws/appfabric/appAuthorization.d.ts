import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS AppFabric App Authorization.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appfabric.AppAuthorization("example", {
 *     app: "TERRAFORMCLOUD",
 *     appBundleArn: arn,
 *     authType: "apiKey",
 *     credential: {
 *         apiKeyCredentials: [{
 *             apiKey: "exampleapikeytoken",
 *         }],
 *     },
 *     tenants: [{
 *         tenantDisplayName: "example",
 *         tenantIdentifier: "example",
 *     }],
 * });
 * ```
 */
export declare class AppAuthorization extends pulumi.CustomResource {
    /**
     * Get an existing AppAuthorization resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AppAuthorizationState, opts?: pulumi.CustomResourceOptions): AppAuthorization;
    /**
     * Returns true if the given object is an instance of AppAuthorization.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AppAuthorization;
    /**
     * The name of the application for valid values see https://docs.aws.amazon.com/appfabric/latest/api/API_CreateAppAuthorization.html.
     */
    readonly app: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the app bundle to use for the request.
     */
    readonly appBundleArn: pulumi.Output<string>;
    /**
     * ARN of the App Authorization. Do not begin the description with "An", "The", "Defines", "Indicates", or "Specifies," as these are verbose. In other words, "Indicates the amount of storage," can be rewritten as "Amount of storage," without losing any information.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The authorization type for the app authorization valid values are oauth2 and apiKey.
     */
    readonly authType: pulumi.Output<string>;
    /**
     * The application URL for the OAuth flow.
     */
    readonly authUrl: pulumi.Output<string>;
    readonly createdAt: pulumi.Output<string>;
    /**
     * Contains credentials for the application, such as an API key or OAuth2 client ID and secret.
     * Specify credentials that match the authorization type for your request. For example, if the authorization type for your request is OAuth2 (oauth2), then you should provide only the OAuth2 credentials.
     */
    readonly credential: pulumi.Output<outputs.appfabric.AppAuthorizationCredential | undefined>;
    /**
     * The user persona of the app authorization.
     */
    readonly persona: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Contains information about an application tenant, such as the application display name and identifier.
     */
    readonly tenants: pulumi.Output<outputs.appfabric.AppAuthorizationTenant[] | undefined>;
    readonly timeouts: pulumi.Output<outputs.appfabric.AppAuthorizationTimeouts | undefined>;
    readonly updatedAt: pulumi.Output<string>;
    /**
     * Create a AppAuthorization resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AppAuthorizationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AppAuthorization resources.
 */
export interface AppAuthorizationState {
    /**
     * The name of the application for valid values see https://docs.aws.amazon.com/appfabric/latest/api/API_CreateAppAuthorization.html.
     */
    app?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the app bundle to use for the request.
     */
    appBundleArn?: pulumi.Input<string>;
    /**
     * ARN of the App Authorization. Do not begin the description with "An", "The", "Defines", "Indicates", or "Specifies," as these are verbose. In other words, "Indicates the amount of storage," can be rewritten as "Amount of storage," without losing any information.
     */
    arn?: pulumi.Input<string>;
    /**
     * The authorization type for the app authorization valid values are oauth2 and apiKey.
     */
    authType?: pulumi.Input<string>;
    /**
     * The application URL for the OAuth flow.
     */
    authUrl?: pulumi.Input<string>;
    createdAt?: pulumi.Input<string>;
    /**
     * Contains credentials for the application, such as an API key or OAuth2 client ID and secret.
     * Specify credentials that match the authorization type for your request. For example, if the authorization type for your request is OAuth2 (oauth2), then you should provide only the OAuth2 credentials.
     */
    credential?: pulumi.Input<inputs.appfabric.AppAuthorizationCredential>;
    /**
     * The user persona of the app authorization.
     */
    persona?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Contains information about an application tenant, such as the application display name and identifier.
     */
    tenants?: pulumi.Input<pulumi.Input<inputs.appfabric.AppAuthorizationTenant>[]>;
    timeouts?: pulumi.Input<inputs.appfabric.AppAuthorizationTimeouts>;
    updatedAt?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AppAuthorization resource.
 */
export interface AppAuthorizationArgs {
    /**
     * The name of the application for valid values see https://docs.aws.amazon.com/appfabric/latest/api/API_CreateAppAuthorization.html.
     */
    app: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the app bundle to use for the request.
     */
    appBundleArn: pulumi.Input<string>;
    /**
     * The authorization type for the app authorization valid values are oauth2 and apiKey.
     */
    authType: pulumi.Input<string>;
    /**
     * Contains credentials for the application, such as an API key or OAuth2 client ID and secret.
     * Specify credentials that match the authorization type for your request. For example, if the authorization type for your request is OAuth2 (oauth2), then you should provide only the OAuth2 credentials.
     */
    credential?: pulumi.Input<inputs.appfabric.AppAuthorizationCredential>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Contains information about an application tenant, such as the application display name and identifier.
     */
    tenants?: pulumi.Input<pulumi.Input<inputs.appfabric.AppAuthorizationTenant>[]>;
    timeouts?: pulumi.Input<inputs.appfabric.AppAuthorizationTimeouts>;
}
