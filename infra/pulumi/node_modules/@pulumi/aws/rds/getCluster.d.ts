import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides information about an RDS cluster.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const clusterName = aws.rds.getCluster({
 *     clusterIdentifier: "clusterName",
 * });
 * ```
 */
export declare function getCluster(args: GetClusterArgs, opts?: pulumi.InvokeOptions): Promise<GetClusterResult>;
/**
 * A collection of arguments for invoking getCluster.
 */
export interface GetClusterArgs {
    /**
     * Cluster identifier of the RDS cluster.
     */
    clusterIdentifier: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * A map of tags assigned to the resource.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getCluster.
 */
export interface GetClusterResult {
    readonly arn: string;
    readonly availabilityZones: string[];
    readonly backtrackWindow: number;
    readonly backupRetentionPeriod: number;
    readonly clusterIdentifier: string;
    readonly clusterMembers: string[];
    readonly clusterResourceId: string;
    readonly clusterScalabilityType: string;
    readonly databaseInsightsMode: string;
    readonly databaseName: string;
    readonly dbClusterParameterGroupName: string;
    readonly dbSubnetGroupName: string;
    readonly dbSystemId: string;
    readonly enabledCloudwatchLogsExports: string[];
    readonly endpoint: string;
    readonly engine: string;
    readonly engineMode: string;
    readonly engineVersion: string;
    readonly finalSnapshotIdentifier: string;
    readonly hostedZoneId: string;
    readonly iamDatabaseAuthenticationEnabled: boolean;
    readonly iamRoles: string[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly kmsKeyId: string;
    readonly masterUserSecrets: outputs.rds.GetClusterMasterUserSecret[];
    readonly masterUsername: string;
    readonly monitoringInterval: number;
    readonly monitoringRoleArn: string;
    readonly networkType: string;
    readonly port: number;
    readonly preferredBackupWindow: string;
    readonly preferredMaintenanceWindow: string;
    readonly readerEndpoint: string;
    readonly region: string;
    readonly replicationSourceIdentifier: string;
    readonly storageEncrypted: boolean;
    /**
     * A map of tags assigned to the resource.
     */
    readonly tags: {
        [key: string]: string;
    };
    readonly vpcSecurityGroupIds: string[];
}
/**
 * Provides information about an RDS cluster.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const clusterName = aws.rds.getCluster({
 *     clusterIdentifier: "clusterName",
 * });
 * ```
 */
export declare function getClusterOutput(args: GetClusterOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetClusterResult>;
/**
 * A collection of arguments for invoking getCluster.
 */
export interface GetClusterOutputArgs {
    /**
     * Cluster identifier of the RDS cluster.
     */
    clusterIdentifier: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags assigned to the resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
