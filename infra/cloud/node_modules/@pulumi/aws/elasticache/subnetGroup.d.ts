import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an ElastiCache Subnet Group resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const foo = new aws.ec2.Vpc("foo", {
 *     cidrBlock: "10.0.0.0/16",
 *     tags: {
 *         Name: "tf-test",
 *     },
 * });
 * const fooSubnet = new aws.ec2.Subnet("foo", {
 *     vpcId: foo.id,
 *     cidrBlock: "10.0.0.0/24",
 *     availabilityZone: "us-west-2a",
 *     tags: {
 *         Name: "tf-test",
 *     },
 * });
 * const bar = new aws.elasticache.SubnetGroup("bar", {
 *     name: "tf-test-cache-subnet",
 *     subnetIds: [fooSubnet.id],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import ElastiCache Subnet Groups using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:elasticache/subnetGroup:SubnetGroup bar tf-test-cache-subnet
 * ```
 */
export declare class SubnetGroup extends pulumi.CustomResource {
    /**
     * Get an existing SubnetGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SubnetGroupState, opts?: pulumi.CustomResourceOptions): SubnetGroup;
    /**
     * Returns true if the given object is an instance of SubnetGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SubnetGroup;
    readonly arn: pulumi.Output<string>;
    /**
     * Description for the cache subnet group. Defaults to "Managed by Pulumi".
     */
    readonly description: pulumi.Output<string>;
    /**
     * Name for the cache subnet group. ElastiCache converts this name to lowercase.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * List of VPC Subnet IDs for the cache subnet group
     */
    readonly subnetIds: pulumi.Output<string[]>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The Amazon Virtual Private Cloud identifier (VPC ID) of the cache subnet group.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a SubnetGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SubnetGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SubnetGroup resources.
 */
export interface SubnetGroupState {
    arn?: pulumi.Input<string>;
    /**
     * Description for the cache subnet group. Defaults to "Managed by Pulumi".
     */
    description?: pulumi.Input<string>;
    /**
     * Name for the cache subnet group. ElastiCache converts this name to lowercase.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of VPC Subnet IDs for the cache subnet group
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The Amazon Virtual Private Cloud identifier (VPC ID) of the cache subnet group.
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SubnetGroup resource.
 */
export interface SubnetGroupArgs {
    /**
     * Description for the cache subnet group. Defaults to "Managed by Pulumi".
     */
    description?: pulumi.Input<string>;
    /**
     * Name for the cache subnet group. ElastiCache converts this name to lowercase.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of VPC Subnet IDs for the cache subnet group
     */
    subnetIds: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
