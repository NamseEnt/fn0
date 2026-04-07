import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a DMS (Data Migration Service) replication subnet group resource. DMS replication subnet groups can be created, updated, deleted, and imported.
 *
 * > **Note:** AWS requires a special IAM role called `dms-vpc-role` when using this resource. See the example below to create it as part of your configuration.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * // Create a new replication subnet group
 * const example = new aws.dms.ReplicationSubnetGroup("example", {
 *     replicationSubnetGroupDescription: "Example replication subnet group",
 *     replicationSubnetGroupId: "example-dms-replication-subnet-group-tf",
 *     subnetIds: [
 *         "subnet-12345678",
 *         "subnet-12345679",
 *     ],
 *     tags: {
 *         Name: "example",
 *     },
 * });
 * ```
 *
 * ### Creating special IAM role
 *
 * If your account does not already include the `dms-vpc-role` IAM role, you will need to create it to allow DMS to manage subnets in the VPC.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const dms_vpc_role = new aws.iam.Role("dms-vpc-role", {
 *     name: "dms-vpc-role",
 *     description: "Allows DMS to manage VPC",
 *     assumeRolePolicy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Effect: "Allow",
 *             Principal: {
 *                 Service: "dms.amazonaws.com",
 *             },
 *             Action: "sts:AssumeRole",
 *         }],
 *     }),
 * });
 * const example = new aws.iam.RolePolicyAttachment("example", {
 *     role: dms_vpc_role.name,
 *     policyArn: "arn:aws:iam::aws:policy/service-role/AmazonDMSVPCManagementRole",
 * });
 * const exampleReplicationSubnetGroup = new aws.dms.ReplicationSubnetGroup("example", {
 *     replicationSubnetGroupDescription: "Example",
 *     replicationSubnetGroupId: "example-id",
 *     subnetIds: [
 *         "subnet-12345678",
 *         "subnet-12345679",
 *     ],
 *     tags: {
 *         Name: "example-id",
 *     },
 * }, {
 *     dependsOn: [example],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import replication subnet groups using the `replication_subnet_group_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:dms/replicationSubnetGroup:ReplicationSubnetGroup test test-dms-replication-subnet-group-tf
 * ```
 */
export declare class ReplicationSubnetGroup extends pulumi.CustomResource {
    /**
     * Get an existing ReplicationSubnetGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ReplicationSubnetGroupState, opts?: pulumi.CustomResourceOptions): ReplicationSubnetGroup;
    /**
     * Returns true if the given object is an instance of ReplicationSubnetGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ReplicationSubnetGroup;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly replicationSubnetGroupArn: pulumi.Output<string>;
    /**
     * Description for the subnet group.
     */
    readonly replicationSubnetGroupDescription: pulumi.Output<string>;
    /**
     * Name for the replication subnet group. This value is stored as a lowercase string. It must contain no more than 255 alphanumeric characters, periods, spaces, underscores, or hyphens and cannot be `default`.
     */
    readonly replicationSubnetGroupId: pulumi.Output<string>;
    /**
     * List of at least 2 EC2 subnet IDs for the subnet group. The subnets must cover at least 2 availability zones.
     */
    readonly subnetIds: pulumi.Output<string[]>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The ID of the VPC the subnet group is in.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a ReplicationSubnetGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ReplicationSubnetGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ReplicationSubnetGroup resources.
 */
export interface ReplicationSubnetGroupState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    replicationSubnetGroupArn?: pulumi.Input<string>;
    /**
     * Description for the subnet group.
     */
    replicationSubnetGroupDescription?: pulumi.Input<string>;
    /**
     * Name for the replication subnet group. This value is stored as a lowercase string. It must contain no more than 255 alphanumeric characters, periods, spaces, underscores, or hyphens and cannot be `default`.
     */
    replicationSubnetGroupId?: pulumi.Input<string>;
    /**
     * List of at least 2 EC2 subnet IDs for the subnet group. The subnets must cover at least 2 availability zones.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the VPC the subnet group is in.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ReplicationSubnetGroup resource.
 */
export interface ReplicationSubnetGroupArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Description for the subnet group.
     */
    replicationSubnetGroupDescription: pulumi.Input<string>;
    /**
     * Name for the replication subnet group. This value is stored as a lowercase string. It must contain no more than 255 alphanumeric characters, periods, spaces, underscores, or hyphens and cannot be `default`.
     */
    replicationSubnetGroupId: pulumi.Input<string>;
    /**
     * List of at least 2 EC2 subnet IDs for the subnet group. The subnets must cover at least 2 availability zones.
     */
    subnetIds: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
