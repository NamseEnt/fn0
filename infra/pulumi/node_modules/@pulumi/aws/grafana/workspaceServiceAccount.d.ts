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
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Managed Grafana Workspace Service Account using the `workspace_id` and `service_account_id` separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:grafana/workspaceServiceAccount:WorkspaceServiceAccount example g-abc12345,1
 * ```
 */
export declare class WorkspaceServiceAccount extends pulumi.CustomResource {
    /**
     * Get an existing WorkspaceServiceAccount resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WorkspaceServiceAccountState, opts?: pulumi.CustomResourceOptions): WorkspaceServiceAccount;
    /**
     * Returns true if the given object is an instance of WorkspaceServiceAccount.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is WorkspaceServiceAccount;
    /**
     * The permission level to use for this service account. For more information about the roles and the permissions each has, see the [User roles](https://docs.aws.amazon.com/grafana/latest/userguide/Grafana-user-roles.html) documentation.
     */
    readonly grafanaRole: pulumi.Output<string>;
    /**
     * A name for the service account. The name must be unique within the workspace, as it determines the ID associated with the service account.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Identifier of the service account in the given Grafana workspace
     */
    readonly serviceAccountId: pulumi.Output<string>;
    /**
     * The Grafana workspace with which the service account is associated.
     */
    readonly workspaceId: pulumi.Output<string>;
    /**
     * Create a WorkspaceServiceAccount resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: WorkspaceServiceAccountArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering WorkspaceServiceAccount resources.
 */
export interface WorkspaceServiceAccountState {
    /**
     * The permission level to use for this service account. For more information about the roles and the permissions each has, see the [User roles](https://docs.aws.amazon.com/grafana/latest/userguide/Grafana-user-roles.html) documentation.
     */
    grafanaRole?: pulumi.Input<string>;
    /**
     * A name for the service account. The name must be unique within the workspace, as it determines the ID associated with the service account.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of the service account in the given Grafana workspace
     */
    serviceAccountId?: pulumi.Input<string>;
    /**
     * The Grafana workspace with which the service account is associated.
     */
    workspaceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a WorkspaceServiceAccount resource.
 */
export interface WorkspaceServiceAccountArgs {
    /**
     * The permission level to use for this service account. For more information about the roles and the permissions each has, see the [User roles](https://docs.aws.amazon.com/grafana/latest/userguide/Grafana-user-roles.html) documentation.
     */
    grafanaRole: pulumi.Input<string>;
    /**
     * A name for the service account. The name must be unique within the workspace, as it determines the ID associated with the service account.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Grafana workspace with which the service account is associated.
     */
    workspaceId: pulumi.Input<string>;
}
