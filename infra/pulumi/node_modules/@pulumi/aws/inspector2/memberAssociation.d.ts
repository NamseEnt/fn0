import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for associating accounts to existing Inspector instances.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.inspector2.MemberAssociation("example", {accountId: "123456789012"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Amazon Inspector Member Association using the `account_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:inspector2/memberAssociation:MemberAssociation example 123456789012
 * ```
 */
export declare class MemberAssociation extends pulumi.CustomResource {
    /**
     * Get an existing MemberAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MemberAssociationState, opts?: pulumi.CustomResourceOptions): MemberAssociation;
    /**
     * Returns true if the given object is an instance of MemberAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is MemberAssociation;
    /**
     * ID of the account to associate
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * Account ID of the delegated administrator account
     */
    readonly delegatedAdminAccountId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Status of the member relationship
     */
    readonly relationshipStatus: pulumi.Output<string>;
    /**
     * Date and time of the last update of the relationship
     */
    readonly updatedAt: pulumi.Output<string>;
    /**
     * Create a MemberAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MemberAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering MemberAssociation resources.
 */
export interface MemberAssociationState {
    /**
     * ID of the account to associate
     */
    accountId?: pulumi.Input<string>;
    /**
     * Account ID of the delegated administrator account
     */
    delegatedAdminAccountId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Status of the member relationship
     */
    relationshipStatus?: pulumi.Input<string>;
    /**
     * Date and time of the last update of the relationship
     */
    updatedAt?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a MemberAssociation resource.
 */
export interface MemberAssociationArgs {
    /**
     * ID of the account to associate
     */
    accountId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
