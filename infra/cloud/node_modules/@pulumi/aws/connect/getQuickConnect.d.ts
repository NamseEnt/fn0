import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides details about a specific Amazon Connect Quick Connect.
 *
 * ## Example Usage
 *
 * By `name`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.connect.getQuickConnect({
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     name: "Example",
 * });
 * ```
 *
 * By `quickConnectId`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.connect.getQuickConnect({
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     quickConnectId: "cccccccc-bbbb-cccc-dddd-111111111111",
 * });
 * ```
 */
export declare function getQuickConnect(args: GetQuickConnectArgs, opts?: pulumi.InvokeOptions): Promise<GetQuickConnectResult>;
/**
 * A collection of arguments for invoking getQuickConnect.
 */
export interface GetQuickConnectArgs {
    /**
     * Reference to the hosting Amazon Connect Instance
     */
    instanceId: string;
    /**
     * Returns information on a specific Quick Connect by name
     *
     * > **NOTE:** `instanceId` and one of either `name` or `quickConnectId` is required.
     */
    name?: string;
    /**
     * Returns information on a specific Quick Connect by Quick Connect id
     */
    quickConnectId?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags to assign to the Quick Connect.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getQuickConnect.
 */
export interface GetQuickConnectResult {
    /**
     * ARN of the Quick Connect.
     */
    readonly arn: string;
    /**
     * Description of the Quick Connect.
     */
    readonly description: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly instanceId: string;
    readonly name: string;
    /**
     * A block that defines the configuration information for the Quick Connect: `quickConnectType` and one of `phoneConfig`, `queueConfig`, `userConfig` . The Quick Connect Config block is documented below.
     */
    readonly quickConnectConfigs: outputs.connect.GetQuickConnectQuickConnectConfig[];
    /**
     * Identifier for the Quick Connect.
     */
    readonly quickConnectId: string;
    readonly region: string;
    /**
     * Map of tags to assign to the Quick Connect.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Provides details about a specific Amazon Connect Quick Connect.
 *
 * ## Example Usage
 *
 * By `name`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.connect.getQuickConnect({
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     name: "Example",
 * });
 * ```
 *
 * By `quickConnectId`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.connect.getQuickConnect({
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     quickConnectId: "cccccccc-bbbb-cccc-dddd-111111111111",
 * });
 * ```
 */
export declare function getQuickConnectOutput(args: GetQuickConnectOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetQuickConnectResult>;
/**
 * A collection of arguments for invoking getQuickConnect.
 */
export interface GetQuickConnectOutputArgs {
    /**
     * Reference to the hosting Amazon Connect Instance
     */
    instanceId: pulumi.Input<string>;
    /**
     * Returns information on a specific Quick Connect by name
     *
     * > **NOTE:** `instanceId` and one of either `name` or `quickConnectId` is required.
     */
    name?: pulumi.Input<string>;
    /**
     * Returns information on a specific Quick Connect by Quick Connect id
     */
    quickConnectId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the Quick Connect.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
