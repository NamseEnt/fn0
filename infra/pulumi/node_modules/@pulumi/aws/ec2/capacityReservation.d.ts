import * as pulumi from "@pulumi/pulumi";
import * as enums from "../types/enums";
/**
 * Provides an EC2 Capacity Reservation. This allows you to reserve capacity for your Amazon EC2 instances in a specific Availability Zone for any duration.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _default = new aws.ec2.CapacityReservation("default", {
 *     instanceType: aws.ec2.InstanceType.T2_Micro,
 *     instancePlatform: aws.ec2.InstancePlatform.LinuxUnix,
 *     availabilityZone: "eu-west-1a",
 *     instanceCount: 1,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Capacity Reservations using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/capacityReservation:CapacityReservation web cr-0123456789abcdef0
 * ```
 */
export declare class CapacityReservation extends pulumi.CustomResource {
    /**
     * Get an existing CapacityReservation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CapacityReservationState, opts?: pulumi.CustomResourceOptions): CapacityReservation;
    /**
     * Returns true if the given object is an instance of CapacityReservation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CapacityReservation;
    /**
     * The ARN of the Capacity Reservation.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The Availability Zone in which to create the Capacity Reservation.
     */
    readonly availabilityZone: pulumi.Output<string>;
    /**
     * Indicates whether the Capacity Reservation supports EBS-optimized instances.
     */
    readonly ebsOptimized: pulumi.Output<boolean | undefined>;
    /**
     * The date and time at which the Capacity Reservation expires. When a Capacity Reservation expires, the reserved capacity is released and you can no longer launch instances into it. Valid values: [RFC3339 time string](https://tools.ietf.org/html/rfc3339#section-5.8) (`YYYY-MM-DDTHH:MM:SSZ`)
     */
    readonly endDate: pulumi.Output<string | undefined>;
    /**
     * Indicates the way in which the Capacity Reservation ends. Specify either `unlimited` or `limited`.
     */
    readonly endDateType: pulumi.Output<string | undefined>;
    /**
     * Indicates whether the Capacity Reservation supports instances with temporary, block-level storage.
     */
    readonly ephemeralStorage: pulumi.Output<boolean | undefined>;
    /**
     * The number of instances for which to reserve capacity.
     */
    readonly instanceCount: pulumi.Output<number>;
    /**
     * Indicates the type of instance launches that the Capacity Reservation accepts. Specify either `open` or `targeted`.
     */
    readonly instanceMatchCriteria: pulumi.Output<string | undefined>;
    /**
     * The type of operating system for which to reserve capacity. Valid options are `Linux/UNIX`, `Red Hat Enterprise Linux`, `SUSE Linux`, `Windows`, `Windows with SQL Server`, `Windows with SQL Server Enterprise`, `Windows with SQL Server Standard` or `Windows with SQL Server Web`.
     */
    readonly instancePlatform: pulumi.Output<string>;
    /**
     * The instance type for which to reserve capacity.
     */
    readonly instanceType: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the Outpost on which to create the Capacity Reservation.
     */
    readonly outpostArn: pulumi.Output<string | undefined>;
    /**
     * The ID of the AWS account that owns the Capacity Reservation.
     */
    readonly ownerId: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the cluster placement group in which to create the Capacity Reservation.
     */
    readonly placementGroupArn: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Indicates the tenancy of the Capacity Reservation. Specify either `default` or `dedicated`.
     */
    readonly tenancy: pulumi.Output<string | undefined>;
    /**
     * Create a CapacityReservation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CapacityReservationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CapacityReservation resources.
 */
export interface CapacityReservationState {
    /**
     * The ARN of the Capacity Reservation.
     */
    arn?: pulumi.Input<string>;
    /**
     * The Availability Zone in which to create the Capacity Reservation.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * Indicates whether the Capacity Reservation supports EBS-optimized instances.
     */
    ebsOptimized?: pulumi.Input<boolean>;
    /**
     * The date and time at which the Capacity Reservation expires. When a Capacity Reservation expires, the reserved capacity is released and you can no longer launch instances into it. Valid values: [RFC3339 time string](https://tools.ietf.org/html/rfc3339#section-5.8) (`YYYY-MM-DDTHH:MM:SSZ`)
     */
    endDate?: pulumi.Input<string>;
    /**
     * Indicates the way in which the Capacity Reservation ends. Specify either `unlimited` or `limited`.
     */
    endDateType?: pulumi.Input<string>;
    /**
     * Indicates whether the Capacity Reservation supports instances with temporary, block-level storage.
     */
    ephemeralStorage?: pulumi.Input<boolean>;
    /**
     * The number of instances for which to reserve capacity.
     */
    instanceCount?: pulumi.Input<number>;
    /**
     * Indicates the type of instance launches that the Capacity Reservation accepts. Specify either `open` or `targeted`.
     */
    instanceMatchCriteria?: pulumi.Input<string>;
    /**
     * The type of operating system for which to reserve capacity. Valid options are `Linux/UNIX`, `Red Hat Enterprise Linux`, `SUSE Linux`, `Windows`, `Windows with SQL Server`, `Windows with SQL Server Enterprise`, `Windows with SQL Server Standard` or `Windows with SQL Server Web`.
     */
    instancePlatform?: pulumi.Input<string | enums.ec2.InstancePlatform>;
    /**
     * The instance type for which to reserve capacity.
     */
    instanceType?: pulumi.Input<string | enums.ec2.InstanceType>;
    /**
     * The Amazon Resource Name (ARN) of the Outpost on which to create the Capacity Reservation.
     */
    outpostArn?: pulumi.Input<string>;
    /**
     * The ID of the AWS account that owns the Capacity Reservation.
     */
    ownerId?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the cluster placement group in which to create the Capacity Reservation.
     */
    placementGroupArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Indicates the tenancy of the Capacity Reservation. Specify either `default` or `dedicated`.
     */
    tenancy?: pulumi.Input<string | enums.ec2.Tenancy>;
}
/**
 * The set of arguments for constructing a CapacityReservation resource.
 */
export interface CapacityReservationArgs {
    /**
     * The Availability Zone in which to create the Capacity Reservation.
     */
    availabilityZone: pulumi.Input<string>;
    /**
     * Indicates whether the Capacity Reservation supports EBS-optimized instances.
     */
    ebsOptimized?: pulumi.Input<boolean>;
    /**
     * The date and time at which the Capacity Reservation expires. When a Capacity Reservation expires, the reserved capacity is released and you can no longer launch instances into it. Valid values: [RFC3339 time string](https://tools.ietf.org/html/rfc3339#section-5.8) (`YYYY-MM-DDTHH:MM:SSZ`)
     */
    endDate?: pulumi.Input<string>;
    /**
     * Indicates the way in which the Capacity Reservation ends. Specify either `unlimited` or `limited`.
     */
    endDateType?: pulumi.Input<string>;
    /**
     * Indicates whether the Capacity Reservation supports instances with temporary, block-level storage.
     */
    ephemeralStorage?: pulumi.Input<boolean>;
    /**
     * The number of instances for which to reserve capacity.
     */
    instanceCount: pulumi.Input<number>;
    /**
     * Indicates the type of instance launches that the Capacity Reservation accepts. Specify either `open` or `targeted`.
     */
    instanceMatchCriteria?: pulumi.Input<string>;
    /**
     * The type of operating system for which to reserve capacity. Valid options are `Linux/UNIX`, `Red Hat Enterprise Linux`, `SUSE Linux`, `Windows`, `Windows with SQL Server`, `Windows with SQL Server Enterprise`, `Windows with SQL Server Standard` or `Windows with SQL Server Web`.
     */
    instancePlatform: pulumi.Input<string | enums.ec2.InstancePlatform>;
    /**
     * The instance type for which to reserve capacity.
     */
    instanceType: pulumi.Input<string | enums.ec2.InstanceType>;
    /**
     * The Amazon Resource Name (ARN) of the Outpost on which to create the Capacity Reservation.
     */
    outpostArn?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the cluster placement group in which to create the Capacity Reservation.
     */
    placementGroupArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Indicates the tenancy of the Capacity Reservation. Specify either `default` or `dedicated`.
     */
    tenancy?: pulumi.Input<string | enums.ec2.Tenancy>;
}
