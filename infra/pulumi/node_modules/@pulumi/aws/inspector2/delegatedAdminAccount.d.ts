import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an Amazon Inspector Delegated Admin Account.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const example = new aws.inspector2.DelegatedAdminAccount("example", {accountId: current.then(current => current.accountId)});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Inspector Delegated Admin Account using the `account_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:inspector2/delegatedAdminAccount:DelegatedAdminAccount example 123456789012
 * ```
 */
export declare class DelegatedAdminAccount extends pulumi.CustomResource {
    /**
     * Get an existing DelegatedAdminAccount resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DelegatedAdminAccountState, opts?: pulumi.CustomResourceOptions): DelegatedAdminAccount;
    /**
     * Returns true if the given object is an instance of DelegatedAdminAccount.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DelegatedAdminAccount;
    /**
     * Account to enable as delegated admin account.
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Status of this delegated admin account.
     */
    readonly relationshipStatus: pulumi.Output<string>;
    /**
     * Create a DelegatedAdminAccount resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DelegatedAdminAccountArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DelegatedAdminAccount resources.
 */
export interface DelegatedAdminAccountState {
    /**
     * Account to enable as delegated admin account.
     */
    accountId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Status of this delegated admin account.
     */
    relationshipStatus?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DelegatedAdminAccount resource.
 */
export interface DelegatedAdminAccountArgs {
    /**
     * Account to enable as delegated admin account.
     */
    accountId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
