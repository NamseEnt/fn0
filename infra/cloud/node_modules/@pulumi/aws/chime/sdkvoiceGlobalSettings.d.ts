import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing Amazon Chime SDK Voice Global Settings.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.chime.SdkvoiceGlobalSettings("example", {voiceConnector: {
 *     cdrBucket: "example-bucket-name",
 * }});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AWS Chime SDK Voice Global Settings using the `id` (AWS account ID). For example:
 *
 * ```sh
 * $ pulumi import aws:chime/sdkvoiceGlobalSettings:SdkvoiceGlobalSettings example 123456789012
 * ```
 */
export declare class SdkvoiceGlobalSettings extends pulumi.CustomResource {
    /**
     * Get an existing SdkvoiceGlobalSettings resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SdkvoiceGlobalSettingsState, opts?: pulumi.CustomResourceOptions): SdkvoiceGlobalSettings;
    /**
     * Returns true if the given object is an instance of SdkvoiceGlobalSettings.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SdkvoiceGlobalSettings;
    /**
     * The Voice Connector settings. See voice_connector.
     */
    readonly voiceConnector: pulumi.Output<outputs.chime.SdkvoiceGlobalSettingsVoiceConnector>;
    /**
     * Create a SdkvoiceGlobalSettings resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SdkvoiceGlobalSettingsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SdkvoiceGlobalSettings resources.
 */
export interface SdkvoiceGlobalSettingsState {
    /**
     * The Voice Connector settings. See voice_connector.
     */
    voiceConnector?: pulumi.Input<inputs.chime.SdkvoiceGlobalSettingsVoiceConnector>;
}
/**
 * The set of arguments for constructing a SdkvoiceGlobalSettings resource.
 */
export interface SdkvoiceGlobalSettingsArgs {
    /**
     * The Voice Connector settings. See voice_connector.
     */
    voiceConnector: pulumi.Input<inputs.chime.SdkvoiceGlobalSettingsVoiceConnector>;
}
