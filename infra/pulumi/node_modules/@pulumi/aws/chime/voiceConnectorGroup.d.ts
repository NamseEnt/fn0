import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Creates an Amazon Chime Voice Connector group under the administrator's AWS account. You can associate Amazon Chime Voice Connectors with the Amazon Chime Voice Connector group by including VoiceConnectorItems in the request.
 *
 * You can include Amazon Chime Voice Connectors from different AWS Regions in your group. This creates a fault tolerant mechanism for fallback in case of availability events.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const vc1 = new aws.chime.VoiceConnector("vc1", {
 *     name: "connector-test-1",
 *     requireEncryption: true,
 *     awsRegion: "us-east-1",
 * });
 * const vc2 = new aws.chime.VoiceConnector("vc2", {
 *     name: "connector-test-2",
 *     requireEncryption: true,
 *     awsRegion: "us-west-2",
 * });
 * const group = new aws.chime.VoiceConnectorGroup("group", {
 *     name: "test-group",
 *     connectors: [
 *         {
 *             voiceConnectorId: vc1.id,
 *             priority: 1,
 *         },
 *         {
 *             voiceConnectorId: vc2.id,
 *             priority: 3,
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Configuration Recorder using the name. For example:
 *
 * ```sh
 * $ pulumi import aws:chime/voiceConnectorGroup:VoiceConnectorGroup default example
 * ```
 */
export declare class VoiceConnectorGroup extends pulumi.CustomResource {
    /**
     * Get an existing VoiceConnectorGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VoiceConnectorGroupState, opts?: pulumi.CustomResourceOptions): VoiceConnectorGroup;
    /**
     * Returns true if the given object is an instance of VoiceConnectorGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VoiceConnectorGroup;
    /**
     * The Amazon Chime Voice Connectors to route inbound calls to.
     */
    readonly connectors: pulumi.Output<outputs.chime.VoiceConnectorGroupConnector[] | undefined>;
    /**
     * The name of the Amazon Chime Voice Connector group.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a VoiceConnectorGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: VoiceConnectorGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VoiceConnectorGroup resources.
 */
export interface VoiceConnectorGroupState {
    /**
     * The Amazon Chime Voice Connectors to route inbound calls to.
     */
    connectors?: pulumi.Input<pulumi.Input<inputs.chime.VoiceConnectorGroupConnector>[]>;
    /**
     * The name of the Amazon Chime Voice Connector group.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VoiceConnectorGroup resource.
 */
export interface VoiceConnectorGroupArgs {
    /**
     * The Amazon Chime Voice Connectors to route inbound calls to.
     */
    connectors?: pulumi.Input<pulumi.Input<inputs.chime.VoiceConnectorGroupConnector>[]>;
    /**
     * The name of the Amazon Chime Voice Connector group.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
