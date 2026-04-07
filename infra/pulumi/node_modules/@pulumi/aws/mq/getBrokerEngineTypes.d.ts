import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides details about available MQ broker engine types. Use this data source to retrieve supported engine types and their versions for Amazon MQ brokers.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.mq.getBrokerEngineTypes({
 *     engineType: "ACTIVEMQ",
 * });
 * ```
 */
export declare function getBrokerEngineTypes(args?: GetBrokerEngineTypesArgs, opts?: pulumi.InvokeOptions): Promise<GetBrokerEngineTypesResult>;
/**
 * A collection of arguments for invoking getBrokerEngineTypes.
 */
export interface GetBrokerEngineTypesArgs {
    /**
     * MQ engine type to return version details for.
     */
    engineType?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getBrokerEngineTypes.
 */
export interface GetBrokerEngineTypesResult {
    /**
     * List of available engine types and versions. See Engine Types.
     */
    readonly brokerEngineTypes: outputs.mq.GetBrokerEngineTypesBrokerEngineType[];
    /**
     * Broker's engine type.
     */
    readonly engineType?: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Provides details about available MQ broker engine types. Use this data source to retrieve supported engine types and their versions for Amazon MQ brokers.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.mq.getBrokerEngineTypes({
 *     engineType: "ACTIVEMQ",
 * });
 * ```
 */
export declare function getBrokerEngineTypesOutput(args?: GetBrokerEngineTypesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetBrokerEngineTypesResult>;
/**
 * A collection of arguments for invoking getBrokerEngineTypes.
 */
export interface GetBrokerEngineTypesOutputArgs {
    /**
     * MQ engine type to return version details for.
     */
    engineType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
