import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage an [Amazon Macie Organization Admin Account](https://docs.aws.amazon.com/macie/latest/APIReference/admin.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.macie2.Account("example", {});
 * const exampleOrganizationAdminAccount = new aws.macie2.OrganizationAdminAccount("example", {adminAccountId: "ID OF THE ADMIN ACCOUNT"}, {
 *     dependsOn: [example],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_macie2_organization_admin_account` using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:macie2/organizationAdminAccount:OrganizationAdminAccount example abcd1
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
     * The AWS account ID for the account to designate as the delegated Amazon Macie administrator account for the organization.
     */
    readonly adminAccountId: pulumi.Output<string>;
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
     * The AWS account ID for the account to designate as the delegated Amazon Macie administrator account for the organization.
     */
    adminAccountId?: pulumi.Input<string>;
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
     * The AWS account ID for the account to designate as the delegated Amazon Macie administrator account for the organization.
     */
    adminAccountId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
