import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to accept a pending GuardDuty invite on creation, ensure the detector has the correct primary account on read, and disassociate with the primary account upon removal.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const primary = new aws.guardduty.Detector("primary", {});
 * const memberDetector = new aws.guardduty.Detector("member", {});
 * const memberMember = new aws.guardduty.Member("member", {
 *     accountId: memberDetector.accountId,
 *     detectorId: primary.id,
 *     email: "required@example.com",
 *     invite: true,
 * });
 * const member = new aws.guardduty.InviteAccepter("member", {
 *     detectorId: memberDetector.id,
 *     masterAccountId: primary.accountId,
 * }, {
 *     dependsOn: [memberMember],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_guardduty_invite_accepter` using the member GuardDuty detector ID. For example:
 *
 * ```sh
 * $ pulumi import aws:guardduty/inviteAccepter:InviteAccepter member 00b00fd5aecc0ab60a708659477e9617
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
     * The detector ID of the member GuardDuty account.
     */
    readonly detectorId: pulumi.Output<string>;
    /**
     * AWS account ID for primary account.
     */
    readonly masterAccountId: pulumi.Output<string>;
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
     * The detector ID of the member GuardDuty account.
     */
    detectorId?: pulumi.Input<string>;
    /**
     * AWS account ID for primary account.
     */
    masterAccountId?: pulumi.Input<string>;
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
     * The detector ID of the member GuardDuty account.
     */
    detectorId: pulumi.Input<string>;
    /**
     * AWS account ID for primary account.
     */
    masterAccountId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
