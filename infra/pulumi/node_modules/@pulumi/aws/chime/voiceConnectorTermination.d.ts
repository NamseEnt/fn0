import * as pulumi from "@pulumi/pulumi";
/**
 * Enable Termination settings to control outbound calling from your SIP infrastructure.
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
 * const defaultVoiceConnectorTermination = new aws.chime.VoiceConnectorTermination("default", {
 *     disabled: false,
 *     cpsLimit: 1,
 *     cidrAllowLists: ["50.35.78.96/31"],
 *     callingRegions: [
 *         "US",
 *         "CA",
 *     ],
 *     voiceConnectorId: _default.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Chime Voice Connector Termination using the `voice_connector_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:chime/voiceConnectorTermination:VoiceConnectorTermination default abcdef1ghij2klmno3pqr4
 * ```
 */
export declare class VoiceConnectorTermination extends pulumi.CustomResource {
    /**
     * Get an existing VoiceConnectorTermination resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VoiceConnectorTerminationState, opts?: pulumi.CustomResourceOptions): VoiceConnectorTermination;
    /**
     * Returns true if the given object is an instance of VoiceConnectorTermination.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VoiceConnectorTermination;
    /**
     * The countries to which calls are allowed, in ISO 3166-1 alpha-2 format.
     */
    readonly callingRegions: pulumi.Output<string[]>;
    /**
     * The IP addresses allowed to make calls, in CIDR format.
     */
    readonly cidrAllowLists: pulumi.Output<string[]>;
    /**
     * The limit on calls per second. Max value based on account service quota. Default value of `1`.
     */
    readonly cpsLimit: pulumi.Output<number | undefined>;
    /**
     * The default caller ID phone number.
     */
    readonly defaultPhoneNumber: pulumi.Output<string | undefined>;
    /**
     * When termination settings are disabled, outbound calls can not be made.
     */
    readonly disabled: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Chime Voice Connector ID.
     */
    readonly voiceConnectorId: pulumi.Output<string>;
    /**
     * Create a VoiceConnectorTermination resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VoiceConnectorTerminationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VoiceConnectorTermination resources.
 */
export interface VoiceConnectorTerminationState {
    /**
     * The countries to which calls are allowed, in ISO 3166-1 alpha-2 format.
     */
    callingRegions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The IP addresses allowed to make calls, in CIDR format.
     */
    cidrAllowLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The limit on calls per second. Max value based on account service quota. Default value of `1`.
     */
    cpsLimit?: pulumi.Input<number>;
    /**
     * The default caller ID phone number.
     */
    defaultPhoneNumber?: pulumi.Input<string>;
    /**
     * When termination settings are disabled, outbound calls can not be made.
     */
    disabled?: pulumi.Input<boolean>;
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
 * The set of arguments for constructing a VoiceConnectorTermination resource.
 */
export interface VoiceConnectorTerminationArgs {
    /**
     * The countries to which calls are allowed, in ISO 3166-1 alpha-2 format.
     */
    callingRegions: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The IP addresses allowed to make calls, in CIDR format.
     */
    cidrAllowLists: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The limit on calls per second. Max value based on account service quota. Default value of `1`.
     */
    cpsLimit?: pulumi.Input<number>;
    /**
     * The default caller ID phone number.
     */
    defaultPhoneNumber?: pulumi.Input<string>;
    /**
     * When termination settings are disabled, outbound calls can not be made.
     */
    disabled?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Chime Voice Connector ID.
     */
    voiceConnectorId: pulumi.Input<string>;
}
