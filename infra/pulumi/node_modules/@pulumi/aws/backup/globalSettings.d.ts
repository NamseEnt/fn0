import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AWS Backup Global Settings resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.backup.GlobalSettings("test", {globalSettings: {
 *     isCrossAccountBackupEnabled: "true",
 * }});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Backup Global Settings using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:backup/globalSettings:GlobalSettings example 123456789012
 * ```
 */
export declare class GlobalSettings extends pulumi.CustomResource {
    /**
     * Get an existing GlobalSettings resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GlobalSettingsState, opts?: pulumi.CustomResourceOptions): GlobalSettings;
    /**
     * Returns true if the given object is an instance of GlobalSettings.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is GlobalSettings;
    /**
     * A list of resources along with the opt-in preferences for the account.
     */
    readonly globalSettings: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a GlobalSettings resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: GlobalSettingsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering GlobalSettings resources.
 */
export interface GlobalSettingsState {
    /**
     * A list of resources along with the opt-in preferences for the account.
     */
    globalSettings?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a GlobalSettings resource.
 */
export interface GlobalSettingsArgs {
    /**
     * A list of resources along with the opt-in preferences for the account.
     */
    globalSettings: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
