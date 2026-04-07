import * as pulumi from "@pulumi/pulumi";
/**
 * Adds a logging configuration for the specified Amazon Chime Voice Connector. The logging configuration specifies whether SIP message logs are enabled for sending to Amazon CloudWatch Logs.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.chime.VoiceConnector("default", {
 *     name: "vc-name-test",
 *     requireEncryption: true,
 * });
 * const defaultVoiceConnectorLogging = new aws.chime.VoiceConnectorLogging("default", {
 *     enableSipLogs: true,
 *     enableMediaMetricLogs: true,
 *     voiceConnectorId: _default.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Chime Voice Connector Logging using the `voice_connector_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:chime/voiceConnectorLogging:VoiceConnectorLogging default abcdef1ghij2klmno3pqr4
 * ```
 */
export declare class VoiceConnectorLogging extends pulumi.CustomResource {
    /**
     * Get an existing VoiceConnectorLogging resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VoiceConnectorLoggingState, opts?: pulumi.CustomResourceOptions): VoiceConnectorLogging;
    /**
     * Returns true if the given object is an instance of VoiceConnectorLogging.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VoiceConnectorLogging;
    /**
     * When true, enables logging of detailed media metrics for Voice Connectors to Amazon CloudWatch logs.
     */
    readonly enableMediaMetricLogs: pulumi.Output<boolean | undefined>;
    /**
     * When true, enables SIP message logs for sending to Amazon CloudWatch Logs.
     */
    readonly enableSipLogs: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Chime Voice Connector ID.
     */
    readonly voiceConnectorId: pulumi.Output<string>;
    /**
     * Create a VoiceConnectorLogging resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VoiceConnectorLoggingArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VoiceConnectorLogging resources.
 */
export interface VoiceConnectorLoggingState {
    /**
     * When true, enables logging of detailed media metrics for Voice Connectors to Amazon CloudWatch logs.
     */
    enableMediaMetricLogs?: pulumi.Input<boolean>;
    /**
     * When true, enables SIP message logs for sending to Amazon CloudWatch Logs.
     */
    enableSipLogs?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Chime Voice Connector ID.
     */
    voiceConnectorId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VoiceConnectorLogging resource.
 */
export interface VoiceConnectorLoggingArgs {
    /**
     * When true, enables logging of detailed media metrics for Voice Connectors to Amazon CloudWatch logs.
     */
    enableMediaMetricLogs?: pulumi.Input<boolean>;
    /**
     * When true, enables SIP message logs for sending to Amazon CloudWatch Logs.
     */
    enableSipLogs?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Chime Voice Connector ID.
     */
    voiceConnectorId: pulumi.Input<string>;
}
