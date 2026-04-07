import * as pulumi from "@pulumi/pulumi";
/**
 * Disable/enable Security Hub standards control in the current region.
 *
 * The `aws.securityhub.StandardsControl` behaves differently from normal resources, in that
 * Pulumi does not _create_ this resource, but instead "adopts" it
 * into management. When you _delete_ this resource configuration, Pulumi "abandons" resource as is and just removes it from the state.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.securityhub.Account("example", {});
 * const cisAwsFoundationsBenchmark = new aws.securityhub.StandardsSubscription("cis_aws_foundations_benchmark", {standardsArn: "arn:aws:securityhub:::ruleset/cis-aws-foundations-benchmark/v/1.2.0"}, {
 *     dependsOn: [example],
 * });
 * const ensureIamPasswordPolicyPreventsPasswordReuse = new aws.securityhub.StandardsControl("ensure_iam_password_policy_prevents_password_reuse", {
 *     standardsControlArn: "arn:aws:securityhub:us-east-1:111111111111:control/cis-aws-foundations-benchmark/v/1.2.0/1.10",
 *     controlStatus: "DISABLED",
 *     disabledReason: "We handle password policies within Okta",
 * }, {
 *     dependsOn: [cisAwsFoundationsBenchmark],
 * });
 * ```
 */
export declare class StandardsControl extends pulumi.CustomResource {
    /**
     * Get an existing StandardsControl resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: StandardsControlState, opts?: pulumi.CustomResourceOptions): StandardsControl;
    /**
     * Returns true if the given object is an instance of StandardsControl.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is StandardsControl;
    /**
     * The identifier of the security standard control.
     */
    readonly controlId: pulumi.Output<string>;
    /**
     * The control status could be `ENABLED` or `DISABLED`. You have to specify `disabledReason` argument for `DISABLED` control status.
     */
    readonly controlStatus: pulumi.Output<string>;
    /**
     * The date and time that the status of the security standard control was most recently updated.
     */
    readonly controlStatusUpdatedAt: pulumi.Output<string>;
    /**
     * The standard control longer description. Provides information about what the control is checking for.
     */
    readonly description: pulumi.Output<string>;
    /**
     * A description of the reason why you are disabling a security standard control. If you specify this attribute, `controlStatus` will be set to `DISABLED` automatically.
     */
    readonly disabledReason: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The list of requirements that are related to this control.
     */
    readonly relatedRequirements: pulumi.Output<string[]>;
    /**
     * A link to remediation information for the control in the Security Hub user documentation.
     */
    readonly remediationUrl: pulumi.Output<string>;
    /**
     * The severity of findings generated from this security standard control.
     */
    readonly severityRating: pulumi.Output<string>;
    /**
     * The standards control ARN. See the AWS documentation for how to list existing controls using [`get-enabled-standards`](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/securityhub/get-enabled-standards.html) and [`describe-standards-controls`](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/securityhub/describe-standards-controls.html).
     */
    readonly standardsControlArn: pulumi.Output<string>;
    /**
     * The standard control title.
     */
    readonly title: pulumi.Output<string>;
    /**
     * Create a StandardsControl resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: StandardsControlArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering StandardsControl resources.
 */
export interface StandardsControlState {
    /**
     * The identifier of the security standard control.
     */
    controlId?: pulumi.Input<string>;
    /**
     * The control status could be `ENABLED` or `DISABLED`. You have to specify `disabledReason` argument for `DISABLED` control status.
     */
    controlStatus?: pulumi.Input<string>;
    /**
     * The date and time that the status of the security standard control was most recently updated.
     */
    controlStatusUpdatedAt?: pulumi.Input<string>;
    /**
     * The standard control longer description. Provides information about what the control is checking for.
     */
    description?: pulumi.Input<string>;
    /**
     * A description of the reason why you are disabling a security standard control. If you specify this attribute, `controlStatus` will be set to `DISABLED` automatically.
     */
    disabledReason?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The list of requirements that are related to this control.
     */
    relatedRequirements?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A link to remediation information for the control in the Security Hub user documentation.
     */
    remediationUrl?: pulumi.Input<string>;
    /**
     * The severity of findings generated from this security standard control.
     */
    severityRating?: pulumi.Input<string>;
    /**
     * The standards control ARN. See the AWS documentation for how to list existing controls using [`get-enabled-standards`](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/securityhub/get-enabled-standards.html) and [`describe-standards-controls`](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/securityhub/describe-standards-controls.html).
     */
    standardsControlArn?: pulumi.Input<string>;
    /**
     * The standard control title.
     */
    title?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a StandardsControl resource.
 */
export interface StandardsControlArgs {
    /**
     * The control status could be `ENABLED` or `DISABLED`. You have to specify `disabledReason` argument for `DISABLED` control status.
     */
    controlStatus: pulumi.Input<string>;
    /**
     * A description of the reason why you are disabling a security standard control. If you specify this attribute, `controlStatus` will be set to `DISABLED` automatically.
     */
    disabledReason?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The standards control ARN. See the AWS documentation for how to list existing controls using [`get-enabled-standards`](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/securityhub/get-enabled-standards.html) and [`describe-standards-controls`](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/securityhub/describe-standards-controls.html).
     */
    standardsControlArn: pulumi.Input<string>;
}
