import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an GameLift Game Session Queue resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.gamelift.GameSessionQueue("test", {
 *     name: "example-session-queue",
 *     destinations: [
 *         usWest2Fleet.arn,
 *         euCentral1Fleet.arn,
 *     ],
 *     notificationTarget: gameSessionQueueNotifications.arn,
 *     playerLatencyPolicies: [
 *         {
 *             maximumIndividualPlayerLatencyMilliseconds: 100,
 *             policyDurationSeconds: 5,
 *         },
 *         {
 *             maximumIndividualPlayerLatencyMilliseconds: 200,
 *         },
 *     ],
 *     timeoutInSeconds: 60,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import GameLift Game Session Queues using their `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:gamelift/gameSessionQueue:GameSessionQueue example example
 * ```
 */
export declare class GameSessionQueue extends pulumi.CustomResource {
    /**
     * Get an existing GameSessionQueue resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GameSessionQueueState, opts?: pulumi.CustomResourceOptions): GameSessionQueue;
    /**
     * Returns true if the given object is an instance of GameSessionQueue.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is GameSessionQueue;
    /**
     * Game Session Queue ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Information to be added to all events that are related to this game session queue.
     */
    readonly customEventData: pulumi.Output<string | undefined>;
    /**
     * List of fleet/alias ARNs used by session queue for placing game sessions.
     */
    readonly destinations: pulumi.Output<string[] | undefined>;
    /**
     * Name of the session queue.
     */
    readonly name: pulumi.Output<string>;
    /**
     * An SNS topic ARN that is set up to receive game session placement notifications.
     */
    readonly notificationTarget: pulumi.Output<string | undefined>;
    /**
     * One or more policies used to choose fleet based on player latency. See below.
     */
    readonly playerLatencyPolicies: pulumi.Output<outputs.gamelift.GameSessionQueuePlayerLatencyPolicy[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Maximum time a game session request can remain in the queue.
     */
    readonly timeoutInSeconds: pulumi.Output<number | undefined>;
    /**
     * Create a GameSessionQueue resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: GameSessionQueueArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering GameSessionQueue resources.
 */
export interface GameSessionQueueState {
    /**
     * Game Session Queue ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * Information to be added to all events that are related to this game session queue.
     */
    customEventData?: pulumi.Input<string>;
    /**
     * List of fleet/alias ARNs used by session queue for placing game sessions.
     */
    destinations?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Name of the session queue.
     */
    name?: pulumi.Input<string>;
    /**
     * An SNS topic ARN that is set up to receive game session placement notifications.
     */
    notificationTarget?: pulumi.Input<string>;
    /**
     * One or more policies used to choose fleet based on player latency. See below.
     */
    playerLatencyPolicies?: pulumi.Input<pulumi.Input<inputs.gamelift.GameSessionQueuePlayerLatencyPolicy>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Maximum time a game session request can remain in the queue.
     */
    timeoutInSeconds?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a GameSessionQueue resource.
 */
export interface GameSessionQueueArgs {
    /**
     * Information to be added to all events that are related to this game session queue.
     */
    customEventData?: pulumi.Input<string>;
    /**
     * List of fleet/alias ARNs used by session queue for placing game sessions.
     */
    destinations?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Name of the session queue.
     */
    name?: pulumi.Input<string>;
    /**
     * An SNS topic ARN that is set up to receive game session placement notifications.
     */
    notificationTarget?: pulumi.Input<string>;
    /**
     * One or more policies used to choose fleet based on player latency. See below.
     */
    playerLatencyPolicies?: pulumi.Input<pulumi.Input<inputs.gamelift.GameSessionQueuePlayerLatencyPolicy>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Maximum time a game session request can remain in the queue.
     */
    timeoutInSeconds?: pulumi.Input<number>;
}
