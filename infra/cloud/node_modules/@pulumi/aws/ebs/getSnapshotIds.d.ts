import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Use this data source to get a list of EBS Snapshot IDs matching the specified
 * criteria.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const ebsVolumes = aws.ebs.getSnapshotIds({
 *     owners: ["self"],
 *     filters: [
 *         {
 *             name: "volume-size",
 *             values: ["40"],
 *         },
 *         {
 *             name: "tag:Name",
 *             values: ["Example"],
 *         },
 *     ],
 * });
 * ```
 */
export declare function getSnapshotIds(args?: GetSnapshotIdsArgs, opts?: pulumi.InvokeOptions): Promise<GetSnapshotIdsResult>;
/**
 * A collection of arguments for invoking getSnapshotIds.
 */
export interface GetSnapshotIdsArgs {
    /**
     * One or more name/value pairs to filter off of. There are several valid keys, for a full reference, check out [describe-volumes in the AWS CLI reference][1].
     */
    filters?: inputs.ebs.GetSnapshotIdsFilter[];
    /**
     * Returns the snapshots owned by the specified owner id. Multiple owners can be specified.
     */
    owners?: string[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * One or more AWS accounts IDs that can create volumes from the snapshot.
     */
    restorableByUserIds?: string[];
}
/**
 * A collection of values returned by getSnapshotIds.
 */
export interface GetSnapshotIdsResult {
    readonly filters?: outputs.ebs.GetSnapshotIdsFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Set of EBS snapshot IDs, sorted by creation time in descending order.
     */
    readonly ids: string[];
    readonly owners?: string[];
    readonly region: string;
    readonly restorableByUserIds?: string[];
}
/**
 * Use this data source to get a list of EBS Snapshot IDs matching the specified
 * criteria.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const ebsVolumes = aws.ebs.getSnapshotIds({
 *     owners: ["self"],
 *     filters: [
 *         {
 *             name: "volume-size",
 *             values: ["40"],
 *         },
 *         {
 *             name: "tag:Name",
 *             values: ["Example"],
 *         },
 *     ],
 * });
 * ```
 */
export declare function getSnapshotIdsOutput(args?: GetSnapshotIdsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSnapshotIdsResult>;
/**
 * A collection of arguments for invoking getSnapshotIds.
 */
export interface GetSnapshotIdsOutputArgs {
    /**
     * One or more name/value pairs to filter off of. There are several valid keys, for a full reference, check out [describe-volumes in the AWS CLI reference][1].
     */
    filters?: pulumi.Input<pulumi.Input<inputs.ebs.GetSnapshotIdsFilterArgs>[]>;
    /**
     * Returns the snapshots owned by the specified owner id. Multiple owners can be specified.
     */
    owners?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * One or more AWS accounts IDs that can create volumes from the snapshot.
     */
    restorableByUserIds?: pulumi.Input<pulumi.Input<string>[]>;
}
