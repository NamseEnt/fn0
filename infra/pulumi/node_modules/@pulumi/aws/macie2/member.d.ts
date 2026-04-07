import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage an [Amazon Macie Member](https://docs.aws.amazon.com/macie/latest/APIReference/members-id.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.macie2.Account("example", {});
 * const exampleMember = new aws.macie2.Member("example", {
 *     accountId: "AWS ACCOUNT ID",
 *     email: "EMAIL",
 *     invite: true,
 *     invitationMessage: "Message of the invitation",
 *     invitationDisableEmailNotification: true,
 * }, {
 *     dependsOn: [example],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_macie2_member` using the account ID of the member account. For example:
 *
 * ```sh
 * $ pulumi import aws:macie2/member:Member example 123456789012
 * ```
 */
export declare class Member extends pulumi.CustomResource {
    /**
     * Get an existing Member resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MemberState, opts?: pulumi.CustomResourceOptions): Member;
    /**
     * Returns true if the given object is an instance of Member.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Member;
    /**
     * The AWS account ID for the account.
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * The AWS account ID for the administrator account.
     */
    readonly administratorAccountId: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the account.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The email address for the account.
     */
    readonly email: pulumi.Output<string>;
    /**
     * Specifies whether to send an email notification to the root user of each account that the invitation will be sent to. This notification is in addition to an alert that the root user receives in AWS Personal Health Dashboard. To send an email notification to the root user of each account, set this value to `true`.
     */
    readonly invitationDisableEmailNotification: pulumi.Output<boolean | undefined>;
    /**
     * A custom message to include in the invitation. Amazon Macie adds this message to the standard content that it sends for an invitation.
     */
    readonly invitationMessage: pulumi.Output<string | undefined>;
    /**
     * Send an invitation to a member
     */
    readonly invite: pulumi.Output<boolean>;
    /**
     * The date and time, in UTC and extended RFC 3339 format, when an Amazon Macie membership invitation was last sent to the account. This value is null if a Macie invitation hasn't been sent to the account.
     */
    readonly invitedAt: pulumi.Output<string>;
    readonly masterAccountId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The current status of the relationship between the account and the administrator account.
     */
    readonly relationshipStatus: pulumi.Output<string>;
    /**
     * Specifies the status for the account. To enable Amazon Macie and start all Macie activities for the account, set this value to `ENABLED`. Valid values are `ENABLED` or `PAUSED`.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The date and time, in UTC and extended RFC 3339 format, of the most recent change to the status of the relationship between the account and the administrator account.
     */
    readonly updatedAt: pulumi.Output<string>;
    /**
     * Create a Member resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MemberArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Member resources.
 */
export interface MemberState {
    /**
     * The AWS account ID for the account.
     */
    accountId?: pulumi.Input<string>;
    /**
     * The AWS account ID for the administrator account.
     */
    administratorAccountId?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the account.
     */
    arn?: pulumi.Input<string>;
    /**
     * The email address for the account.
     */
    email?: pulumi.Input<string>;
    /**
     * Specifies whether to send an email notification to the root user of each account that the invitation will be sent to. This notification is in addition to an alert that the root user receives in AWS Personal Health Dashboard. To send an email notification to the root user of each account, set this value to `true`.
     */
    invitationDisableEmailNotification?: pulumi.Input<boolean>;
    /**
     * A custom message to include in the invitation. Amazon Macie adds this message to the standard content that it sends for an invitation.
     */
    invitationMessage?: pulumi.Input<string>;
    /**
     * Send an invitation to a member
     */
    invite?: pulumi.Input<boolean>;
    /**
     * The date and time, in UTC and extended RFC 3339 format, when an Amazon Macie membership invitation was last sent to the account. This value is null if a Macie invitation hasn't been sent to the account.
     */
    invitedAt?: pulumi.Input<string>;
    masterAccountId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The current status of the relationship between the account and the administrator account.
     */
    relationshipStatus?: pulumi.Input<string>;
    /**
     * Specifies the status for the account. To enable Amazon Macie and start all Macie activities for the account, set this value to `ENABLED`. Valid values are `ENABLED` or `PAUSED`.
     */
    status?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The date and time, in UTC and extended RFC 3339 format, of the most recent change to the status of the relationship between the account and the administrator account.
     */
    updatedAt?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Member resource.
 */
export interface MemberArgs {
    /**
     * The AWS account ID for the account.
     */
    accountId: pulumi.Input<string>;
    /**
     * The email address for the account.
     */
    email: pulumi.Input<string>;
    /**
     * Specifies whether to send an email notification to the root user of each account that the invitation will be sent to. This notification is in addition to an alert that the root user receives in AWS Personal Health Dashboard. To send an email notification to the root user of each account, set this value to `true`.
     */
    invitationDisableEmailNotification?: pulumi.Input<boolean>;
    /**
     * A custom message to include in the invitation. Amazon Macie adds this message to the standard content that it sends for an invitation.
     */
    invitationMessage?: pulumi.Input<string>;
    /**
     * Send an invitation to a member
     */
    invite?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the status for the account. To enable Amazon Macie and start all Macie activities for the account, set this value to `ENABLED`. Valid values are `ENABLED` or `PAUSED`.
     */
    status?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
