import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Use this data source to get information on an existing AppIntegrations Event Integration.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.appintegrations.getEventIntegration({
 *     name: "example",
 * });
 * ```
 */
export declare function getEventIntegration(args: GetEventIntegrationArgs, opts?: pulumi.InvokeOptions): Promise<GetEventIntegrationResult>;
/**
 * A collection of arguments for invoking getEventIntegration.
 */
export interface GetEventIntegrationArgs {
    /**
     * The AppIntegrations Event Integration name.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Metadata that you can assign to help organize the report plans you create.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getEventIntegration.
 */
export interface GetEventIntegrationResult {
    /**
     * The ARN of the AppIntegrations Event Integration.
     */
    readonly arn: string;
    /**
     * The description of the Event Integration.
     */
    readonly description: string;
    /**
     * A block that defines the configuration information for the event filter. The Event Filter block is documented below.
     */
    readonly eventFilters: outputs.appintegrations.GetEventIntegrationEventFilter[];
    /**
     * The EventBridge bus.
     */
    readonly eventbridgeBus: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
    readonly region: string;
    /**
     * Metadata that you can assign to help organize the report plans you create.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Use this data source to get information on an existing AppIntegrations Event Integration.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.appintegrations.getEventIntegration({
 *     name: "example",
 * });
 * ```
 */
export declare function getEventIntegrationOutput(args: GetEventIntegrationOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetEventIntegrationResult>;
/**
 * A collection of arguments for invoking getEventIntegration.
 */
export interface GetEventIntegrationOutputArgs {
    /**
     * The AppIntegrations Event Integration name.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Metadata that you can assign to help organize the report plans you create.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
