import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing AWS Audit Manager Account Registration.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.auditmanager.AccountRegistration("example", {});
 * ```
 *
 * ### Deregister On Destroy
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.auditmanager.AccountRegistration("example", {deregisterOnDestroy: true});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Audit Manager Account Registration resources using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:auditmanager/accountRegistration:AccountRegistration example us-east-1
 * ```
 */
export declare class AccountRegistration extends pulumi.CustomResource {
    /**
     * Get an existing AccountRegistration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AccountRegistrationState, opts?: pulumi.CustomResourceOptions): AccountRegistration;
    /**
     * Returns true if the given object is an instance of AccountRegistration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AccountRegistration;
    /**
     * Identifier for the delegated administrator account.
     */
    readonly delegatedAdminAccount: pulumi.Output<string | undefined>;
    /**
     * Flag to deregister AuditManager in the account upon destruction. Defaults to `false` (ie. AuditManager will remain active in the account, even if this resource is removed).
     */
    readonly deregisterOnDestroy: pulumi.Output<boolean | undefined>;
    /**
     * KMS key identifier.
     */
    readonly kmsKey: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Status of the account registration request.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Create a AccountRegistration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: AccountRegistrationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AccountRegistration resources.
 */
export interface AccountRegistrationState {
    /**
     * Identifier for the delegated administrator account.
     */
    delegatedAdminAccount?: pulumi.Input<string>;
    /**
     * Flag to deregister AuditManager in the account upon destruction. Defaults to `false` (ie. AuditManager will remain active in the account, even if this resource is removed).
     */
    deregisterOnDestroy?: pulumi.Input<boolean>;
    /**
     * KMS key identifier.
     */
    kmsKey?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Status of the account registration request.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AccountRegistration resource.
 */
export interface AccountRegistrationArgs {
    /**
     * Identifier for the delegated administrator account.
     */
    delegatedAdminAccount?: pulumi.Input<string>;
    /**
     * Flag to deregister AuditManager in the account upon destruction. Defaults to `false` (ie. AuditManager will remain active in the account, even if this resource is removed).
     */
    deregisterOnDestroy?: pulumi.Input<boolean>;
    /**
     * KMS key identifier.
     */
    kmsKey?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
