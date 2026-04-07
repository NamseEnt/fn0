import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Amazon Managed Grafana workspace role association resource.
 *
 * ## Example Usage
 *
 * ### Basic configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const assume = new aws.iam.Role("assume", {
 *     name: "grafana-assume",
 *     assumeRolePolicy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: "sts:AssumeRole",
 *             Effect: "Allow",
 *             Sid: "",
 *             Principal: {
 *                 Service: "grafana.amazonaws.com",
 *             },
 *         }],
 *     }),
 * });
 * const exampleWorkspace = new aws.grafana.Workspace("example", {
 *     accountAccessType: "CURRENT_ACCOUNT",
 *     authenticationProviders: ["SAML"],
 *     permissionType: "SERVICE_MANAGED",
 *     roleArn: assume.arn,
 * });
 * const example = new aws.grafana.RoleAssociation("example", {
 *     role: "ADMIN",
 *     userIds: [
 *         "USER_ID_1",
 *         "USER_ID_2",
 *     ],
 *     workspaceId: exampleWorkspace.id,
 * });
 * ```
 */
export declare class RoleAssociation extends pulumi.CustomResource {
    /**
     * Get an existing RoleAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RoleAssociationState, opts?: pulumi.CustomResourceOptions): RoleAssociation;
    /**
     * Returns true if the given object is an instance of RoleAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RoleAssociation;
    /**
     * The AWS SSO group ids to be assigned the role given in `role`.
     */
    readonly groupIds: pulumi.Output<string[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The grafana role. Valid values can be found [here](https://docs.aws.amazon.com/grafana/latest/APIReference/API_UpdateInstruction.html#ManagedGrafana-Type-UpdateInstruction-role).
     */
    readonly role: pulumi.Output<string>;
    /**
     * The AWS SSO user ids to be assigned the role given in `role`.
     */
    readonly userIds: pulumi.Output<string[] | undefined>;
    /**
     * The workspace id.
     *
     * The following arguments are optional:
     */
    readonly workspaceId: pulumi.Output<string>;
    /**
     * Create a RoleAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RoleAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RoleAssociation resources.
 */
export interface RoleAssociationState {
    /**
     * The AWS SSO group ids to be assigned the role given in `role`.
     */
    groupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The grafana role. Valid values can be found [here](https://docs.aws.amazon.com/grafana/latest/APIReference/API_UpdateInstruction.html#ManagedGrafana-Type-UpdateInstruction-role).
     */
    role?: pulumi.Input<string>;
    /**
     * The AWS SSO user ids to be assigned the role given in `role`.
     */
    userIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The workspace id.
     *
     * The following arguments are optional:
     */
    workspaceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RoleAssociation resource.
 */
export interface RoleAssociationArgs {
    /**
     * The AWS SSO group ids to be assigned the role given in `role`.
     */
    groupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The grafana role. Valid values can be found [here](https://docs.aws.amazon.com/grafana/latest/APIReference/API_UpdateInstruction.html#ManagedGrafana-Type-UpdateInstruction-role).
     */
    role: pulumi.Input<string>;
    /**
     * The AWS SSO user ids to be assigned the role given in `role`.
     */
    userIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The workspace id.
     *
     * The following arguments are optional:
     */
    workspaceId: pulumi.Input<string>;
}
