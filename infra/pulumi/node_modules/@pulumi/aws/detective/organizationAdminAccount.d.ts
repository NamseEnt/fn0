import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Detective Organization Admin Account. The AWS account utilizing this resource must be an Organizations primary account. More information about Organizations support in Detective can be found in the [Detective User Guide](https://docs.aws.amazon.com/detective/latest/adminguide/accounts-orgs-transition.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.organizations.Organization("example", {
 *     awsServiceAccessPrincipals: ["detective.amazonaws.com"],
 *     featureSet: "ALL",
 * });
 * const exampleOrganizationAdminAccount = new aws.detective.OrganizationAdminAccount("example", {accountId: "123456789012"}, {
 *     dependsOn: [example],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_detective_organization_admin_account` using `account_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:detective/organizationAdminAccount:OrganizationAdminAccount example 123456789012
 * ```
 */
export declare class OrganizationAdminAccount extends pulumi.CustomResource {
    /**
     * Get an existing OrganizationAdminAccount resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OrganizationAdminAccountState, opts?: pulumi.CustomResourceOptions): OrganizationAdminAccount;
    /**
     * Returns true if the given object is an instance of OrganizationAdminAccount.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OrganizationAdminAccount;
    /**
     * AWS account identifier to designate as a delegated administrator for Detective.
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a OrganizationAdminAccount resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OrganizationAdminAccountArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OrganizationAdminAccount resources.
 */
export interface OrganizationAdminAccountState {
    /**
     * AWS account identifier to designate as a delegated administrator for Detective.
     */
    accountId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a OrganizationAdminAccount resource.
 */
export interface OrganizationAdminAccountArgs {
    /**
     * AWS account identifier to designate as a delegated administrator for Detective.
     */
    accountId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
