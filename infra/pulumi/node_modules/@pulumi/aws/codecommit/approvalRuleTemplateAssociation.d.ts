import * as pulumi from "@pulumi/pulumi";
/**
 * Associates a CodeCommit Approval Rule Template with a Repository.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.codecommit.ApprovalRuleTemplateAssociation("example", {
 *     approvalRuleTemplateName: exampleAwsCodecommitApprovalRuleTemplate.name,
 *     repositoryName: exampleAwsCodecommitRepository.repositoryName,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CodeCommit approval rule template associations using the `approval_rule_template_name` and `repository_name` separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:codecommit/approvalRuleTemplateAssociation:ApprovalRuleTemplateAssociation example approver-rule-for-example,MyExampleRepo
 * ```
 */
export declare class ApprovalRuleTemplateAssociation extends pulumi.CustomResource {
    /**
     * Get an existing ApprovalRuleTemplateAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ApprovalRuleTemplateAssociationState, opts?: pulumi.CustomResourceOptions): ApprovalRuleTemplateAssociation;
    /**
     * Returns true if the given object is an instance of ApprovalRuleTemplateAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ApprovalRuleTemplateAssociation;
    /**
     * The name for the approval rule template.
     */
    readonly approvalRuleTemplateName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name of the repository that you want to associate with the template.
     */
    readonly repositoryName: pulumi.Output<string>;
    /**
     * Create a ApprovalRuleTemplateAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ApprovalRuleTemplateAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ApprovalRuleTemplateAssociation resources.
 */
export interface ApprovalRuleTemplateAssociationState {
    /**
     * The name for the approval rule template.
     */
    approvalRuleTemplateName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the repository that you want to associate with the template.
     */
    repositoryName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ApprovalRuleTemplateAssociation resource.
 */
export interface ApprovalRuleTemplateAssociationArgs {
    /**
     * The name for the approval rule template.
     */
    approvalRuleTemplateName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the repository that you want to associate with the template.
     */
    repositoryName: pulumi.Input<string>;
}
