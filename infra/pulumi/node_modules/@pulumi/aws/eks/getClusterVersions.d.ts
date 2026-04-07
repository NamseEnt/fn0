import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing AWS EKS (Elastic Kubernetes) Cluster Versions.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.eks.getClusterVersions({});
 * export const eksClusterVersions = example.then(example => example.clusterVersions);
 * export const eksClusterVersionFiltered = example.then(example => .filter(version => version.clusterVersion == "1.33").map(version => (version)));
 * export const eksClusterVersionList = example.then(example => .map(version => (version.clusterVersion)));
 * ```
 *
 * ### Filter by Cluster Type
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.eks.getClusterVersions({
 *     clusterType: "eks",
 * });
 * ```
 *
 * ### Filter by Version Status
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.eks.getClusterVersions({
 *     versionStatus: "STANDARD_SUPPORT",
 * });
 * ```
 */
export declare function getClusterVersions(args?: GetClusterVersionsArgs, opts?: pulumi.InvokeOptions): Promise<GetClusterVersionsResult>;
/**
 * A collection of arguments for invoking getClusterVersions.
 */
export interface GetClusterVersionsArgs {
    /**
     * Type of clusters to filter by.
     * Currently, the only valid value is `eks`.
     */
    clusterType?: string;
    clusterVersionsOnlies?: string[];
    /**
     * Whether to show only the default versions of Kubernetes supported by EKS.
     */
    defaultOnly?: boolean;
    /**
     * Whether to include all kubernetes versions in the response.
     */
    includeAll?: boolean;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Status of the EKS cluster versions to list.
     * Valid values are `STANDARD_SUPPORT` or `UNSUPPORTED` or `EXTENDED_SUPPORT`.
     */
    versionStatus?: string;
}
/**
 * A collection of values returned by getClusterVersions.
 */
export interface GetClusterVersionsResult {
    /**
     * Type of cluster that the version belongs to.
     */
    readonly clusterType?: string;
    /**
     * A list of Kubernetes version information.
     */
    readonly clusterVersions: outputs.eks.GetClusterVersionsClusterVersion[];
    readonly clusterVersionsOnlies?: string[];
    readonly defaultOnly?: boolean;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly includeAll?: boolean;
    readonly region: string;
    /**
     * Status of the EKS cluster version.
     */
    readonly versionStatus?: string;
}
/**
 * Data source for managing AWS EKS (Elastic Kubernetes) Cluster Versions.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.eks.getClusterVersions({});
 * export const eksClusterVersions = example.then(example => example.clusterVersions);
 * export const eksClusterVersionFiltered = example.then(example => .filter(version => version.clusterVersion == "1.33").map(version => (version)));
 * export const eksClusterVersionList = example.then(example => .map(version => (version.clusterVersion)));
 * ```
 *
 * ### Filter by Cluster Type
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.eks.getClusterVersions({
 *     clusterType: "eks",
 * });
 * ```
 *
 * ### Filter by Version Status
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.eks.getClusterVersions({
 *     versionStatus: "STANDARD_SUPPORT",
 * });
 * ```
 */
export declare function getClusterVersionsOutput(args?: GetClusterVersionsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetClusterVersionsResult>;
/**
 * A collection of arguments for invoking getClusterVersions.
 */
export interface GetClusterVersionsOutputArgs {
    /**
     * Type of clusters to filter by.
     * Currently, the only valid value is `eks`.
     */
    clusterType?: pulumi.Input<string>;
    clusterVersionsOnlies?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Whether to show only the default versions of Kubernetes supported by EKS.
     */
    defaultOnly?: pulumi.Input<boolean>;
    /**
     * Whether to include all kubernetes versions in the response.
     */
    includeAll?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Status of the EKS cluster versions to list.
     * Valid values are `STANDARD_SUPPORT` or `UNSUPPORTED` or `EXTENDED_SUPPORT`.
     */
    versionStatus?: pulumi.Input<string>;
}
