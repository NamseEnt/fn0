import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing AWS Cost Optimization Hub Enrollment Status.
 *
 * > **TIP:** The Cost Optimization Hub only has a `us-east-1` endpoint. However, you can access the service globally with the AWS Provider from other regions. Other tools, such as the [AWS CLI](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/cost-optimization-hub/index.html), may require you to specify the `us-east-1` region when using the service.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.costoptimizationhub.EnrollmentStatus("example", {});
 * ```
 *
 * ### Usage with all the arguments
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.costoptimizationhub.EnrollmentStatus("example", {includeMemberAccounts: true});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Cost Optimization Hub Enrollment Status using your AWS account ID. For example:
 *
 * ```sh
 * $ pulumi import aws:costoptimizationhub/enrollmentStatus:EnrollmentStatus example 111222333444
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
     * Flag to enroll member accounts of the organization if the account is the management account. No drift detection is currently supported for this argument. Default value is `false`.
     */
    readonly includeMemberAccounts: pulumi.Output<boolean>;
    readonly status: pulumi.Output<string>;
    /**
     * Create a EnrollmentStatus resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: EnrollmentStatusArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EnrollmentStatus resources.
 */
export interface EnrollmentStatusState {
    /**
     * Flag to enroll member accounts of the organization if the account is the management account. No drift detection is currently supported for this argument. Default value is `false`.
     */
    includeMemberAccounts?: pulumi.Input<boolean>;
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EnrollmentStatus resource.
 */
export interface EnrollmentStatusArgs {
    /**
     * Flag to enroll member accounts of the organization if the account is the management account. No drift detection is currently supported for this argument. Default value is `false`.
     */
    includeMemberAccounts?: pulumi.Input<boolean>;
}
