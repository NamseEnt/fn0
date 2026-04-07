import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages AWS Compute Optimizer enrollment status.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.computeoptimizer.EnrollmentStatus("example", {status: "Active"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import enrollment status using the account ID. For example:
 *
 * ```sh
 * $ pulumi import aws:computeoptimizer/enrollmentStatus:EnrollmentStatus example 123456789012
 * ```
 */
export declare class EnrollmentStatus extends pulumi.CustomResource {
    /**
     * Get an existing EnrollmentStatus resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EnrollmentStatusState, opts?: pulumi.CustomResourceOptions): EnrollmentStatus;
    /**
     * Returns true if the given object is an instance of EnrollmentStatus.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EnrollmentStatus;
    /**
     * Whether to enroll member accounts of the organization if the account is the management account of an organization. Default is `false`.
     */
    readonly includeMemberAccounts: pulumi.Output<boolean>;
    /**
     * The count of organization member accounts that are opted in to the service, if your account is an organization management account.
     */
    readonly numberOfMemberAccountsOptedIn: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The enrollment status of the account. Valid values: `Active`, `Inactive`.
     */
    readonly status: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.computeoptimizer.EnrollmentStatusTimeouts | undefined>;
    /**
     * Create a EnrollmentStatus resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EnrollmentStatusArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EnrollmentStatus resources.
 */
export interface EnrollmentStatusState {
    /**
     * Whether to enroll member accounts of the organization if the account is the management account of an organization. Default is `false`.
     */
    includeMemberAccounts?: pulumi.Input<boolean>;
    /**
     * The count of organization member accounts that are opted in to the service, if your account is an organization management account.
     */
    numberOfMemberAccountsOptedIn?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The enrollment status of the account. Valid values: `Active`, `Inactive`.
     */
    status?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.computeoptimizer.EnrollmentStatusTimeouts>;
}
/**
 * The set of arguments for constructing a EnrollmentStatus resource.
 */
export interface EnrollmentStatusArgs {
    /**
     * Whether to enroll member accounts of the organization if the account is the management account of an organization. Default is `false`.
     */
    includeMemberAccounts?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The enrollment status of the account. Valid values: `Active`, `Inactive`.
     */
    status: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.computeoptimizer.EnrollmentStatusTimeouts>;
}
