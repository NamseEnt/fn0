import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS AppFabric App Authorization Connection.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appfabric.AppAuthorizationConnection("example", {
 *     appAuthorizationArn: test.arn,
 *     appBundleArn: arn,
 * });
 * ```
 */
export declare class AppAuthorizationConnection extends pulumi.CustomResource {
    /**
     * Get an existing AppAuthorizationConnection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AppAuthorizationConnectionState, opts?: pulumi.CustomResourceOptions): AppAuthorizationConnection;
    /**
     * Returns true if the given object is an instance of AppAuthorizationConnection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AppAuthorizationConnection;
    /**
     * The name of the application.
     */
    readonly app: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) or Universal Unique Identifier (UUID) of the app authorization to use for the request.
     */
    readonly appAuthorizationArn: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the app bundle to use for the request.
     */
    readonly appBundleArn: pulumi.Output<string>;
    /**
     * Contains OAuth2 authorization information.This is required if the app authorization for the request is configured with an OAuth2 (oauth2) authorization type.
     */
    readonly authRequest: pulumi.Output<outputs.appfabric.AppAuthorizationConnectionAuthRequest | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Contains information about an application tenant, such as the application display name and identifier.
     */
    readonly tenants: pulumi.Output<outputs.appfabric.AppAuthorizationConnectionTenant[]>;
    readonly timeouts: pulumi.Output<outputs.appfabric.AppAuthorizationConnectionTimeouts | undefined>;
    /**
     * Create a AppAuthorizationConnection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AppAuthorizationConnectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AppAuthorizationConnection resources.
 */
export interface AppAuthorizationConnectionState {
    /**
     * The name of the application.
     */
    app?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) or Universal Unique Identifier (UUID) of the app authorization to use for the request.
     */
    appAuthorizationArn?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the app bundle to use for the request.
     */
    appBundleArn?: pulumi.Input<string>;
    /**
     * Contains OAuth2 authorization information.This is required if the app authorization for the request is configured with an OAuth2 (oauth2) authorization type.
     */
    authRequest?: pulumi.Input<inputs.appfabric.AppAuthorizationConnectionAuthRequest>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Contains information about an application tenant, such as the application display name and identifier.
     */
    tenants?: pulumi.Input<pulumi.Input<inputs.appfabric.AppAuthorizationConnectionTenant>[]>;
    timeouts?: pulumi.Input<inputs.appfabric.AppAuthorizationConnectionTimeouts>;
}
/**
 * The set of arguments for constructing a AppAuthorizationConnection resource.
 */
export interface AppAuthorizationConnectionArgs {
    /**
     * The Amazon Resource Name (ARN) or Universal Unique Identifier (UUID) of the app authorization to use for the request.
     */
    appAuthorizationArn: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the app bundle to use for the request.
     */
    appBundleArn: pulumi.Input<string>;
    /**
     * Contains OAuth2 authorization information.This is required if the app authorization for the request is configured with an OAuth2 (oauth2) authorization type.
     */
    authRequest?: pulumi.Input<inputs.appfabric.AppAuthorizationConnectionAuthRequest>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.appfabric.AppAuthorizationConnectionTimeouts>;
}
