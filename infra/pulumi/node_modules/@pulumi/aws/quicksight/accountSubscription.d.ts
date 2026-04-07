import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS QuickSight Account Subscription.
 *
 * > Due to the absence of the `adminGroup`, `authorGroup`, `readerGroup`, `adminProGroup`, `authorProGroup`, and `readerProGroup` fields in the [`DescribeAccountSettings`](https://docs.aws.amazon.com/quicksight/latest/APIReference/API_DescribeAccountSettings.html) API response, changes made to these groups post-subscription will not be detected by this resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const subscription = new aws.quicksight.AccountSubscription("subscription", {
 *     accountName: "quicksight-pulumi",
 *     authenticationMethod: "IAM_AND_QUICKSIGHT",
 *     edition: "ENTERPRISE",
 *     notificationEmail: "notification@email.com",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a QuickSight Account Subscription using `aws_account_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/accountSubscription:AccountSubscription example "012345678901"
 * ```
 */
export declare class AccountSubscription extends pulumi.CustomResource {
    /**
     * Get an existing AccountSubscription resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AccountSubscriptionState, opts?: pulumi.CustomResourceOptions): AccountSubscription;
    /**
     * Returns true if the given object is an instance of AccountSubscription.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AccountSubscription;
    /**
     * Name of your Amazon QuickSight account. This name is unique over all of AWS, and it appears only when users sign in.
     */
    readonly accountName: pulumi.Output<string>;
    /**
     * Status of the Amazon QuickSight account's subscription.
     */
    readonly accountSubscriptionStatus: pulumi.Output<string>;
    /**
     * Name of your Active Directory. This field is required if `ACTIVE_DIRECTORY` is the selected authentication method of the new Amazon QuickSight account.
     */
    readonly activeDirectoryName: pulumi.Output<string | undefined>;
    /**
     * Admin group associated with your Active Directory or IAM Identity Center account. This field is required if `ACTIVE_DIRECTORY` or `IAM_IDENTITY_CENTER` is the selected authentication method of the new Amazon QuickSight account.
     */
    readonly adminGroups: pulumi.Output<string[] | undefined>;
    /**
     * Admin PRO group associated with your Active Directory or IAM Identity Center account.
     */
    readonly adminProGroups: pulumi.Output<string[] | undefined>;
    /**
     * Method that you want to use to authenticate your Amazon QuickSight account. Currently, the valid values for this parameter are `IAM_AND_QUICKSIGHT`, `IAM_ONLY`, `IAM_IDENTITY_CENTER`, and `ACTIVE_DIRECTORY`.
     */
    readonly authenticationMethod: pulumi.Output<string>;
    /**
     * Author group associated with your Active Directory or IAM Identity Center account.
     */
    readonly authorGroups: pulumi.Output<string[] | undefined>;
    /**
     * Author PRO group associated with your Active Directory or IAM Identity Center account.
     */
    readonly authorProGroups: pulumi.Output<string[] | undefined>;
    /**
     * AWS account ID. Defaults to automatically determined account ID of the Pulumi AWS provider.
     */
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * A 10-digit phone number for the author of the Amazon QuickSight account to use for future communications. This field is required if `ENTERPPRISE_AND_Q` is the selected edition of the new Amazon QuickSight account.
     */
    readonly contactNumber: pulumi.Output<string | undefined>;
    /**
     * Active Directory ID that is associated with your Amazon QuickSight account.
     */
    readonly directoryId: pulumi.Output<string | undefined>;
    /**
     * Edition of Amazon QuickSight that you want your account to have. Currently, you can choose from `STANDARD`, `ENTERPRISE` or `ENTERPRISE_AND_Q`.
     */
    readonly edition: pulumi.Output<string>;
    /**
     * Email address of the author of the Amazon QuickSight account to use for future communications. This field is required if `ENTERPPRISE_AND_Q` is the selected edition of the new Amazon QuickSight account.
     */
    readonly emailAddress: pulumi.Output<string | undefined>;
    /**
     * First name of the author of the Amazon QuickSight account to use for future communications. This field is required if `ENTERPPRISE_AND_Q` is the selected edition of the new Amazon QuickSight account.
     */
    readonly firstName: pulumi.Output<string | undefined>;
    /**
     * The Amazon Resource Name (ARN) for the IAM Identity Center instance.
     */
    readonly iamIdentityCenterInstanceArn: pulumi.Output<string | undefined>;
    /**
     * Last name of the author of the Amazon QuickSight account to use for future communications. This field is required if `ENTERPPRISE_AND_Q` is the selected edition of the new Amazon QuickSight account.
     */
    readonly lastName: pulumi.Output<string | undefined>;
    /**
     * Email address that you want Amazon QuickSight to send notifications to regarding your Amazon QuickSight account or Amazon QuickSight subscription.
     *
     * The following arguments are optional:
     */
    readonly notificationEmail: pulumi.Output<string>;
    /**
     * Reader group associated with your Active Directory or IAM Identity Center account.
     */
    readonly readerGroups: pulumi.Output<string[] | undefined>;
    /**
     * Reader PRO group associated with your Active Directory or IAM Identity Center account.
     */
    readonly readerProGroups: pulumi.Output<string[] | undefined>;
    /**
     * Realm of the Active Directory that is associated with your Amazon QuickSight account.
     */
    readonly realm: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a AccountSubscription resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AccountSubscriptionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AccountSubscription resources.
 */
export interface AccountSubscriptionState {
    /**
     * Name of your Amazon QuickSight account. This name is unique over all of AWS, and it appears only when users sign in.
     */
    accountName?: pulumi.Input<string>;
    /**
     * Status of the Amazon QuickSight account's subscription.
     */
    accountSubscriptionStatus?: pulumi.Input<string>;
    /**
     * Name of your Active Directory. This field is required if `ACTIVE_DIRECTORY` is the selected authentication method of the new Amazon QuickSight account.
     */
    activeDirectoryName?: pulumi.Input<string>;
    /**
     * Admin group associated with your Active Directory or IAM Identity Center account. This field is required if `ACTIVE_DIRECTORY` or `IAM_IDENTITY_CENTER` is the selected authentication method of the new Amazon QuickSight account.
     */
    adminGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Admin PRO group associated with your Active Directory or IAM Identity Center account.
     */
    adminProGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Method that you want to use to authenticate your Amazon QuickSight account. Currently, the valid values for this parameter are `IAM_AND_QUICKSIGHT`, `IAM_ONLY`, `IAM_IDENTITY_CENTER`, and `ACTIVE_DIRECTORY`.
     */
    authenticationMethod?: pulumi.Input<string>;
    /**
     * Author group associated with your Active Directory or IAM Identity Center account.
     */
    authorGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Author PRO group associated with your Active Directory or IAM Identity Center account.
     */
    authorProGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * AWS account ID. Defaults to automatically determined account ID of the Pulumi AWS provider.
     */
    awsAccountId?: pulumi.Input<string>;
    /**
     * A 10-digit phone number for the author of the Amazon QuickSight account to use for future communications. This field is required if `ENTERPPRISE_AND_Q` is the selected edition of the new Amazon QuickSight account.
     */
    contactNumber?: pulumi.Input<string>;
    /**
     * Active Directory ID that is associated with your Amazon QuickSight account.
     */
    directoryId?: pulumi.Input<string>;
    /**
     * Edition of Amazon QuickSight that you want your account to have. Currently, you can choose from `STANDARD`, `ENTERPRISE` or `ENTERPRISE_AND_Q`.
     */
    edition?: pulumi.Input<string>;
    /**
     * Email address of the author of the Amazon QuickSight account to use for future communications. This field is required if `ENTERPPRISE_AND_Q` is the selected edition of the new Amazon QuickSight account.
     */
    emailAddress?: pulumi.Input<string>;
    /**
     * First name of the author of the Amazon QuickSight account to use for future communications. This field is required if `ENTERPPRISE_AND_Q` is the selected edition of the new Amazon QuickSight account.
     */
    firstName?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for the IAM Identity Center instance.
     */
    iamIdentityCenterInstanceArn?: pulumi.Input<string>;
    /**
     * Last name of the author of the Amazon QuickSight account to use for future communications. This field is required if `ENTERPPRISE_AND_Q` is the selected edition of the new Amazon QuickSight account.
     */
    lastName?: pulumi.Input<string>;
    /**
     * Email address that you want Amazon QuickSight to send notifications to regarding your Amazon QuickSight account or Amazon QuickSight subscription.
     *
     * The following arguments are optional:
     */
    notificationEmail?: pulumi.Input<string>;
    /**
     * Reader group associated with your Active Directory or IAM Identity Center account.
     */
    readerGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Reader PRO group associated with your Active Directory or IAM Identity Center account.
     */
    readerProGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Realm of the Active Directory that is associated with your Amazon QuickSight account.
     */
    realm?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AccountSubscription resource.
 */
export interface AccountSubscriptionArgs {
    /**
     * Name of your Amazon QuickSight account. This name is unique over all of AWS, and it appears only when users sign in.
     */
    accountName: pulumi.Input<string>;
    /**
     * Name of your Active Directory. This field is required if `ACTIVE_DIRECTORY` is the selected authentication method of the new Amazon QuickSight account.
     */
    activeDirectoryName?: pulumi.Input<string>;
    /**
     * Admin group associated with your Active Directory or IAM Identity Center account. This field is required if `ACTIVE_DIRECTORY` or `IAM_IDENTITY_CENTER` is the selected authentication method of the new Amazon QuickSight account.
     */
    adminGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Admin PRO group associated with your Active Directory or IAM Identity Center account.
     */
    adminProGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Method that you want to use to authenticate your Amazon QuickSight account. Currently, the valid values for this parameter are `IAM_AND_QUICKSIGHT`, `IAM_ONLY`, `IAM_IDENTITY_CENTER`, and `ACTIVE_DIRECTORY`.
     */
    authenticationMethod: pulumi.Input<string>;
    /**
     * Author group associated with your Active Directory or IAM Identity Center account.
     */
    authorGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Author PRO group associated with your Active Directory or IAM Identity Center account.
     */
    authorProGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * AWS account ID. Defaults to automatically determined account ID of the Pulumi AWS provider.
     */
    awsAccountId?: pulumi.Input<string>;
    /**
     * A 10-digit phone number for the author of the Amazon QuickSight account to use for future communications. This field is required if `ENTERPPRISE_AND_Q` is the selected edition of the new Amazon QuickSight account.
     */
    contactNumber?: pulumi.Input<string>;
    /**
     * Active Directory ID that is associated with your Amazon QuickSight account.
     */
    directoryId?: pulumi.Input<string>;
    /**
     * Edition of Amazon QuickSight that you want your account to have. Currently, you can choose from `STANDARD`, `ENTERPRISE` or `ENTERPRISE_AND_Q`.
     */
    edition: pulumi.Input<string>;
    /**
     * Email address of the author of the Amazon QuickSight account to use for future communications. This field is required if `ENTERPPRISE_AND_Q` is the selected edition of the new Amazon QuickSight account.
     */
    emailAddress?: pulumi.Input<string>;
    /**
     * First name of the author of the Amazon QuickSight account to use for future communications. This field is required if `ENTERPPRISE_AND_Q` is the selected edition of the new Amazon QuickSight account.
     */
    firstName?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) for the IAM Identity Center instance.
     */
    iamIdentityCenterInstanceArn?: pulumi.Input<string>;
    /**
     * Last name of the author of the Amazon QuickSight account to use for future communications. This field is required if `ENTERPPRISE_AND_Q` is the selected edition of the new Amazon QuickSight account.
     */
    lastName?: pulumi.Input<string>;
    /**
     * Email address that you want Amazon QuickSight to send notifications to regarding your Amazon QuickSight account or Amazon QuickSight subscription.
     *
     * The following arguments are optional:
     */
    notificationEmail: pulumi.Input<string>;
    /**
     * Reader group associated with your Active Directory or IAM Identity Center account.
     */
    readerGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Reader PRO group associated with your Active Directory or IAM Identity Center account.
     */
    readerProGroups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Realm of the Active Directory that is associated with your Amazon QuickSight account.
     */
    realm?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
