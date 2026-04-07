import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides access to an AppConfig Environment.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.appconfig.getEnvironment({
 *     applicationId: "b5d5gpj",
 *     environmentId: "qrbb1c1",
 * });
 * ```
 */
export declare function getEnvironment(args: GetEnvironmentArgs, opts?: pulumi.InvokeOptions): Promise<GetEnvironmentResult>;
/**
 * A collection of arguments for invoking getEnvironment.
 */
export interface GetEnvironmentArgs {
    /**
     * ID of the AppConfig Application to which this Environment belongs.
     */
    applicationId: string;
    /**
     * ID of the AppConfig Environment.
     */
    environmentId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags for the resource.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getEnvironment.
 */
export interface GetEnvironmentResult {
    readonly applicationId: string;
    /**
     * ARN of the environment.
     */
    readonly arn: string;
    /**
     * Name of the environment.
     */
    readonly description: string;
    readonly environmentId: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Set of Amazon CloudWatch alarms to monitor during the deployment process.
     */
    readonly monitors: outputs.appconfig.GetEnvironmentMonitor[];
    /**
     * Name of the environment.
     */
    readonly name: string;
    readonly region: string;
    /**
     * State of the environment. Possible values are `READY_FOR_DEPLOYMENT`, `DEPLOYING`, `ROLLING_BACK`
     * or `ROLLED_BACK`.
     */
    readonly state: string;
    /**
     * Map of tags for the resource.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Provides access to an AppConfig Environment.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.appconfig.getEnvironment({
 *     applicationId: "b5d5gpj",
 *     environmentId: "qrbb1c1",
 * });
 * ```
 */
export declare function getEnvironmentOutput(args: GetEnvironmentOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetEnvironmentResult>;
/**
 * A collection of arguments for invoking getEnvironment.
 */
export interface GetEnvironmentOutputArgs {
    /**
     * ID of the AppConfig Application to which this Environment belongs.
     */
    applicationId: pulumi.Input<string>;
    /**
     * ID of the AppConfig Environment.
     */
    environmentId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags for the resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
