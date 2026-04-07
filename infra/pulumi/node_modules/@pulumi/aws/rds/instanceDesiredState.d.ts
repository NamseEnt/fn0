import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS RDS (Relational Database) RDS Instance State.
 *
 * > Destruction of this resource is a no-op and **will not** modify the instance state
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.rds.InstanceDesiredState("test", {
 *     identifier: testAwsDbInstance.identifier,
 *     state: "available",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import RDS (Relational Database) RDS Instance State using the `identifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:rds/instanceDesiredState:InstanceDesiredState example rds_instance_state-id-12345678
 * ```
 */
export declare class InstanceDesiredState extends pulumi.CustomResource {
    /**
     * Get an existing InstanceDesiredState resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InstanceDesiredStateState, opts?: pulumi.CustomResourceOptions): InstanceDesiredState;
    /**
     * Returns true if the given object is an instance of InstanceDesiredState.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is InstanceDesiredState;
    /**
     * DB Instance Identifier
     */
    readonly identifier: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configured state of the DB Instance. Valid values are `available` and `stopped`.
     */
    readonly state: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.rds.InstanceDesiredStateTimeouts | undefined>;
    /**
     * Create a InstanceDesiredState resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InstanceDesiredStateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering InstanceDesiredState resources.
 */
export interface InstanceDesiredStateState {
    /**
     * DB Instance Identifier
     */
    identifier?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configured state of the DB Instance. Valid values are `available` and `stopped`.
     */
    state?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.rds.InstanceDesiredStateTimeouts>;
}
/**
 * The set of arguments for constructing a InstanceDesiredState resource.
 */
export interface InstanceDesiredStateArgs {
    /**
     * DB Instance Identifier
     */
    identifier: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configured state of the DB Instance. Valid values are `available` and `stopped`.
     */
    state: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.rds.InstanceDesiredStateTimeouts>;
}
