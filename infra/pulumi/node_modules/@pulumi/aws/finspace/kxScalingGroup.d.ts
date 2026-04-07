import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS FinSpace Kx Scaling Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.finspace.KxScalingGroup("example", {
 *     name: "my-tf-kx-scalinggroup",
 *     environmentId: exampleAwsFinspaceKxEnvironment.id,
 *     availabilityZoneId: "use1-az2",
 *     hostType: "kx.sg.4xlarge",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import an AWS FinSpace Kx Scaling Group using the `id` (environment ID and scaling group name, comma-delimited). For example:
 *
 * ```sh
 * $ pulumi import aws:finspace/kxScalingGroup:KxScalingGroup example n3ceo7wqxoxcti5tujqwzs,my-tf-kx-scalinggroup
 * ```
 */
export declare class KxScalingGroup extends pulumi.CustomResource {
    /**
     * Get an existing KxScalingGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: KxScalingGroupState, opts?: pulumi.CustomResourceOptions): KxScalingGroup;
    /**
     * Returns true if the given object is an instance of KxScalingGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is KxScalingGroup;
    /**
     * Amazon Resource Name (ARN) identifier of the KX Scaling Group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The availability zone identifiers for the requested regions.
     */
    readonly availabilityZoneId: pulumi.Output<string>;
    /**
     * The list of Managed kdb clusters that are currently active in the given scaling group.
     */
    readonly clusters: pulumi.Output<string[]>;
    /**
     * The timestamp at which the scaling group was created in FinSpace. The value is determined as epoch time in milliseconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000000.
     */
    readonly createdTimestamp: pulumi.Output<string>;
    /**
     * A unique identifier for the kdb environment, where you want to create the scaling group.
     */
    readonly environmentId: pulumi.Output<string>;
    /**
     * The memory and CPU capabilities of the scaling group host on which FinSpace Managed kdb clusters will be placed.
     *
     * The following arguments are optional:
     */
    readonly hostType: pulumi.Output<string>;
    /**
     * Last timestamp at which the scaling group was updated in FinSpace. Value determined as epoch time in seconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000.
     */
    readonly lastModifiedTimestamp: pulumi.Output<string>;
    /**
     * Unique name for the scaling group that you want to create.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The status of scaling group.
     * * `CREATING` - The scaling group creation is in progress.
     * * `CREATE_FAILED` - The scaling group creation has failed.
     * * `ACTIVE` - The scaling group is active.
     * * `UPDATING` - The scaling group is in the process of being updated.
     * * `UPDATE_FAILED` - The update action failed.
     * * `DELETING` - The scaling group is in the process of being deleted.
     * * `DELETE_FAILED` - The system failed to delete the scaling group.
     * * `DELETED` - The scaling group is successfully deleted.
     */
    readonly status: pulumi.Output<string>;
    /**
     * The error message when a failed state occurs.
     */
    readonly statusReason: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level. You can add up to 50 tags to a scaling group.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a KxScalingGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: KxScalingGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering KxScalingGroup resources.
 */
export interface KxScalingGroupState {
    /**
     * Amazon Resource Name (ARN) identifier of the KX Scaling Group.
     */
    arn?: pulumi.Input<string>;
    /**
     * The availability zone identifiers for the requested regions.
     */
    availabilityZoneId?: pulumi.Input<string>;
    /**
     * The list of Managed kdb clusters that are currently active in the given scaling group.
     */
    clusters?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The timestamp at which the scaling group was created in FinSpace. The value is determined as epoch time in milliseconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000000.
     */
    createdTimestamp?: pulumi.Input<string>;
    /**
     * A unique identifier for the kdb environment, where you want to create the scaling group.
     */
    environmentId?: pulumi.Input<string>;
    /**
     * The memory and CPU capabilities of the scaling group host on which FinSpace Managed kdb clusters will be placed.
     *
     * The following arguments are optional:
     */
    hostType?: pulumi.Input<string>;
    /**
     * Last timestamp at which the scaling group was updated in FinSpace. Value determined as epoch time in seconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000.
     */
    lastModifiedTimestamp?: pulumi.Input<string>;
    /**
     * Unique name for the scaling group that you want to create.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The status of scaling group.
     * * `CREATING` - The scaling group creation is in progress.
     * * `CREATE_FAILED` - The scaling group creation has failed.
     * * `ACTIVE` - The scaling group is active.
     * * `UPDATING` - The scaling group is in the process of being updated.
     * * `UPDATE_FAILED` - The update action failed.
     * * `DELETING` - The scaling group is in the process of being deleted.
     * * `DELETE_FAILED` - The system failed to delete the scaling group.
     * * `DELETED` - The scaling group is successfully deleted.
     */
    status?: pulumi.Input<string>;
    /**
     * The error message when a failed state occurs.
     */
    statusReason?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level. You can add up to 50 tags to a scaling group.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a KxScalingGroup resource.
 */
export interface KxScalingGroupArgs {
    /**
     * The availability zone identifiers for the requested regions.
     */
    availabilityZoneId: pulumi.Input<string>;
    /**
     * A unique identifier for the kdb environment, where you want to create the scaling group.
     */
    environmentId: pulumi.Input<string>;
    /**
     * The memory and CPU capabilities of the scaling group host on which FinSpace Managed kdb clusters will be placed.
     *
     * The following arguments are optional:
     */
    hostType: pulumi.Input<string>;
    /**
     * Unique name for the scaling group that you want to create.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level. You can add up to 50 tags to a scaling group.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
