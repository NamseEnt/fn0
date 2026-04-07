import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage an AWS CloudTrail Delegated Administrator.
 *
 * ## Example Usage
 *
 * Basic usage:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const delegated = aws.getCallerIdentity({});
 * const example = new aws.cloudtrail.OrganizationDelegatedAdminAccount("example", {accountId: delegated.then(delegated => delegated.accountId)});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import delegated administrators using the delegate account `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudtrail/organizationDelegatedAdminAccount:OrganizationDelegatedAdminAccount example 12345678901
 * ```
 */
export declare class OrganizationDelegatedAdminAccount extends pulumi.CustomResource {
    /**
     * Get an existing OrganizationDelegatedAdminAccount resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OrganizationDelegatedAdminAccountState, opts?: pulumi.CustomResourceOptions): OrganizationDelegatedAdminAccount;
    /**
     * Returns true if the given object is an instance of OrganizationDelegatedAdminAccount.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OrganizationDelegatedAdminAccount;
    /**
     * An organization member account ID that you want to designate as a delegated administrator.
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the delegated administrator's account.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The email address that is associated with the delegated administrator's AWS account.
     */
    readonly email: pulumi.Output<string>;
    /**
     * The friendly name of the delegated administrator's account.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The AWS CloudTrail service principal name.
     */
    readonly servicePrincipal: pulumi.Output<string>;
    /**
     * Create a OrganizationDelegatedAdminAccount resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OrganizationDelegatedAdminAccountArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OrganizationDelegatedAdminAccount resources.
 */
export interface OrganizationDelegatedAdminAccountState {
    /**
     * An organization member account ID that you want to designate as a delegated administrator.
     */
    accountId?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the delegated administrator's account.
     */
    arn?: pulumi.Input<string>;
    /**
     * The email address that is associated with the delegated administrator's AWS account.
     */
    email?: pulumi.Input<string>;
    /**
     * The friendly name of the delegated administrator's account.
     */
    name?: pulumi.Input<string>;
    /**
     * The AWS CloudTrail service principal name.
     */
    servicePrincipal?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a OrganizationDelegatedAdminAccount resource.
 */
export interface OrganizationDelegatedAdminAccountArgs {
    /**
     * An organization member account ID that you want to designate as a delegated administrator.
     */
    accountId: pulumi.Input<string>;
}
