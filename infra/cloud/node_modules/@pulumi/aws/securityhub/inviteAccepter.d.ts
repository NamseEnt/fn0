import * as pulumi from "@pulumi/pulumi";
/**
 * > **Note:** AWS accounts can only be associated with a single Security Hub master account. Destroying this resource will disassociate the member account from the master account.
 *
 * Accepts a Security Hub invitation.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.securityhub.Account("example", {});
 * const exampleMember = new aws.securityhub.Member("example", {
 *     accountId: "123456789012",
 *     email: "example@example.com",
 *     invite: true,
 * });
 * const invitee = new aws.securityhub.Account("invitee", {});
 * const inviteeInviteAccepter = new aws.securityhub.InviteAccepter("invitee", {masterId: exampleMember.masterId}, {
 *     dependsOn: [invitee],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Security Hub invite acceptance using the account ID. For example:
 *
 * ```sh
 * $ pulumi import aws:securityhub/inviteAccepter:InviteAccepter example 123456789012
 * ```
 */
export declare class InviteAccepter extends pulumi.CustomResource {
    /**
     * Get an existing InviteAccepter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InviteAccepterState, opts?: pulumi.CustomResourceOptions): InviteAccepter;
    /**
     * Returns true if the given object is an instance of InviteAccepter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is InviteAccepter;
    /**
     * The ID of the invitation.
     */
    readonly invitationId: pulumi.Output<string>;
    /**
     * The account ID of the master Security Hub account whose invitation you're accepting.
     */
    readonly masterId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a InviteAccepter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InviteAccepterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering InviteAccepter resources.
 */
export interface InviteAccepterState {
    /**
     * The ID of the invitation.
     */
    invitationId?: pulumi.Input<string>;
    /**
     * The account ID of the master Security Hub account whose invitation you're accepting.
     */
    masterId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a InviteAccepter resource.
 */
export interface InviteAccepterArgs {
    /**
     * The account ID of the master Security Hub account whose invitation you're accepting.
     */
    masterId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
