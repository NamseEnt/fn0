import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS RDS (Relational Database) Clusters.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.rds.getClusters({
 *     filters: [{
 *         name: "engine",
 *         values: ["aurora-postgresql"],
 *     }],
 * });
 * ```
 */
export declare function getClusters(args?: GetClustersArgs, opts?: pulumi.InvokeOptions): Promise<GetClustersResult>;
/**
 * A collection of arguments for invoking getClusters.
 */
export interface GetClustersArgs {
    /**
     * Configuration block(s) for filtering. Detailed below.
     */
    filters?: inputs.rds.GetClustersFilter[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getClusters.
 */
export interface GetClustersResult {
    /**
     * Set of cluster ARNs of the matched RDS clusters.
     */
    readonly clusterArns: string[];
    /**
     * Set of ARNs of cluster identifiers of the matched RDS clusters.
     */
    readonly clusterIdentifiers: string[];
    readonly filters?: outputs.rds.GetClustersFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Data source for managing an AWS RDS (Relational Database) Clusters.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.rds.getClusters({
 *     filters: [{
 *         name: "engine",
 *         values: ["aurora-postgresql"],
 *     }],
 * });
 * ```
 */
export declare function getClustersOutput(args?: GetClustersOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetClustersResult>;
/**
 * A collection of arguments for invoking getClusters.
 */
export interface GetClustersOutputArgs {
    /**
     * Configuration block(s) for filtering. Detailed below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.rds.GetClustersFilterArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
