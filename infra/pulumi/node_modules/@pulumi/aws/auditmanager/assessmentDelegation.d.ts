import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Audit Manager Assessment Delegation.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.auditmanager.AssessmentDelegation("example", {
 *     assessmentId: exampleAwsAuditmanagerAssessment.id,
 *     roleArn: exampleAwsIamRole.arn,
 *     roleType: "RESOURCE_OWNER",
 *     controlSetId: "example",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Audit Manager Assessment Delegation using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:auditmanager/assessmentDelegation:AssessmentDelegation example abcdef-123456,arn:aws:iam::123456789012:role/example,example
 * ```
 */
export declare class AssessmentDelegation extends pulumi.CustomResource {
    /**
     * Get an existing AssessmentDelegation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AssessmentDelegationState, opts?: pulumi.CustomResourceOptions): AssessmentDelegation;
    /**
     * Returns true if the given object is an instance of AssessmentDelegation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AssessmentDelegation;
    /**
     * Identifier for the assessment.
     */
    readonly assessmentId: pulumi.Output<string>;
    /**
     * Comment describing the delegation request.
     */
    readonly comment: pulumi.Output<string | undefined>;
    /**
     * Assessment control set name. This value is the control set name used during assessment creation (not the AWS-generated ID). The `_id` suffix on this attribute has been preserved to be consistent with the underlying AWS API.
     */
    readonly controlSetId: pulumi.Output<string>;
    /**
     * Unique identifier for the delegation.
     */
    readonly delegationId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the IAM role.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * Type of customer persona. For assessment delegation, type must always be `RESOURCE_OWNER`.
     *
     * The following arguments are optional:
     */
    readonly roleType: pulumi.Output<string>;
    /**
     * Status of the delegation.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Create a AssessmentDelegation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AssessmentDelegationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AssessmentDelegation resources.
 */
export interface AssessmentDelegationState {
    /**
     * Identifier for the assessment.
     */
    assessmentId?: pulumi.Input<string>;
    /**
     * Comment describing the delegation request.
     */
    comment?: pulumi.Input<string>;
    /**
     * Assessment control set name. This value is the control set name used during assessment creation (not the AWS-generated ID). The `_id` suffix on this attribute has been preserved to be consistent with the underlying AWS API.
     */
    controlSetId?: pulumi.Input<string>;
    /**
     * Unique identifier for the delegation.
     */
    delegationId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the IAM role.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * Type of customer persona. For assessment delegation, type must always be `RESOURCE_OWNER`.
     *
     * The following arguments are optional:
     */
    roleType?: pulumi.Input<string>;
    /**
     * Status of the delegation.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AssessmentDelegation resource.
 */
export interface AssessmentDelegationArgs {
    /**
     * Identifier for the assessment.
     */
    assessmentId: pulumi.Input<string>;
    /**
     * Comment describing the delegation request.
     */
    comment?: pulumi.Input<string>;
    /**
     * Assessment control set name. This value is the control set name used during assessment creation (not the AWS-generated ID). The `_id` suffix on this attribute has been preserved to be consistent with the underlying AWS API.
     */
    controlSetId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the IAM role.
     */
    roleArn: pulumi.Input<string>;
    /**
     * Type of customer persona. For assessment delegation, type must always be `RESOURCE_OWNER`.
     *
     * The following arguments are optional:
     */
    roleType: pulumi.Input<string>;
}
