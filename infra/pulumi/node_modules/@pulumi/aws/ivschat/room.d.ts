import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS IVS (Interactive Video) Chat Room.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ivschat.Room("example", {name: "tf-room"});
 * ```
 */
export declare class Room extends pulumi.CustomResource {
    /**
     * Get an existing Room resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RoomState, opts?: pulumi.CustomResourceOptions): Room;
    /**
     * Returns true if the given object is an instance of Room.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Room;
    /**
     * ARN of the Room.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * List of Logging Configuration
     * ARNs to attach to the room.
     */
    readonly loggingConfigurationIdentifiers: pulumi.Output<string[] | undefined>;
    /**
     * Maximum number of characters in a single
     * message. Messages are expected to be UTF-8 encoded and this limit applies
     * specifically to rune/code-point count, not number of bytes.
     */
    readonly maximumMessageLength: pulumi.Output<number>;
    /**
     * Maximum number of messages per
     * second that can be sent to the room (by all clients).
     */
    readonly maximumMessageRatePerSecond: pulumi.Output<number>;
    /**
     * Configuration information for optional
     * review of messages.
     */
    readonly messageReviewHandler: pulumi.Output<outputs.ivschat.RoomMessageReviewHandler | undefined>;
    /**
     * Room name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Room resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: RoomArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Room resources.
 */
export interface RoomState {
    /**
     * ARN of the Room.
     */
    arn?: pulumi.Input<string>;
    /**
     * List of Logging Configuration
     * ARNs to attach to the room.
     */
    loggingConfigurationIdentifiers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Maximum number of characters in a single
     * message. Messages are expected to be UTF-8 encoded and this limit applies
     * specifically to rune/code-point count, not number of bytes.
     */
    maximumMessageLength?: pulumi.Input<number>;
    /**
     * Maximum number of messages per
     * second that can be sent to the room (by all clients).
     */
    maximumMessageRatePerSecond?: pulumi.Input<number>;
    /**
     * Configuration information for optional
     * review of messages.
     */
    messageReviewHandler?: pulumi.Input<inputs.ivschat.RoomMessageReviewHandler>;
    /**
     * Room name.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Room resource.
 */
export interface RoomArgs {
    /**
     * List of Logging Configuration
     * ARNs to attach to the room.
     */
    loggingConfigurationIdentifiers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Maximum number of characters in a single
     * message. Messages are expected to be UTF-8 encoded and this limit applies
     * specifically to rune/code-point count, not number of bytes.
     */
    maximumMessageLength?: pulumi.Input<number>;
    /**
     * Maximum number of messages per
     * second that can be sent to the room (by all clients).
     */
    maximumMessageRatePerSecond?: pulumi.Input<number>;
    /**
     * Configuration information for optional
     * review of messages.
     */
    messageReviewHandler?: pulumi.Input<inputs.ivschat.RoomMessageReviewHandler>;
    /**
     * Room name.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
