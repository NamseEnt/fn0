import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CloudFormation Stack resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const network = new aws.cloudformation.Stack("network", {
 *     name: "networking-stack",
 *     parameters: {
 *         VPCCidr: "10.0.0.0/16",
 *     },
 *     templateBody: JSON.stringify({
 *         Parameters: {
 *             VPCCidr: {
 *                 Type: "String",
 *                 Default: "10.0.0.0/16",
 *                 Description: "Enter the CIDR block for the VPC. Default is 10.0.0.0/16.",
 *             },
 *         },
 *         Resources: {
 *             myVpc: {
 *                 Type: "AWS::EC2::VPC",
 *                 Properties: {
 *                     CidrBlock: {
 *                         Ref: "VPCCidr",
 *                     },
 *                     Tags: [{
 *                         Key: "Name",
 *                         Value: "Primary_CF_VPC",
 *                     }],
 *                 },
 *             },
 *         },
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Cloudformation Stacks using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudformation/stack:Stack stack networking-stack
 * ```
 */
export declare class Stack extends pulumi.CustomResource {
    /**
     * Get an existing Stack resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: StackState, opts?: pulumi.CustomResourceOptions): Stack;
    /**
     * Returns true if the given object is an instance of Stack.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Stack;
    /**
     * A list of capabilities.
     * Valid values: `CAPABILITY_IAM`, `CAPABILITY_NAMED_IAM`, or `CAPABILITY_AUTO_EXPAND`
     */
    readonly capabilities: pulumi.Output<string[] | undefined>;
    /**
     * Set to true to disable rollback of the stack if stack creation failed.
     * Conflicts with `onFailure`.
     */
    readonly disableRollback: pulumi.Output<boolean | undefined>;
    /**
     * The ARN of an IAM role that AWS CloudFormation assumes to create the stack. If you don't specify a value, AWS CloudFormation uses the role that was previously associated with the stack. If no role is available, AWS CloudFormation uses a temporary session that is generated from your user credentials.
     */
    readonly iamRoleArn: pulumi.Output<string | undefined>;
    /**
     * Stack name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * A list of SNS topic ARNs to publish stack related events.
     */
    readonly notificationArns: pulumi.Output<string[] | undefined>;
    /**
     * Action to be taken if stack creation fails. This must be
     * one of: `DO_NOTHING`, `ROLLBACK`, or `DELETE`. Conflicts with `disableRollback`.
     */
    readonly onFailure: pulumi.Output<string | undefined>;
    /**
     * A map of outputs from the stack.
     */
    readonly outputs: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * A map of Parameter structures that specify input parameters for the stack.
     */
    readonly parameters: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Structure containing the stack policy body.
     * Conflicts w/ `policyUrl`.
     */
    readonly policyBody: pulumi.Output<string>;
    /**
     * Location of a file containing the stack policy.
     * Conflicts w/ `policyBody`.
     */
    readonly policyUrl: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of resource tags to associate with this stack. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Structure containing the template body (max size: 51,200 bytes).
     */
    readonly templateBody: pulumi.Output<string>;
    /**
     * Location of a file containing the template body (max size: 460,800 bytes).
     */
    readonly templateUrl: pulumi.Output<string | undefined>;
    /**
     * The amount of time that can pass before the stack status becomes `CREATE_FAILED`.
     */
    readonly timeoutInMinutes: pulumi.Output<number | undefined>;
    /**
     * Create a Stack resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: StackArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Stack resources.
 */
export interface StackState {
    /**
     * A list of capabilities.
     * Valid values: `CAPABILITY_IAM`, `CAPABILITY_NAMED_IAM`, or `CAPABILITY_AUTO_EXPAND`
     */
    capabilities?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Set to true to disable rollback of the stack if stack creation failed.
     * Conflicts with `onFailure`.
     */
    disableRollback?: pulumi.Input<boolean>;
    /**
     * The ARN of an IAM role that AWS CloudFormation assumes to create the stack. If you don't specify a value, AWS CloudFormation uses the role that was previously associated with the stack. If no role is available, AWS CloudFormation uses a temporary session that is generated from your user credentials.
     */
    iamRoleArn?: pulumi.Input<string>;
    /**
     * Stack name.
     */
    name?: pulumi.Input<string>;
    /**
     * A list of SNS topic ARNs to publish stack related events.
     */
    notificationArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Action to be taken if stack creation fails. This must be
     * one of: `DO_NOTHING`, `ROLLBACK`, or `DELETE`. Conflicts with `disableRollback`.
     */
    onFailure?: pulumi.Input<string>;
    /**
     * A map of outputs from the stack.
     */
    outputs?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of Parameter structures that specify input parameters for the stack.
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Structure containing the stack policy body.
     * Conflicts w/ `policyUrl`.
     */
    policyBody?: pulumi.Input<string>;
    /**
     * Location of a file containing the stack policy.
     * Conflicts w/ `policyBody`.
     */
    policyUrl?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of resource tags to associate with this stack. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Structure containing the template body (max size: 51,200 bytes).
     */
    templateBody?: pulumi.Input<string>;
    /**
     * Location of a file containing the template body (max size: 460,800 bytes).
     */
    templateUrl?: pulumi.Input<string>;
    /**
     * The amount of time that can pass before the stack status becomes `CREATE_FAILED`.
     */
    timeoutInMinutes?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a Stack resource.
 */
export interface StackArgs {
    /**
     * A list of capabilities.
     * Valid values: `CAPABILITY_IAM`, `CAPABILITY_NAMED_IAM`, or `CAPABILITY_AUTO_EXPAND`
     */
    capabilities?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Set to true to disable rollback of the stack if stack creation failed.
     * Conflicts with `onFailure`.
     */
    disableRollback?: pulumi.Input<boolean>;
    /**
     * The ARN of an IAM role that AWS CloudFormation assumes to create the stack. If you don't specify a value, AWS CloudFormation uses the role that was previously associated with the stack. If no role is available, AWS CloudFormation uses a temporary session that is generated from your user credentials.
     */
    iamRoleArn?: pulumi.Input<string>;
    /**
     * Stack name.
     */
    name?: pulumi.Input<string>;
    /**
     * A list of SNS topic ARNs to publish stack related events.
     */
    notificationArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Action to be taken if stack creation fails. This must be
     * one of: `DO_NOTHING`, `ROLLBACK`, or `DELETE`. Conflicts with `disableRollback`.
     */
    onFailure?: pulumi.Input<string>;
    /**
     * A map of Parameter structures that specify input parameters for the stack.
     */
    parameters?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Structure containing the stack policy body.
     * Conflicts w/ `policyUrl`.
     */
    policyBody?: pulumi.Input<string>;
    /**
     * Location of a file containing the stack policy.
     * Conflicts w/ `policyBody`.
     */
    policyUrl?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of resource tags to associate with this stack. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Structure containing the template body (max size: 51,200 bytes).
     */
    templateBody?: pulumi.Input<string>;
    /**
     * Location of a file containing the template body (max size: 460,800 bytes).
     */
    templateUrl?: pulumi.Input<string>;
    /**
     * The amount of time that can pass before the stack status becomes `CREATE_FAILED`.
     */
    timeoutInMinutes?: pulumi.Input<number>;
}
