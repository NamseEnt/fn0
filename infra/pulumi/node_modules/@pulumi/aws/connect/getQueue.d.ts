import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides details about a specific Amazon Connect Queue.
 *
 * ## Example Usage
 *
 * By `name`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.connect.getQueue({
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     name: "Example",
 * });
 * ```
 *
 * By `queueId`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.connect.getQueue({
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     queueId: "cccccccc-bbbb-cccc-dddd-111111111111",
 * });
 * ```
 */
export declare function getQueue(args: GetQueueArgs, opts?: pulumi.InvokeOptions): Promise<GetQueueResult>;
/**
 * A collection of arguments for invoking getQueue.
 */
export interface GetQueueArgs {
    /**
     * Reference to the hosting Amazon Connect Instance
     */
    instanceId: string;
    /**
     * Returns information on a specific Queue by name
     *
     * > **NOTE:** `instanceId` and one of either `name` or `queueId` is required.
     */
    name?: string;
    /**
     * Returns information on a specific Queue by Queue id
     */
    queueId?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags assigned to the Queue.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getQueue.
 */
export interface GetQueueResult {
    /**
     * ARN of the Queue.
     */
    readonly arn: string;
    /**
     * Description of the Queue.
     */
    readonly description: string;
    /**
     * Specifies the identifier of the Hours of Operation.
     */
    readonly hoursOfOperationId: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly instanceId: string;
    /**
     * Maximum number of contacts that can be in the queue before it is considered full. Minimum value of 0.
     */
    readonly maxContacts: number;
    readonly name: string;
    /**
     * A block that defines the outbound caller ID name, number, and outbound whisper flow. The Outbound Caller Config block is documented below.
     */
    readonly outboundCallerConfigs: outputs.connect.GetQueueOutboundCallerConfig[];
    /**
     * Identifier for the Queue.
     */
    readonly queueId: string;
    readonly region: string;
    /**
     * Description of the Queue. Values are `ENABLED` or `DISABLED`.
     */
    readonly status: string;
    /**
     * Map of tags assigned to the Queue.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Provides details about a specific Amazon Connect Queue.
 *
 * ## Example Usage
 *
 * By `name`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.connect.getQueue({
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     name: "Example",
 * });
 * ```
 *
 * By `queueId`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.connect.getQueue({
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     queueId: "cccccccc-bbbb-cccc-dddd-111111111111",
 * });
 * ```
 */
export declare function getQueueOutput(args: GetQueueOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetQueueResult>;
/**
 * A collection of arguments for invoking getQueue.
 */
export interface GetQueueOutputArgs {
    /**
     * Reference to the hosting Amazon Connect Instance
     */
    instanceId: pulumi.Input<string>;
    /**
     * Returns information on a specific Queue by name
     *
     * > **NOTE:** `instanceId` and one of either `name` or `queueId` is required.
     */
    name?: pulumi.Input<string>;
    /**
     * Returns information on a specific Queue by Queue id
     */
    queueId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the Queue.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
