import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage an [Amazon Detective Member](https://docs.aws.amazon.com/detective/latest/APIReference/API_CreateMembers.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.detective.Graph("example", {});
 * const exampleMember = new aws.detective.Member("example", {
 *     accountId: "AWS ACCOUNT ID",
 *     emailAddress: "EMAIL",
 *     graphArn: example.graphArn,
 *     message: "Message of the invitation",
 *     disableEmailNotification: true,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_detective_member` using the ARN of the graph followed by the account ID of the member account. For example:
 *
 * ```sh
 * $ pulumi import aws:detective/member:Member example arn:aws:detective:us-east-1:123456789101:graph:231684d34gh74g4bae1dbc7bd807d02d/123456789012
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
     * AWS account ID for the account.
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * AWS account ID for the administrator account.
     */
    readonly administratorId: pulumi.Output<string>;
    /**
     * If set to true, then the root user of the invited account will _not_ receive an email notification. This notification is in addition to an alert that the root user receives in AWS Personal Health Dashboard. By default, this is set to `false`.
     */
    readonly disableEmailNotification: pulumi.Output<boolean | undefined>;
    readonly disabledReason: pulumi.Output<string>;
    /**
     * Email address for the account.
     */
    readonly emailAddress: pulumi.Output<string>;
    /**
     * ARN of the behavior graph to invite the member accounts to contribute their data to.
     */
    readonly graphArn: pulumi.Output<string>;
    /**
     * Date and time, in UTC and extended RFC 3339 format, when an Amazon Detective membership invitation was last sent to the account.
     */
    readonly invitedTime: pulumi.Output<string>;
    /**
     * A custom message to include in the invitation. Amazon Detective adds this message to the standard content that it sends for an invitation.
     */
    readonly message: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Current membership status of the member account.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Date and time, in UTC and extended RFC 3339 format, of the most recent change to the member account's status.
     */
    readonly updatedTime: pulumi.Output<string>;
    /**
     * Data volume in bytes per day for the member account.
     */
    readonly volumeUsageInBytes: pulumi.Output<string>;
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
     * AWS account ID for the account.
     */
    accountId?: pulumi.Input<string>;
    /**
     * AWS account ID for the administrator account.
     */
    administratorId?: pulumi.Input<string>;
    /**
     * If set to true, then the root user of the invited account will _not_ receive an email notification. This notification is in addition to an alert that the root user receives in AWS Personal Health Dashboard. By default, this is set to `false`.
     */
    disableEmailNotification?: pulumi.Input<boolean>;
    disabledReason?: pulumi.Input<string>;
    /**
     * Email address for the account.
     */
    emailAddress?: pulumi.Input<string>;
    /**
     * ARN of the behavior graph to invite the member accounts to contribute their data to.
     */
    graphArn?: pulumi.Input<string>;
    /**
     * Date and time, in UTC and extended RFC 3339 format, when an Amazon Detective membership invitation was last sent to the account.
     */
    invitedTime?: pulumi.Input<string>;
    /**
     * A custom message to include in the invitation. Amazon Detective adds this message to the standard content that it sends for an invitation.
     */
    message?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Current membership status of the member account.
     */
    status?: pulumi.Input<string>;
    /**
     * Date and time, in UTC and extended RFC 3339 format, of the most recent change to the member account's status.
     */
    updatedTime?: pulumi.Input<string>;
    /**
     * Data volume in bytes per day for the member account.
     */
    volumeUsageInBytes?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Member resource.
 */
export interface MemberArgs {
    /**
     * AWS account ID for the account.
     */
    accountId: pulumi.Input<string>;
    /**
     * If set to true, then the root user of the invited account will _not_ receive an email notification. This notification is in addition to an alert that the root user receives in AWS Personal Health Dashboard. By default, this is set to `false`.
     */
    disableEmailNotification?: pulumi.Input<boolean>;
    /**
     * Email address for the account.
     */
    emailAddress: pulumi.Input<string>;
    /**
     * ARN of the behavior graph to invite the member accounts to contribute their data to.
     */
    graphArn: pulumi.Input<string>;
    /**
     * A custom message to include in the invitation. Amazon Detective adds this message to the standard content that it sends for an invitation.
     */
    message?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
