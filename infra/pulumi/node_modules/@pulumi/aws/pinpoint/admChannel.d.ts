import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Pinpoint ADM (Amazon Device Messaging) Channel resource.
 *
 * > **Note:** All arguments including the Client ID and Client Secret will be stored in the raw state as plain-text.
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const app = new aws.pinpoint.App("app", {});
 * const channel = new aws.pinpoint.AdmChannel("channel", {
 *     applicationId: app.applicationId,
 *     clientId: "",
 *     clientSecret: "",
 *     enabled: true,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Pinpoint ADM Channel using the `application-id`. For example:
 *
 * ```sh
 * $ pulumi import aws:pinpoint/admChannel:AdmChannel channel application-id
 * ```
 */
export declare class AdmChannel extends pulumi.CustomResource {
    /**
     * Get an existing AdmChannel resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AdmChannelState, opts?: pulumi.CustomResourceOptions): AdmChannel;
    /**
     * Returns true if the given object is an instance of AdmChannel.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AdmChannel;
    /**
     * The application ID.
     */
    readonly applicationId: pulumi.Output<string>;
    /**
     * Client ID (part of OAuth Credentials) obtained via Amazon Developer Account.
     */
    readonly clientId: pulumi.Output<string>;
    /**
     * Client Secret (part of OAuth Credentials) obtained via Amazon Developer Account.
     */
    readonly clientSecret: pulumi.Output<string>;
    /**
     * Specifies whether to enable the channel. Defaults to `true`.
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a AdmChannel resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AdmChannelArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AdmChannel resources.
 */
export interface AdmChannelState {
    /**
     * The application ID.
     */
    applicationId?: pulumi.Input<string>;
    /**
     * Client ID (part of OAuth Credentials) obtained via Amazon Developer Account.
     */
    clientId?: pulumi.Input<string>;
    /**
     * Client Secret (part of OAuth Credentials) obtained via Amazon Developer Account.
     */
    clientSecret?: pulumi.Input<string>;
    /**
     * Specifies whether to enable the channel. Defaults to `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AdmChannel resource.
 */
export interface AdmChannelArgs {
    /**
     * The application ID.
     */
    applicationId: pulumi.Input<string>;
    /**
     * Client ID (part of OAuth Credentials) obtained via Amazon Developer Account.
     */
    clientId: pulumi.Input<string>;
    /**
     * Client Secret (part of OAuth Credentials) obtained via Amazon Developer Account.
     */
    clientSecret: pulumi.Input<string>;
    /**
     * Specifies whether to enable the channel. Defaults to `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
