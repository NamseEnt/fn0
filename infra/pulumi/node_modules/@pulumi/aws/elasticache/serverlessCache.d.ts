import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an ElastiCache Serverless Cache resource which manages memcached, redis or valkey.
 *
 * ## Example Usage
 *
 * ### Memcached Serverless
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.elasticache.ServerlessCache("example", {
 *     engine: "memcached",
 *     name: "example",
 *     cacheUsageLimits: {
 *         dataStorage: {
 *             maximum: 10,
 *             unit: "GB",
 *         },
 *         ecpuPerSeconds: [{
 *             maximum: 5000,
 *         }],
 *     },
 *     description: "Test Server",
 *     kmsKeyId: test.arn,
 *     majorEngineVersion: "1.6",
 *     securityGroupIds: [testAwsSecurityGroup.id],
 *     subnetIds: testAwsSubnet.map(__item => __item.id),
 * });
 * ```
 *
 * ### Redis OSS Serverless
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.elasticache.ServerlessCache("example", {
 *     engine: "redis",
 *     name: "example",
 *     cacheUsageLimits: {
 *         dataStorage: {
 *             maximum: 10,
 *             unit: "GB",
 *         },
 *         ecpuPerSeconds: [{
 *             maximum: 5000,
 *         }],
 *     },
 *     dailySnapshotTime: "09:00",
 *     description: "Test Server",
 *     kmsKeyId: test.arn,
 *     majorEngineVersion: "7",
 *     snapshotRetentionLimit: 1,
 *     securityGroupIds: [testAwsSecurityGroup.id],
 *     subnetIds: testAwsSubnet.map(__item => __item.id),
 * });
 * ```
 *
 * ### Valkey Serverless
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.elasticache.ServerlessCache("example", {
 *     engine: "valkey",
 *     name: "example",
 *     cacheUsageLimits: {
 *         dataStorage: {
 *             maximum: 10,
 *             unit: "GB",
 *         },
 *         ecpuPerSeconds: [{
 *             maximum: 5000,
 *         }],
 *     },
 *     dailySnapshotTime: "09:00",
 *     description: "Test Server",
 *     kmsKeyId: test.arn,
 *     majorEngineVersion: "7",
 *     snapshotRetentionLimit: 1,
 *     securityGroupIds: [testAwsSecurityGroup.id],
 *     subnetIds: testAwsSubnet.map(__item => __item.id),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import ElastiCache Serverless Cache using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:elasticache/serverlessCache:ServerlessCache my_cluster my_cluster
 * ```
 */
export declare class ServerlessCache extends pulumi.CustomResource {
    /**
     * Get an existing ServerlessCache resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServerlessCacheState, opts?: pulumi.CustomResourceOptions): ServerlessCache;
    /**
     * Returns true if the given object is an instance of ServerlessCache.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ServerlessCache;
    /**
     * The Amazon Resource Name (ARN) of the serverless cache.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Sets the cache usage limits for storage and ElastiCache Processing Units for the cache. See `cacheUsageLimits` Block for details.
     */
    readonly cacheUsageLimits: pulumi.Output<outputs.elasticache.ServerlessCacheCacheUsageLimits | undefined>;
    /**
     * Timestamp of when the serverless cache was created.
     */
    readonly createTime: pulumi.Output<string>;
    /**
     * The daily time that snapshots will be created from the new serverless cache. Only supported for engine types `"redis"` or `"valkey"`. Defaults to `0`.
     */
    readonly dailySnapshotTime: pulumi.Output<string>;
    /**
     * User-provided description for the serverless cache. The default is NULL.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Represents the information required for client programs to connect to a cache node. See `endpoint` Block for details.
     */
    readonly endpoints: pulumi.Output<outputs.elasticache.ServerlessCacheEndpoint[]>;
    /**
     * Name of the cache engine to be used for this cache cluster. Valid values are `memcached`, `redis` or `valkey`.
     */
    readonly engine: pulumi.Output<string>;
    /**
     * The name and version number of the engine the serverless cache is compatible with.
     */
    readonly fullEngineVersion: pulumi.Output<string>;
    /**
     * ARN of the customer managed key for encrypting the data at rest. If no KMS key is provided, a default service key is used.
     */
    readonly kmsKeyId: pulumi.Output<string | undefined>;
    /**
     * The version of the cache engine that will be used to create the serverless cache.
     * See [Describe Cache Engine Versions](https://docs.aws.amazon.com/cli/latest/reference/elasticache/describe-cache-engine-versions.html) in the AWS Documentation for supported versions.
     */
    readonly majorEngineVersion: pulumi.Output<string>;
    /**
     * The Cluster name which serves as a unique identifier to the serverless cache
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Represents the information required for client programs to connect to a cache node. See `readerEndpoint` Block for details.
     */
    readonly readerEndpoints: pulumi.Output<outputs.elasticache.ServerlessCacheReaderEndpoint[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A list of the one or more VPC security groups to be associated with the serverless cache. The security group will authorize traffic access for the VPC end-point (private-link). If no other information is given this will be the VPC’s Default Security Group that is associated with the cluster VPC end-point.
     */
    readonly securityGroupIds: pulumi.Output<string[]>;
    /**
     * The list of ARN(s) of the snapshot that the new serverless cache will be created from. Available for Redis only.
     */
    readonly snapshotArnsToRestores: pulumi.Output<string[] | undefined>;
    /**
     * The number of snapshots that will be retained for the serverless cache that is being created. As new snapshots beyond this limit are added, the oldest snapshots will be deleted on a rolling basis. Available for Redis only.
     */
    readonly snapshotRetentionLimit: pulumi.Output<number>;
    /**
     * The current status of the serverless cache. The allowed values are CREATING, AVAILABLE, DELETING, CREATE-FAILED and MODIFYING.
     */
    readonly status: pulumi.Output<string>;
    /**
     * A list of the identifiers of the subnets where the VPC endpoint for the serverless cache will be deployed. All the subnetIds must belong to the same VPC.
     */
    readonly subnetIds: pulumi.Output<string[]>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.elasticache.ServerlessCacheTimeouts | undefined>;
    /**
     * The identifier of the UserGroup to be associated with the serverless cache. Available for Redis only. Default is NULL.
     */
    readonly userGroupId: pulumi.Output<string | undefined>;
    /**
     * Create a ServerlessCache resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ServerlessCacheArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ServerlessCache resources.
 */
export interface ServerlessCacheState {
    /**
     * The Amazon Resource Name (ARN) of the serverless cache.
     */
    arn?: pulumi.Input<string>;
    /**
     * Sets the cache usage limits for storage and ElastiCache Processing Units for the cache. See `cacheUsageLimits` Block for details.
     */
    cacheUsageLimits?: pulumi.Input<inputs.elasticache.ServerlessCacheCacheUsageLimits>;
    /**
     * Timestamp of when the serverless cache was created.
     */
    createTime?: pulumi.Input<string>;
    /**
     * The daily time that snapshots will be created from the new serverless cache. Only supported for engine types `"redis"` or `"valkey"`. Defaults to `0`.
     */
    dailySnapshotTime?: pulumi.Input<string>;
    /**
     * User-provided description for the serverless cache. The default is NULL.
     */
    description?: pulumi.Input<string>;
    /**
     * Represents the information required for client programs to connect to a cache node. See `endpoint` Block for details.
     */
    endpoints?: pulumi.Input<pulumi.Input<inputs.elasticache.ServerlessCacheEndpoint>[]>;
    /**
     * Name of the cache engine to be used for this cache cluster. Valid values are `memcached`, `redis` or `valkey`.
     */
    engine?: pulumi.Input<string>;
    /**
     * The name and version number of the engine the serverless cache is compatible with.
     */
    fullEngineVersion?: pulumi.Input<string>;
    /**
     * ARN of the customer managed key for encrypting the data at rest. If no KMS key is provided, a default service key is used.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The version of the cache engine that will be used to create the serverless cache.
     * See [Describe Cache Engine Versions](https://docs.aws.amazon.com/cli/latest/reference/elasticache/describe-cache-engine-versions.html) in the AWS Documentation for supported versions.
     */
    majorEngineVersion?: pulumi.Input<string>;
    /**
     * The Cluster name which serves as a unique identifier to the serverless cache
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Represents the information required for client programs to connect to a cache node. See `readerEndpoint` Block for details.
     */
    readerEndpoints?: pulumi.Input<pulumi.Input<inputs.elasticache.ServerlessCacheReaderEndpoint>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of the one or more VPC security groups to be associated with the serverless cache. The security group will authorize traffic access for the VPC end-point (private-link). If no other information is given this will be the VPC’s Default Security Group that is associated with the cluster VPC end-point.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The list of ARN(s) of the snapshot that the new serverless cache will be created from. Available for Redis only.
     */
    snapshotArnsToRestores?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The number of snapshots that will be retained for the serverless cache that is being created. As new snapshots beyond this limit are added, the oldest snapshots will be deleted on a rolling basis. Available for Redis only.
     */
    snapshotRetentionLimit?: pulumi.Input<number>;
    /**
     * The current status of the serverless cache. The allowed values are CREATING, AVAILABLE, DELETING, CREATE-FAILED and MODIFYING.
     */
    status?: pulumi.Input<string>;
    /**
     * A list of the identifiers of the subnets where the VPC endpoint for the serverless cache will be deployed. All the subnetIds must belong to the same VPC.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.elasticache.ServerlessCacheTimeouts>;
    /**
     * The identifier of the UserGroup to be associated with the serverless cache. Available for Redis only. Default is NULL.
     */
    userGroupId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ServerlessCache resource.
 */
export interface ServerlessCacheArgs {
    /**
     * Sets the cache usage limits for storage and ElastiCache Processing Units for the cache. See `cacheUsageLimits` Block for details.
     */
    cacheUsageLimits?: pulumi.Input<inputs.elasticache.ServerlessCacheCacheUsageLimits>;
    /**
     * The daily time that snapshots will be created from the new serverless cache. Only supported for engine types `"redis"` or `"valkey"`. Defaults to `0`.
     */
    dailySnapshotTime?: pulumi.Input<string>;
    /**
     * User-provided description for the serverless cache. The default is NULL.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the cache engine to be used for this cache cluster. Valid values are `memcached`, `redis` or `valkey`.
     */
    engine: pulumi.Input<string>;
    /**
     * ARN of the customer managed key for encrypting the data at rest. If no KMS key is provided, a default service key is used.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The version of the cache engine that will be used to create the serverless cache.
     * See [Describe Cache Engine Versions](https://docs.aws.amazon.com/cli/latest/reference/elasticache/describe-cache-engine-versions.html) in the AWS Documentation for supported versions.
     */
    majorEngineVersion?: pulumi.Input<string>;
    /**
     * The Cluster name which serves as a unique identifier to the serverless cache
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of the one or more VPC security groups to be associated with the serverless cache. The security group will authorize traffic access for the VPC end-point (private-link). If no other information is given this will be the VPC’s Default Security Group that is associated with the cluster VPC end-point.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The list of ARN(s) of the snapshot that the new serverless cache will be created from. Available for Redis only.
     */
    snapshotArnsToRestores?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The number of snapshots that will be retained for the serverless cache that is being created. As new snapshots beyond this limit are added, the oldest snapshots will be deleted on a rolling basis. Available for Redis only.
     */
    snapshotRetentionLimit?: pulumi.Input<number>;
    /**
     * A list of the identifiers of the subnets where the VPC endpoint for the serverless cache will be deployed. All the subnetIds must belong to the same VPC.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.elasticache.ServerlessCacheTimeouts>;
    /**
     * The identifier of the UserGroup to be associated with the serverless cache. Available for Redis only. Default is NULL.
     */
    userGroupId?: pulumi.Input<string>;
}
