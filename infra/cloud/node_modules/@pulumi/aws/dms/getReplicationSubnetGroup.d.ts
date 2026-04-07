import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS DMS (Database Migration) Replication Subnet Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.dms.getReplicationSubnetGroup({
 *     replicationSubnetGroupId: testAwsDmsReplicationSubnetGroup.replicationSubnetGroupId,
 * });
 * ```
 */
export declare function getReplicationSubnetGroup(args: GetReplicationSubnetGroupArgs, opts?: pulumi.InvokeOptions): Promise<GetReplicationSubnetGroupResult>;
/**
 * A collection of arguments for invoking getReplicationSubnetGroup.
 */
export interface GetReplicationSubnetGroupArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Name for the replication subnet group. This value is stored as a lowercase string. It must contain no more than 255 alphanumeric characters, periods, spaces, underscores, or hyphens and cannot be `default`.
     */
    replicationSubnetGroupId: string;
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getReplicationSubnetGroup.
 */
export interface GetReplicationSubnetGroupResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    readonly replicationSubnetGroupArn: string;
    /**
     * Description for the subnet group.
     */
    readonly replicationSubnetGroupDescription: string;
    readonly replicationSubnetGroupId: string;
    readonly subnetGroupStatus: string;
    /**
     * List of at least 2 EC2 subnet IDs for the subnet group. The subnets must cover at least 2 availability zones.
     */
    readonly subnetIds: string[];
    readonly tags: {
        [key: string]: string;
    };
    /**
     * The ID of the VPC the subnet group is in.
     */
    readonly vpcId: string;
}
/**
 * Data source for managing an AWS DMS (Database Migration) Replication Subnet Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.dms.getReplicationSubnetGroup({
 *     replicationSubnetGroupId: testAwsDmsReplicationSubnetGroup.replicationSubnetGroupId,
 * });
 * ```
 */
export declare function getReplicationSubnetGroupOutput(args: GetReplicationSubnetGroupOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetReplicationSubnetGroupResult>;
/**
 * A collection of arguments for invoking getReplicationSubnetGroup.
 */
export interface GetReplicationSubnetGroupOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name for the replication subnet group. This value is stored as a lowercase string. It must contain no more than 255 alphanumeric characters, periods, spaces, underscores, or hyphens and cannot be `default`.
     */
    replicationSubnetGroupId: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
