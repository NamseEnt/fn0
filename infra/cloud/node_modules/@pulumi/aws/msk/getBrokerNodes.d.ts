import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Get information on an Amazon MSK Broker Nodes.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.msk.getBrokerNodes({
 *     clusterArn: exampleAwsMskCluster.arn,
 * });
 * ```
 */
export declare function getBrokerNodes(args: GetBrokerNodesArgs, opts?: pulumi.InvokeOptions): Promise<GetBrokerNodesResult>;
/**
 * A collection of arguments for invoking getBrokerNodes.
 */
export interface GetBrokerNodesArgs {
    /**
     * ARN of the cluster the nodes belong to.
     */
    clusterArn: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getBrokerNodes.
 */
export interface GetBrokerNodesResult {
    readonly clusterArn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly nodeInfoLists: outputs.msk.GetBrokerNodesNodeInfoList[];
    readonly region: string;
}
/**
 * Get information on an Amazon MSK Broker Nodes.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.msk.getBrokerNodes({
 *     clusterArn: exampleAwsMskCluster.arn,
 * });
 * ```
 */
export declare function getBrokerNodesOutput(args: GetBrokerNodesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetBrokerNodesResult>;
/**
 * A collection of arguments for invoking getBrokerNodes.
 */
export interface GetBrokerNodesOutputArgs {
    /**
     * ARN of the cluster the nodes belong to.
     */
    clusterArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
