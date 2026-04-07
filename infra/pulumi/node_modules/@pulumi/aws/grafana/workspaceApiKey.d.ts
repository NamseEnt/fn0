import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Amazon Managed Grafana workspace API Key resource.
 *
 * ## Example Usage
 *
 * ### Basic configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const key = new aws.grafana.WorkspaceApiKey("key", {
 *     keyName: "test-key",
 *     keyRole: "VIEWER",
 *     secondsToLive: 3600,
 *     workspaceId: test.id,
 * });
 * ```
 */
export declare class WorkspaceApiKey extends pulumi.CustomResource {
    /**
     * Get an existing WorkspaceApiKey resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WorkspaceApiKeyState, opts?: pulumi.CustomResourceOptions): WorkspaceApiKey;
    /**
     * Returns true if the given object is an instance of WorkspaceApiKey.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is WorkspaceApiKey;
    /**
     * The key token in JSON format. Use this value as a bearer token to authenticate HTTP requests to the workspace.
     */
    readonly key: pulumi.Output<string>;
    /**
     * Specifies the name of the API key. Key names must be unique to the workspace.
     */
    readonly keyName: pulumi.Output<string>;
    /**
     * Specifies the permission level of the API key. Valid values are `VIEWER`, `EDITOR`, or `ADMIN`.
     */
    readonly keyRole: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies the time in seconds until the API key expires. Keys can be valid for up to 30 days.
     */
    readonly secondsToLive: pulumi.Output<number>;
    /**
     * The ID of the workspace that the API key is valid for.
     */
    readonly workspaceId: pulumi.Output<string>;
    /**
     * Create a WorkspaceApiKey resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: WorkspaceApiKeyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering WorkspaceApiKey resources.
 */
export interface WorkspaceApiKeyState {
    /**
     * The key token in JSON format. Use this value as a bearer token to authenticate HTTP requests to the workspace.
     */
    key?: pulumi.Input<string>;
    /**
     * Specifies the name of the API key. Key names must be unique to the workspace.
     */
    keyName?: pulumi.Input<string>;
    /**
     * Specifies the permission level of the API key. Valid values are `VIEWER`, `EDITOR`, or `ADMIN`.
     */
    keyRole?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the time in seconds until the API key expires. Keys can be valid for up to 30 days.
     */
    secondsToLive?: pulumi.Input<number>;
    /**
     * The ID of the workspace that the API key is valid for.
     */
    workspaceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a WorkspaceApiKey resource.
 */
export interface WorkspaceApiKeyArgs {
    /**
     * Specifies the name of the API key. Key names must be unique to the workspace.
     */
    keyName: pulumi.Input<string>;
    /**
     * Specifies the permission level of the API key. Valid values are `VIEWER`, `EDITOR`, or `ADMIN`.
     */
    keyRole: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the time in seconds until the API key expires. Keys can be valid for up to 30 days.
     */
    secondsToLive: pulumi.Input<number>;
    /**
     * The ID of the workspace that the API key is valid for.
     */
    workspaceId: pulumi.Input<string>;
}
