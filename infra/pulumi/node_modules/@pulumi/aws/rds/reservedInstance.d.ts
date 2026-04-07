import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an RDS DB Reserved Instance.
 *
 * > **NOTE:** Once created, a reservation is valid for the `duration` of the provided `offeringId` and cannot be deleted. Performing a `destroy` will only remove the resource from state. For more information see [RDS Reserved Instances Documentation](https://aws.amazon.com/rds/reserved-instances/) and [PurchaseReservedDBInstancesOffering](https://docs.aws.amazon.com/AmazonRDS/latest/APIReference/API_PurchaseReservedDBInstancesOffering.html).
 *
 * > **NOTE:** Due to the expense of testing this resource, we provide it as best effort. If you find it useful, and have the ability to help test or notice issues, consider reaching out to us on GitHub.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.rds.getReservedInstanceOffering({
 *     dbInstanceClass: "db.t2.micro",
 *     duration: 31536000,
 *     multiAz: false,
 *     offeringType: "All Upfront",
 *     productDescription: "mysql",
 * });
 * const example = new aws.rds.ReservedInstance("example", {
 *     offeringId: test.then(test => test.offeringId),
 *     reservationId: "optionalCustomReservationID",
 *     instanceCount: 3,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import RDS DB Instance Reservations using the `instance_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:rds/reservedInstance:ReservedInstance reservation_instance CustomReservationID
 * ```
 */
export declare class ReservedInstance extends pulumi.CustomResource {
    /**
     * Get an existing ReservedInstance resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ReservedInstanceState, opts?: pulumi.CustomResourceOptions): ReservedInstance;
    /**
     * Returns true if the given object is an instance of ReservedInstance.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ReservedInstance;
    /**
     * ARN for the reserved DB instance.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Currency code for the reserved DB instance.
     */
    readonly currencyCode: pulumi.Output<string>;
    /**
     * DB instance class for the reserved DB instance.
     */
    readonly dbInstanceClass: pulumi.Output<string>;
    /**
     * Duration of the reservation in seconds.
     */
    readonly duration: pulumi.Output<number>;
    /**
     * Fixed price charged for this reserved DB instance.
     */
    readonly fixedPrice: pulumi.Output<number>;
    /**
     * Number of instances to reserve. Default value is `1`.
     */
    readonly instanceCount: pulumi.Output<number | undefined>;
    /**
     * Unique identifier for the lease associated with the reserved DB instance. Amazon Web Services Support might request the lease ID for an issue related to a reserved DB instance.
     */
    readonly leaseId: pulumi.Output<string>;
    /**
     * Whether the reservation applies to Multi-AZ deployments.
     */
    readonly multiAz: pulumi.Output<boolean>;
    /**
     * ID of the Reserved DB instance offering to purchase. To determine an `offeringId`, see the `aws.rds.getReservedInstanceOffering` data source.
     *
     * The following arguments are optional:
     */
    readonly offeringId: pulumi.Output<string>;
    /**
     * Offering type of this reserved DB instance.
     */
    readonly offeringType: pulumi.Output<string>;
    /**
     * Description of the reserved DB instance.
     */
    readonly productDescription: pulumi.Output<string>;
    /**
     * Recurring price charged to run this reserved DB instance.
     */
    readonly recurringCharges: pulumi.Output<outputs.rds.ReservedInstanceRecurringCharge[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Customer-specified identifier to track this reservation.
     */
    readonly reservationId: pulumi.Output<string | undefined>;
    /**
     * Time the reservation started.
     */
    readonly startTime: pulumi.Output<string>;
    /**
     * State of the reserved DB instance.
     */
    readonly state: pulumi.Output<string>;
    /**
     * Map of tags to assign to the DB reservation. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Hourly price charged for this reserved DB instance.
     */
    readonly usagePrice: pulumi.Output<number>;
    /**
     * Create a ReservedInstance resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ReservedInstanceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ReservedInstance resources.
 */
export interface ReservedInstanceState {
    /**
     * ARN for the reserved DB instance.
     */
    arn?: pulumi.Input<string>;
    /**
     * Currency code for the reserved DB instance.
     */
    currencyCode?: pulumi.Input<string>;
    /**
     * DB instance class for the reserved DB instance.
     */
    dbInstanceClass?: pulumi.Input<string>;
    /**
     * Duration of the reservation in seconds.
     */
    duration?: pulumi.Input<number>;
    /**
     * Fixed price charged for this reserved DB instance.
     */
    fixedPrice?: pulumi.Input<number>;
    /**
     * Number of instances to reserve. Default value is `1`.
     */
    instanceCount?: pulumi.Input<number>;
    /**
     * Unique identifier for the lease associated with the reserved DB instance. Amazon Web Services Support might request the lease ID for an issue related to a reserved DB instance.
     */
    leaseId?: pulumi.Input<string>;
    /**
     * Whether the reservation applies to Multi-AZ deployments.
     */
    multiAz?: pulumi.Input<boolean>;
    /**
     * ID of the Reserved DB instance offering to purchase. To determine an `offeringId`, see the `aws.rds.getReservedInstanceOffering` data source.
     *
     * The following arguments are optional:
     */
    offeringId?: pulumi.Input<string>;
    /**
     * Offering type of this reserved DB instance.
     */
    offeringType?: pulumi.Input<string>;
    /**
     * Description of the reserved DB instance.
     */
    productDescription?: pulumi.Input<string>;
    /**
     * Recurring price charged to run this reserved DB instance.
     */
    recurringCharges?: pulumi.Input<pulumi.Input<inputs.rds.ReservedInstanceRecurringCharge>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Customer-specified identifier to track this reservation.
     */
    reservationId?: pulumi.Input<string>;
    /**
     * Time the reservation started.
     */
    startTime?: pulumi.Input<string>;
    /**
     * State of the reserved DB instance.
     */
    state?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the DB reservation. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * Hourly price charged for this reserved DB instance.
     */
    usagePrice?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a ReservedInstance resource.
 */
export interface ReservedInstanceArgs {
    /**
     * Number of instances to reserve. Default value is `1`.
     */
    instanceCount?: pulumi.Input<number>;
    /**
     * ID of the Reserved DB instance offering to purchase. To determine an `offeringId`, see the `aws.rds.getReservedInstanceOffering` data source.
     *
     * The following arguments are optional:
     */
    offeringId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Customer-specified identifier to track this reservation.
     */
    reservationId?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the DB reservation. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
