import * as pulumi from "@pulumi/pulumi";
/**
 * Enables you to connect your phone system to the telephone network at a substantial cost savings by using SIP trunking.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.chime.VoiceConnector("test", {
 *     name: "connector-test-1",
 *     requireEncryption: true,
 *     awsRegion: "us-east-1",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Configuration Recorder using the name. For example:
 *
 * ```sh
 * $ pulumi import aws:chime/voiceConnector:VoiceConnector test example
 * ```
 */
export declare class VoiceConnector extends pulumi.CustomResource {
    /**
     * Get an existing VoiceConnector resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VoiceConnectorState, opts?: pulumi.CustomResourceOptions): VoiceConnector;
    /**
     * Returns true if the given object is an instance of VoiceConnector.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VoiceConnector;
    /**
     * ARN (Amazon Resource Name) of the Amazon Chime Voice Connector.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The AWS Region in which the Amazon Chime Voice Connector is created. Default value: `us-east-1`
     */
    readonly awsRegion: pulumi.Output<string>;
    /**
     * The name of the Amazon Chime Voice Connector.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The outbound host name for the Amazon Chime Voice Connector.
     */
    readonly outboundHostName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * When enabled, requires encryption for the Amazon Chime Voice Connector.
     *
     * The following arguments are optional:
     */
    readonly requireEncryption: pulumi.Output<boolean>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a VoiceConnector resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VoiceConnectorArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VoiceConnector resources.
 */
export interface VoiceConnectorState {
    /**
     * ARN (Amazon Resource Name) of the Amazon Chime Voice Connector.
     */
    arn?: pulumi.Input<string>;
    /**
     * The AWS Region in which the Amazon Chime Voice Connector is created. Default value: `us-east-1`
     */
    awsRegion?: pulumi.Input<string>;
    /**
     * The name of the Amazon Chime Voice Connector.
     */
    name?: pulumi.Input<string>;
    /**
     * The outbound host name for the Amazon Chime Voice Connector.
     */
    outboundHostName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * When enabled, requires encryption for the Amazon Chime Voice Connector.
     *
     * The following arguments are optional:
     */
    requireEncryption?: pulumi.Input<boolean>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a VoiceConnector resource.
 */
export interface VoiceConnectorArgs {
    /**
     * The AWS Region in which the Amazon Chime Voice Connector is created. Default value: `us-east-1`
     */
    awsRegion?: pulumi.Input<string>;
    /**
     * The name of the Amazon Chime Voice Connector.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * When enabled, requires encryption for the Amazon Chime Voice Connector.
     *
     * The following arguments are optional:
     */
    requireEncryption: pulumi.Input<boolean>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
