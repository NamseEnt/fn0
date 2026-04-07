import * as pulumi from "@pulumi/pulumi";
/**
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.grafana.WorkspaceServiceAccount("example", {
 *     name: "example-admin",
 *     grafanaRole: "ADMIN",
 *     workspaceId: exampleAwsGrafanaWorkspace.id,
 * });
 * const exampleWorkspaceServiceAccountToken = new aws.grafana.WorkspaceServiceAccountToken("example", {
 *     name: "example-key",
 *     serviceAccountId: example.serviceAccountId,
 *     secondsToLive: 3600,
 *     workspaceId: exampleAwsGrafanaWorkspace.id,
 * });
 * ```
 */
export declare class WorkspaceServiceAccountToken extends pulumi.CustomResource {
    /**
     * Get an existing WorkspaceServiceAccountToken resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WorkspaceServiceAccountTokenState, opts?: pulumi.CustomResourceOptions): WorkspaceServiceAccountToken;
    /**
     * Returns true if the given object is an instance of WorkspaceServiceAccountToken.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is WorkspaceServiceAccountToken;
    /**
     * Specifies when the service account token was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Specifies when the service account token will expire.
     */
    readonly expiresAt: pulumi.Output<string>;
    /**
     * The key for the service account token. Used when making calls to the Grafana HTTP APIs to authenticate and authorize the requests.
     */
    readonly key: pulumi.Output<string>;
    /**
     * A name for the token to create. The name must be unique within the workspace.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Sets how long the token will be valid, in seconds. You can set the time up to 30 days in the future.
     */
    readonly secondsToLive: pulumi.Output<number>;
    /**
     * The ID of the service account for which to create a token.
     */
    readonly serviceAccountId: pulumi.Output<string>;
    /**
     * Identifier of the service account token in the given Grafana workspace.
     */
    readonly serviceAccountTokenId: pulumi.Output<string>;
    /**
     * The Grafana workspace with which the service account token is associated.
     */
    readonly workspaceId: pulumi.Output<string>;
    /**
     * Create a WorkspaceServiceAccountToken resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: WorkspaceServiceAccountTokenArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering WorkspaceServiceAccountToken resources.
 */
export interface WorkspaceServiceAccountTokenState {
    /**
     * Specifies when the service account token was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Specifies when the service account token will expire.
     */
    expiresAt?: pulumi.Input<string>;
    /**
     * The key for the service account token. Used when making calls to the Grafana HTTP APIs to authenticate and authorize the requests.
     */
    key?: pulumi.Input<string>;
    /**
     * A name for the token to create. The name must be unique within the workspace.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Sets how long the token will be valid, in seconds. You can set the time up to 30 days in the future.
     */
    secondsToLive?: pulumi.Input<number>;
    /**
     * The ID of the service account for which to create a token.
     */
    serviceAccountId?: pulumi.Input<string>;
    /**
     * Identifier of the service account token in the given Grafana workspace.
     */
    serviceAccountTokenId?: pulumi.Input<string>;
    /**
     * The Grafana workspace with which the service account token is associated.
     */
    workspaceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a WorkspaceServiceAccountToken resource.
 */
export interface WorkspaceServiceAccountTokenArgs {
    /**
     * A name for the token to create. The name must be unique within the workspace.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Sets how long the token will be valid, in seconds. You can set the time up to 30 days in the future.
     */
    secondsToLive: pulumi.Input<number>;
    /**
     * The ID of the service account for which to create a token.
     */
    serviceAccountId: pulumi.Input<string>;
    /**
     * The Grafana workspace with which the service account token is associated.
     */
    workspaceId: pulumi.Input<string>;
}
