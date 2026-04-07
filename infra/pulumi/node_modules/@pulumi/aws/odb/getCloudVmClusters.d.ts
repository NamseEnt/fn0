import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for retrieving all cloud vm clusters AWS for Oracle Database@AWS.
 *
 * You can find out more about Oracle Database@AWS from [User Guide](https://docs.aws.amazon.com/odb/latest/UserGuide/what-is-odb.html).
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.odb.getCloudVmClusters({});
 * ```
 */
export declare function getCloudVmClusters(args?: GetCloudVmClustersArgs, opts?: pulumi.InvokeOptions): Promise<GetCloudVmClustersResult>;
/**
 * A collection of arguments for invoking getCloudVmClusters.
 */
export interface GetCloudVmClustersArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getCloudVmClusters.
 */
export interface GetCloudVmClustersResult {
    /**
     * List of Cloud VM Clusters. It returns only basic information about the cloud VM clusters.
     */
    readonly cloudVmClusters: outputs.odb.GetCloudVmClustersCloudVmCluster[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Data source for retrieving all cloud vm clusters AWS for Oracle Database@AWS.
 *
 * You can find out more about Oracle Database@AWS from [User Guide](https://docs.aws.amazon.com/odb/latest/UserGuide/what-is-odb.html).
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.odb.getCloudVmClusters({});
 * ```
 */
export declare function getCloudVmClustersOutput(args?: GetCloudVmClustersOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetCloudVmClustersResult>;
/**
 * A collection of arguments for invoking getCloudVmClusters.
 */
export interface GetCloudVmClustersOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
