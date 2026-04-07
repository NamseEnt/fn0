import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides details about a specific Amazon Connect Instance Storage Config.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.connect.getInstanceStorageConfig({
 *     associationId: "1234567891234567890122345678912345678901223456789123456789012234",
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     resourceType: "CONTACT_TRACE_RECORDS",
 * });
 * ```
 */
export declare function getInstanceStorageConfig(args: GetInstanceStorageConfigArgs, opts?: pulumi.InvokeOptions): Promise<GetInstanceStorageConfigResult>;
/**
 * A collection of arguments for invoking getInstanceStorageConfig.
 */
export interface GetInstanceStorageConfigArgs {
    /**
     * The existing association identifier that uniquely identifies the resource type and storage config for the given instance ID.
     */
    associationId: string;
    /**
     * Reference to the hosting Amazon Connect Instance
     */
    instanceId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * A valid resource type. Valid Values: `AGENT_EVENTS` | `ATTACHMENTS` | `CALL_RECORDINGS` | `CHAT_TRANSCRIPTS` | `CONTACT_EVALUATIONS` | `CONTACT_TRACE_RECORDS` | `MEDIA_STREAMS` | `REAL_TIME_CONTACT_ANALYSIS_SEGMENTS` | `SCHEDULED_REPORTS` |  `SCREEN_RECORDINGS`.
     */
    resourceType: string;
}
/**
 * A collection of values returned by getInstanceStorageConfig.
 */
export interface GetInstanceStorageConfigResult {
    readonly associationId: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly instanceId: string;
    readonly region: string;
    readonly resourceType: string;
    /**
     * Specifies the storage configuration options for the Connect Instance. Documented below.
     */
    readonly storageConfigs: outputs.connect.GetInstanceStorageConfigStorageConfig[];
}
/**
 * Provides details about a specific Amazon Connect Instance Storage Config.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.connect.getInstanceStorageConfig({
 *     associationId: "1234567891234567890122345678912345678901223456789123456789012234",
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     resourceType: "CONTACT_TRACE_RECORDS",
 * });
 * ```
 */
export declare function getInstanceStorageConfigOutput(args: GetInstanceStorageConfigOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetInstanceStorageConfigResult>;
/**
 * A collection of arguments for invoking getInstanceStorageConfig.
 */
export interface GetInstanceStorageConfigOutputArgs {
    /**
     * The existing association identifier that uniquely identifies the resource type and storage config for the given instance ID.
     */
    associationId: pulumi.Input<string>;
    /**
     * Reference to the hosting Amazon Connect Instance
     */
    instanceId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A valid resource type. Valid Values: `AGENT_EVENTS` | `ATTACHMENTS` | `CALL_RECORDINGS` | `CHAT_TRANSCRIPTS` | `CONTACT_EVALUATIONS` | `CONTACT_TRACE_RECORDS` | `MEDIA_STREAMS` | `REAL_TIME_CONTACT_ANALYSIS_SEGMENTS` | `SCHEDULED_REPORTS` |  `SCREEN_RECORDINGS`.
     */
    resourceType: pulumi.Input<string>;
}
