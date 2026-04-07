import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing AWS Audit Manager Organization Admin Account Registration.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.auditmanager.OrganizationAdminAccountRegistration("example", {adminAccountId: "123456789012"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Audit Manager Organization Admin Account Registration using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:auditmanager/organizationAdminAccountRegistration:OrganizationAdminAccountRegistration example 123456789012
 * ```
 */
export declare class OrganizationAdminAccountRegistration extends pulumi.CustomResource {
    /**
     * Get an existing OrganizationAdminAccountRegistration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OrganizationAdminAccountRegistrationState, opts?: pulumi.CustomResourceOptions): OrganizationAdminAccountRegistration;
    /**
     * Returns true if the given object is an instance of OrganizationAdminAccountRegistration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OrganizationAdminAccountRegistration;
    /**
     * Identifier for the organization administrator account.
     */
    readonly adminAccountId: pulumi.Output<string>;
    /**
     * Identifier for the organization.
     */
    readonly organizationId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a OrganizationAdminAccountRegistration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OrganizationAdminAccountRegistrationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OrganizationAdminAccountRegistration resources.
 */
export interface OrganizationAdminAccountRegistrationState {
    /**
     * Identifier for the organization administrator account.
     */
    adminAccountId?: pulumi.Input<string>;
    /**
     * Identifier for the organization.
     */
    organizationId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a OrganizationAdminAccountRegistration resource.
 */
export interface OrganizationAdminAccountRegistrationArgs {
    /**
     * Identifier for the organization administrator account.
     */
    adminAccountId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
