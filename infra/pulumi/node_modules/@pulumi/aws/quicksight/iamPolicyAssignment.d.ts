import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS QuickSight IAM Policy Assignment.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.IamPolicyAssignment("example", {
 *     assignmentName: "example",
 *     assignmentStatus: "ENABLED",
 *     policyArn: exampleAwsIamPolicy.arn,
 *     identities: {
 *         users: [exampleAwsQuicksightUser.userName],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import QuickSight IAM Policy Assignment using the AWS account ID, namespace, and assignment name separated by commas (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/iamPolicyAssignment:IamPolicyAssignment example 123456789012,default,example
 * ```
 */
export declare class IamPolicyAssignment extends pulumi.CustomResource {
    /**
     * Get an existing IamPolicyAssignment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: IamPolicyAssignmentState, opts?: pulumi.CustomResourceOptions): IamPolicyAssignment;
    /**
     * Returns true if the given object is an instance of IamPolicyAssignment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is IamPolicyAssignment;
    /**
     * Assignment ID.
     */
    readonly assignmentId: pulumi.Output<string>;
    /**
     * Name of the assignment.
     */
    readonly assignmentName: pulumi.Output<string>;
    /**
     * Status of the assignment. Valid values are `ENABLED`, `DISABLED`, and `DRAFT`.
     *
     * The following arguments are optional:
     */
    readonly assignmentStatus: pulumi.Output<string>;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * Amazon QuickSight users, groups, or both to assign the policy to. See `identities` block.
     */
    readonly identities: pulumi.Output<outputs.quicksight.IamPolicyAssignmentIdentities | undefined>;
    /**
     * Namespace that contains the assignment. Defaults to `default`.
     */
    readonly namespace: pulumi.Output<string>;
    /**
     * ARN of the IAM policy to apply to the Amazon QuickSight users and groups specified in this assignment.
     */
    readonly policyArn: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a IamPolicyAssignment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: IamPolicyAssignmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering IamPolicyAssignment resources.
 */
export interface IamPolicyAssignmentState {
    /**
     * Assignment ID.
     */
    assignmentId?: pulumi.Input<string>;
    /**
     * Name of the assignment.
     */
    assignmentName?: pulumi.Input<string>;
    /**
     * Status of the assignment. Valid values are `ENABLED`, `DISABLED`, and `DRAFT`.
     *
     * The following arguments are optional:
     */
    assignmentStatus?: pulumi.Input<string>;
    awsAccountId?: pulumi.Input<string>;
    /**
     * Amazon QuickSight users, groups, or both to assign the policy to. See `identities` block.
     */
    identities?: pulumi.Input<inputs.quicksight.IamPolicyAssignmentIdentities>;
    /**
     * Namespace that contains the assignment. Defaults to `default`.
     */
    namespace?: pulumi.Input<string>;
    /**
     * ARN of the IAM policy to apply to the Amazon QuickSight users and groups specified in this assignment.
     */
    policyArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a IamPolicyAssignment resource.
 */
export interface IamPolicyAssignmentArgs {
    /**
     * Name of the assignment.
     */
    assignmentName: pulumi.Input<string>;
    /**
     * Status of the assignment. Valid values are `ENABLED`, `DISABLED`, and `DRAFT`.
     *
     * The following arguments are optional:
     */
    assignmentStatus: pulumi.Input<string>;
    awsAccountId?: pulumi.Input<string>;
    /**
     * Amazon QuickSight users, groups, or both to assign the policy to. See `identities` block.
     */
    identities?: pulumi.Input<inputs.quicksight.IamPolicyAssignmentIdentities>;
    /**
     * Namespace that contains the assignment. Defaults to `default`.
     */
    namespace?: pulumi.Input<string>;
    /**
     * ARN of the IAM policy to apply to the Amazon QuickSight users and groups specified in this assignment.
     */
    policyArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
