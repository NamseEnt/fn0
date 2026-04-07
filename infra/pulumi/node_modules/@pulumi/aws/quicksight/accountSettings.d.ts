import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS QuickSight Account Settings.
 *
 * > Deletion of this resource will not modify any settings, only remove the resource from state.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const subscription = new aws.quicksight.AccountSubscription("subscription", {
 *     accountName: "quicksight-terraform",
 *     authenticationMethod: "IAM_AND_QUICKSIGHT",
 *     edition: "ENTERPRISE",
 *     notificationEmail: "notification@email.com",
 * });
 * const example = new aws.quicksight.AccountSettings("example", {terminationProtectionEnabled: false}, {
 *     dependsOn: [subscription],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import QuickSight Account Settings using the AWS account ID. For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/accountSettings:AccountSettings example "012345678901"
 * ```
 */
export declare class AccountSettings extends pulumi.CustomResource {
    /**
     * Get an existing AccountSettings resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AccountSettingsState, opts?: pulumi.CustomResourceOptions): AccountSettings;
    /**
     * Returns true if the given object is an instance of AccountSettings.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AccountSettings;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * The default namespace for this Amazon Web Services account. Currently, the default is `default`.
     */
    readonly defaultNamespace: pulumi.Output<string>;
    /**
     * A boolean value that determines whether or not an Amazon QuickSight account can be deleted. If `true`, it does not allow the account to be deleted and results in an error message if a user tries to make a DeleteAccountSubscription request. If `false`, it will allow the account to be deleted.
     */
    readonly terminationProtectionEnabled: pulumi.Output<boolean>;
    readonly timeouts: pulumi.Output<outputs.quicksight.AccountSettingsTimeouts | undefined>;
    /**
     * Create a AccountSettings resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: AccountSettingsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AccountSettings resources.
 */
export interface AccountSettingsState {
    awsAccountId?: pulumi.Input<string>;
    /**
     * The default namespace for this Amazon Web Services account. Currently, the default is `default`.
     */
    defaultNamespace?: pulumi.Input<string>;
    /**
     * A boolean value that determines whether or not an Amazon QuickSight account can be deleted. If `true`, it does not allow the account to be deleted and results in an error message if a user tries to make a DeleteAccountSubscription request. If `false`, it will allow the account to be deleted.
     */
    terminationProtectionEnabled?: pulumi.Input<boolean>;
    timeouts?: pulumi.Input<inputs.quicksight.AccountSettingsTimeouts>;
}
/**
 * The set of arguments for constructing a AccountSettings resource.
 */
export interface AccountSettingsArgs {
    awsAccountId?: pulumi.Input<string>;
    /**
     * The default namespace for this Amazon Web Services account. Currently, the default is `default`.
     */
    defaultNamespace?: pulumi.Input<string>;
    /**
     * A boolean value that determines whether or not an Amazon QuickSight account can be deleted. If `true`, it does not allow the account to be deleted and results in an error message if a user tries to make a DeleteAccountSubscription request. If `false`, it will allow the account to be deleted.
     */
    terminationProtectionEnabled?: pulumi.Input<boolean>;
    timeouts?: pulumi.Input<inputs.quicksight.AccountSettingsTimeouts>;
}
