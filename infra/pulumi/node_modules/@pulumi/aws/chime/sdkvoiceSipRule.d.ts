import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * A SIP rule associates your SIP media application with a phone number or a Request URI hostname. You can associate a SIP rule with more than one SIP media application. Each application then runs only that rule.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.chime.SdkvoiceSipRule("example", {
 *     name: "example-sip-rule",
 *     triggerType: "RequestUriHostname",
 *     triggerValue: example_voice_connector.outboundHostName,
 *     targetApplications: [{
 *         priority: 1,
 *         sipMediaApplicationId: example_sma.id,
 *         awsRegion: "us-east-1",
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a ChimeSDKVoice SIP Rule using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:chime/sdkvoiceSipRule:SdkvoiceSipRule example abcdef123456
 * ```
 */
export declare class SdkvoiceSipRule extends pulumi.CustomResource {
    /**
     * Get an existing SdkvoiceSipRule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SdkvoiceSipRuleState, opts?: pulumi.CustomResourceOptions): SdkvoiceSipRule;
    /**
     * Returns true if the given object is an instance of SdkvoiceSipRule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SdkvoiceSipRule;
    /**
     * Enables or disables a rule. You must disable rules before you can delete them.
     */
    readonly disabled: pulumi.Output<boolean | undefined>;
    /**
     * The name of the SIP rule.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * List of SIP media applications with priority and AWS Region. Only one SIP application per AWS Region can be used. See `targetApplications`.
     */
    readonly targetApplications: pulumi.Output<outputs.chime.SdkvoiceSipRuleTargetApplication[]>;
    /**
     * The type of trigger assigned to the SIP rule in `triggerValue`. Valid values are `RequestUriHostname` or `ToPhoneNumber`.
     */
    readonly triggerType: pulumi.Output<string>;
    /**
     * If `triggerType` is `RequestUriHostname`, the value can be the outbound host name of an Amazon Chime Voice Connector. If `triggerType` is `ToPhoneNumber`, the value can be a customer-owned phone number in the E164 format. The Sip Media Application specified in the Sip Rule is triggered if the request URI in an incoming SIP request matches the `RequestUriHostname`, or if the "To" header in the incoming SIP request matches the `ToPhoneNumber` value.
     *
     * The following arguments are optional:
     */
    readonly triggerValue: pulumi.Output<string>;
    /**
     * Create a SdkvoiceSipRule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SdkvoiceSipRuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SdkvoiceSipRule resources.
 */
export interface SdkvoiceSipRuleState {
    /**
     * Enables or disables a rule. You must disable rules before you can delete them.
     */
    disabled?: pulumi.Input<boolean>;
    /**
     * The name of the SIP rule.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of SIP media applications with priority and AWS Region. Only one SIP application per AWS Region can be used. See `targetApplications`.
     */
    targetApplications?: pulumi.Input<pulumi.Input<inputs.chime.SdkvoiceSipRuleTargetApplication>[]>;
    /**
     * The type of trigger assigned to the SIP rule in `triggerValue`. Valid values are `RequestUriHostname` or `ToPhoneNumber`.
     */
    triggerType?: pulumi.Input<string>;
    /**
     * If `triggerType` is `RequestUriHostname`, the value can be the outbound host name of an Amazon Chime Voice Connector. If `triggerType` is `ToPhoneNumber`, the value can be a customer-owned phone number in the E164 format. The Sip Media Application specified in the Sip Rule is triggered if the request URI in an incoming SIP request matches the `RequestUriHostname`, or if the "To" header in the incoming SIP request matches the `ToPhoneNumber` value.
     *
     * The following arguments are optional:
     */
    triggerValue?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SdkvoiceSipRule resource.
 */
export interface SdkvoiceSipRuleArgs {
    /**
     * Enables or disables a rule. You must disable rules before you can delete them.
     */
    disabled?: pulumi.Input<boolean>;
    /**
     * The name of the SIP rule.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of SIP media applications with priority and AWS Region. Only one SIP application per AWS Region can be used. See `targetApplications`.
     */
    targetApplications: pulumi.Input<pulumi.Input<inputs.chime.SdkvoiceSipRuleTargetApplication>[]>;
    /**
     * The type of trigger assigned to the SIP rule in `triggerValue`. Valid values are `RequestUriHostname` or `ToPhoneNumber`.
     */
    triggerType: pulumi.Input<string>;
    /**
     * If `triggerType` is `RequestUriHostname`, the value can be the outbound host name of an Amazon Chime Voice Connector. If `triggerType` is `ToPhoneNumber`, the value can be a customer-owned phone number in the E164 format. The Sip Media Application specified in the Sip Rule is triggered if the request URI in an incoming SIP request matches the `RequestUriHostname`, or if the "To" header in the incoming SIP request matches the `ToPhoneNumber` value.
     *
     * The following arguments are optional:
     */
    triggerValue: pulumi.Input<string>;
}
