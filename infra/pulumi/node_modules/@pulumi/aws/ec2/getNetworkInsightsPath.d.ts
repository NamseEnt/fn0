import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * `aws.ec2.NetworkInsightsPath` provides details about a specific Network Insights Path.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ec2.getNetworkInsightsPath({
 *     networkInsightsPathId: exampleAwsEc2NetworkInsightsPath.id,
 * });
 * ```
 */
export declare function getNetworkInsightsPath(args?: GetNetworkInsightsPathArgs, opts?: pulumi.InvokeOptions): Promise<GetNetworkInsightsPathResult>;
/**
 * A collection of arguments for invoking getNetworkInsightsPath.
 */
export interface GetNetworkInsightsPathArgs {
    /**
     * Configuration block(s) for filtering. Detailed below.
     *
     * The arguments of this data source act as filters for querying the available
     * Network Insights Paths. The given filters must match exactly one Network Insights Path
     * whose data will be exported as attributes.
     */
    filters?: inputs.ec2.GetNetworkInsightsPathFilter[];
    /**
     * ID of the Network Insights Path to select.
     */
    networkInsightsPathId?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags assigned to the resource.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getNetworkInsightsPath.
 */
export interface GetNetworkInsightsPathResult {
    /**
     * ARN of the selected Network Insights Path.
     */
    readonly arn: string;
    /**
     * AWS resource that is the destination of the path.
     */
    readonly destination: string;
    /**
     * ARN of the destination.
     */
    readonly destinationArn: string;
    /**
     * IP address of the AWS resource that is the destination of the path.
     */
    readonly destinationIp: string;
    /**
     * Destination port.
     */
    readonly destinationPort: number;
    /**
     * Filters of the network paths at the destination.
     */
    readonly filterAtDestinations: outputs.ec2.GetNetworkInsightsPathFilterAtDestination[];
    /**
     * Filters of the network paths at the source.
     */
    readonly filterAtSources: outputs.ec2.GetNetworkInsightsPathFilterAtSource[];
    readonly filters?: outputs.ec2.GetNetworkInsightsPathFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly networkInsightsPathId: string;
    /**
     * Protocol.
     */
    readonly protocol: string;
    readonly region: string;
    /**
     * AWS resource that is the source of the path.
     */
    readonly source: string;
    /**
     * ARN of the source.
     */
    readonly sourceArn: string;
    /**
     * IP address of the AWS resource that is the source of the path.
     */
    readonly sourceIp: string;
    /**
     * Map of tags assigned to the resource.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * `aws.ec2.NetworkInsightsPath` provides details about a specific Network Insights Path.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ec2.getNetworkInsightsPath({
 *     networkInsightsPathId: exampleAwsEc2NetworkInsightsPath.id,
 * });
 * ```
 */
export declare function getNetworkInsightsPathOutput(args?: GetNetworkInsightsPathOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetNetworkInsightsPathResult>;
/**
 * A collection of arguments for invoking getNetworkInsightsPath.
 */
export interface GetNetworkInsightsPathOutputArgs {
    /**
     * Configuration block(s) for filtering. Detailed below.
     *
     * The arguments of this data source act as filters for querying the available
     * Network Insights Paths. The given filters must match exactly one Network Insights Path
     * whose data will be exported as attributes.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.ec2.GetNetworkInsightsPathFilterArgs>[]>;
    /**
     * ID of the Network Insights Path to select.
     */
    networkInsightsPathId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
