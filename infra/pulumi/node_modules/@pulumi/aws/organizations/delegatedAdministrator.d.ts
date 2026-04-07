import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage an [AWS Organizations Delegated Administrator](https://docs.aws.amazon.com/organizations/latest/APIReference/API_RegisterDelegatedAdministrator.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.organizations.DelegatedAdministrator("example", {
 *     accountId: "123456789012",
 *     servicePrincipal: "principal",
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `service_principal` (String) Service principal for the AWS service.
 *
 * * `delegated_account_id` (String) Account ID to be designated as a delegated administrator.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * Using `pulumi import`, import `aws_organizations_delegated_administrator` using the account ID and its service principal. For example:
 *
 * console
 *
 * % pulumi import aws_organizations_delegated_administrator.example 123456789012/config.amazonaws.com
 */
export declare class DelegatedAdministrator extends pulumi.CustomResource {
    /**
     * Get an existing DelegatedAdministrator resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DelegatedAdministratorState, opts?: pulumi.CustomResourceOptions): DelegatedAdministrator;
    /**
     * Returns true if the given object is an instance of DelegatedAdministrator.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DelegatedAdministrator;
    /**
     * The account ID number of the member account in the organization to register as a delegated administrator.
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the delegated administrator's account.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The date when the account was made a delegated administrator.
     */
    readonly delegationEnabledDate: pulumi.Output<string>;
    /**
     * The email address that is associated with the delegated administrator's AWS account.
     */
    readonly email: pulumi.Output<string>;
    /**
     * The method by which the delegated administrator's account joined the organization.
     */
    readonly joinedMethod: pulumi.Output<string>;
    /**
     * The date when the delegated administrator's account became a part of the organization.
     */
    readonly joinedTimestamp: pulumi.Output<string>;
    /**
     * The friendly name of the delegated administrator's account.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The service principal of the AWS service for which you want to make the member account a delegated administrator.
     */
    readonly servicePrincipal: pulumi.Output<string>;
    /**
     * The status of the delegated administrator's account in the organization.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Create a DelegatedAdministrator resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DelegatedAdministratorArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DelegatedAdministrator resources.
 */
export interface DelegatedAdministratorState {
    /**
     * The account ID number of the member account in the organization to register as a delegated administrator.
     */
    accountId?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the delegated administrator's account.
     */
    arn?: pulumi.Input<string>;
    /**
     * The date when the account was made a delegated administrator.
     */
    delegationEnabledDate?: pulumi.Input<string>;
    /**
     * The email address that is associated with the delegated administrator's AWS account.
     */
    email?: pulumi.Input<string>;
    /**
     * The method by which the delegated administrator's account joined the organization.
     */
    joinedMethod?: pulumi.Input<string>;
    /**
     * The date when the delegated administrator's account became a part of the organization.
     */
    joinedTimestamp?: pulumi.Input<string>;
    /**
     * The friendly name of the delegated administrator's account.
     */
    name?: pulumi.Input<string>;
    /**
     * The service principal of the AWS service for which you want to make the member account a delegated administrator.
     */
    servicePrincipal?: pulumi.Input<string>;
    /**
     * The status of the delegated administrator's account in the organization.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DelegatedAdministrator resource.
 */
export interface DelegatedAdministratorArgs {
    /**
     * The account ID number of the member account in the organization to register as a delegated administrator.
     */
    accountId: pulumi.Input<string>;
    /**
     * The service principal of the AWS service for which you want to make the member account a delegated administrator.
     */
    servicePrincipal: pulumi.Input<string>;
}
