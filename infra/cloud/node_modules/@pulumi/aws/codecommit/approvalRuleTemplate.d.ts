import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CodeCommit Approval Rule Template Resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.codecommit.ApprovalRuleTemplate("example", {
 *     name: "MyExampleApprovalRuleTemplate",
 *     description: "This is an example approval rule template",
 *     content: JSON.stringify({
 *         Version: "2018-11-08",
 *         DestinationReferences: ["refs/heads/master"],
 *         Statements: [{
 *             Type: "Approvers",
 *             NumberOfApprovalsNeeded: 2,
 *             ApprovalPoolMembers: ["arn:aws:sts::123456789012:assumed-role/CodeCommitReview/*"],
 *         }],
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CodeCommit approval rule templates using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:codecommit/approvalRuleTemplate:ApprovalRuleTemplate imported ExistingApprovalRuleTemplateName
 * ```
 */
export declare class ApprovalRuleTemplate extends pulumi.CustomResource {
    /**
     * Get an existing ApprovalRuleTemplate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ApprovalRuleTemplateState, opts?: pulumi.CustomResourceOptions): ApprovalRuleTemplate;
    /**
     * Returns true if the given object is an instance of ApprovalRuleTemplate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ApprovalRuleTemplate;
    /**
     * The ID of the approval rule template
     */
    readonly approvalRuleTemplateId: pulumi.Output<string>;
    /**
     * The content of the approval rule template. Maximum of 3000 characters.
     */
    readonly content: pulumi.Output<string>;
    /**
     * The date the approval rule template was created, in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8).
     */
    readonly creationDate: pulumi.Output<string>;
    /**
     * The description of the approval rule template. Maximum of 1000 characters.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The date the approval rule template was most recently changed, in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8).
     */
    readonly lastModifiedDate: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the user who made the most recent changes to the approval rule template.
     */
    readonly lastModifiedUser: pulumi.Output<string>;
    /**
     * The name for the approval rule template. Maximum of 100 characters.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The SHA-256 hash signature for the content of the approval rule template.
     */
    readonly ruleContentSha256: pulumi.Output<string>;
    /**
     * Create a ApprovalRuleTemplate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ApprovalRuleTemplateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ApprovalRuleTemplate resources.
 */
export interface ApprovalRuleTemplateState {
    /**
     * The ID of the approval rule template
     */
    approvalRuleTemplateId?: pulumi.Input<string>;
    /**
     * The content of the approval rule template. Maximum of 3000 characters.
     */
    content?: pulumi.Input<string>;
    /**
     * The date the approval rule template was created, in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8).
     */
    creationDate?: pulumi.Input<string>;
    /**
     * The description of the approval rule template. Maximum of 1000 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * The date the approval rule template was most recently changed, in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8).
     */
    lastModifiedDate?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the user who made the most recent changes to the approval rule template.
     */
    lastModifiedUser?: pulumi.Input<string>;
    /**
     * The name for the approval rule template. Maximum of 100 characters.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The SHA-256 hash signature for the content of the approval rule template.
     */
    ruleContentSha256?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ApprovalRuleTemplate resource.
 */
export interface ApprovalRuleTemplateArgs {
    /**
     * The content of the approval rule template. Maximum of 3000 characters.
     */
    content: pulumi.Input<string>;
    /**
     * The description of the approval rule template. Maximum of 1000 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * The name for the approval rule template. Maximum of 100 characters.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
