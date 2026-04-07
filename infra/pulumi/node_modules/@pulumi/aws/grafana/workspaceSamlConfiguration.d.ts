import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Amazon Managed Grafana workspace SAML configuration resource.
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
 * const example = new aws.grafana.WorkspaceSamlConfiguration("example", {
 *     editorRoleValues: ["editor"],
 *     idpMetadataUrl: "https://my_idp_metadata.url",
 *     workspaceId: exampleWorkspace.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Grafana Workspace SAML configuration using the workspace's `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:grafana/workspaceSamlConfiguration:WorkspaceSamlConfiguration example g-2054c75a02
 * ```
 */
export declare class WorkspaceSamlConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing WorkspaceSamlConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WorkspaceSamlConfigurationState, opts?: pulumi.CustomResourceOptions): WorkspaceSamlConfiguration;
    /**
     * Returns true if the given object is an instance of WorkspaceSamlConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is WorkspaceSamlConfiguration;
    /**
     * The admin role values.
     */
    readonly adminRoleValues: pulumi.Output<string[] | undefined>;
    /**
     * The allowed organizations.
     */
    readonly allowedOrganizations: pulumi.Output<string[] | undefined>;
    /**
     * The editor role values.
     */
    readonly editorRoleValues: pulumi.Output<string[]>;
    /**
     * The email assertion.
     */
    readonly emailAssertion: pulumi.Output<string>;
    /**
     * The groups assertion.
     */
    readonly groupsAssertion: pulumi.Output<string | undefined>;
    /**
     * The IDP Metadata URL. Note that either `idpMetadataUrl` or `idpMetadataXml` (but not both) must be specified.
     */
    readonly idpMetadataUrl: pulumi.Output<string | undefined>;
    /**
     * The IDP Metadata XML. Note that either `idpMetadataUrl` or `idpMetadataXml` (but not both) must be specified.
     */
    readonly idpMetadataXml: pulumi.Output<string | undefined>;
    /**
     * The login assertion.
     */
    readonly loginAssertion: pulumi.Output<string>;
    /**
     * The login validity duration.
     */
    readonly loginValidityDuration: pulumi.Output<number>;
    /**
     * The name assertion.
     */
    readonly nameAssertion: pulumi.Output<string>;
    /**
     * The org assertion.
     */
    readonly orgAssertion: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The role assertion.
     */
    readonly roleAssertion: pulumi.Output<string | undefined>;
    /**
     * The status of the SAML configuration.
     */
    readonly status: pulumi.Output<string>;
    /**
     * The workspace id.
     *
     * The following arguments are optional:
     */
    readonly workspaceId: pulumi.Output<string>;
    /**
     * Create a WorkspaceSamlConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: WorkspaceSamlConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering WorkspaceSamlConfiguration resources.
 */
export interface WorkspaceSamlConfigurationState {
    /**
     * The admin role values.
     */
    adminRoleValues?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The allowed organizations.
     */
    allowedOrganizations?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The editor role values.
     */
    editorRoleValues?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The email assertion.
     */
    emailAssertion?: pulumi.Input<string>;
    /**
     * The groups assertion.
     */
    groupsAssertion?: pulumi.Input<string>;
    /**
     * The IDP Metadata URL. Note that either `idpMetadataUrl` or `idpMetadataXml` (but not both) must be specified.
     */
    idpMetadataUrl?: pulumi.Input<string>;
    /**
     * The IDP Metadata XML. Note that either `idpMetadataUrl` or `idpMetadataXml` (but not both) must be specified.
     */
    idpMetadataXml?: pulumi.Input<string>;
    /**
     * The login assertion.
     */
    loginAssertion?: pulumi.Input<string>;
    /**
     * The login validity duration.
     */
    loginValidityDuration?: pulumi.Input<number>;
    /**
     * The name assertion.
     */
    nameAssertion?: pulumi.Input<string>;
    /**
     * The org assertion.
     */
    orgAssertion?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The role assertion.
     */
    roleAssertion?: pulumi.Input<string>;
    /**
     * The status of the SAML configuration.
     */
    status?: pulumi.Input<string>;
    /**
     * The workspace id.
     *
     * The following arguments are optional:
     */
    workspaceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a WorkspaceSamlConfiguration resource.
 */
export interface WorkspaceSamlConfigurationArgs {
    /**
     * The admin role values.
     */
    adminRoleValues?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The allowed organizations.
     */
    allowedOrganizations?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The editor role values.
     */
    editorRoleValues: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The email assertion.
     */
    emailAssertion?: pulumi.Input<string>;
    /**
     * The groups assertion.
     */
    groupsAssertion?: pulumi.Input<string>;
    /**
     * The IDP Metadata URL. Note that either `idpMetadataUrl` or `idpMetadataXml` (but not both) must be specified.
     */
    idpMetadataUrl?: pulumi.Input<string>;
    /**
     * The IDP Metadata XML. Note that either `idpMetadataUrl` or `idpMetadataXml` (but not both) must be specified.
     */
    idpMetadataXml?: pulumi.Input<string>;
    /**
     * The login assertion.
     */
    loginAssertion?: pulumi.Input<string>;
    /**
     * The login validity duration.
     */
    loginValidityDuration?: pulumi.Input<number>;
    /**
     * The name assertion.
     */
    nameAssertion?: pulumi.Input<string>;
    /**
     * The org assertion.
     */
    orgAssertion?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The role assertion.
     */
    roleAssertion?: pulumi.Input<string>;
    /**
     * The workspace id.
     *
     * The following arguments are optional:
     */
    workspaceId: pulumi.Input<string>;
}
