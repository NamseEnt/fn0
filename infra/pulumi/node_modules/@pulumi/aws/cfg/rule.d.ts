import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS Config Rule.
 *
 * > **Note:** Config Rule requires an existing Configuration Recorder to be present. Use of `dependsOn` is recommended (as shown below) to avoid race conditions.
 *
 * ## Example Usage
 *
 * ### AWS Managed Rules
 *
 * AWS managed rules can be used by setting the source owner to `AWS` and the source identifier to the name of the managed rule. More information about AWS managed rules can be found in the [AWS Config Developer Guide](https://docs.aws.amazon.com/config/latest/developerguide/evaluate-config_use-managed-rules.html).
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const assumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["config.amazonaws.com"],
 *         }],
 *         actions: ["sts:AssumeRole"],
 *     }],
 * });
 * const rRole = new aws.iam.Role("r", {
 *     name: "my-awsconfig-role",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const foo = new aws.cfg.Recorder("foo", {
 *     name: "example",
 *     roleArn: rRole.arn,
 * });
 * const r = new aws.cfg.Rule("r", {
 *     name: "example",
 *     source: {
 *         owner: "AWS",
 *         sourceIdentifier: "S3_BUCKET_VERSIONING_ENABLED",
 *     },
 * }, {
 *     dependsOn: [foo],
 * });
 * const p = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         actions: ["config:Put*"],
 *         resources: ["*"],
 *     }],
 * });
 * const pRolePolicy = new aws.iam.RolePolicy("p", {
 *     name: "my-awsconfig-policy",
 *     role: rRole.id,
 *     policy: p.then(p => p.json),
 * });
 * ```
 *
 * ### Custom Rules
 *
 * Custom rules can be used by setting the source owner to `CUSTOM_LAMBDA` and the source identifier to the Amazon Resource Name (ARN) of the Lambda Function. The AWS Config service must have permissions to invoke the Lambda Function, e.g., via the `aws.lambda.Permission` resource. More information about custom rules can be found in the [AWS Config Developer Guide](https://docs.aws.amazon.com/config/latest/developerguide/evaluate-config_develop-rules.html).
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cfg.Recorder("example", {});
 * const exampleFunction = new aws.lambda.Function("example", {});
 * const examplePermission = new aws.lambda.Permission("example", {
 *     action: "lambda:InvokeFunction",
 *     "function": exampleFunction.arn,
 *     principal: "config.amazonaws.com",
 *     statementId: "AllowExecutionFromConfig",
 * });
 * const exampleRule = new aws.cfg.Rule("example", {source: {
 *     owner: "CUSTOM_LAMBDA",
 *     sourceIdentifier: exampleFunction.arn,
 * }}, {
 *     dependsOn: [
 *         example,
 *         examplePermission,
 *     ],
 * });
 * ```
 *
 * ### Custom Policies
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cfg.Rule("example", {
 *     name: "example",
 *     source: {
 *         owner: "CUSTOM_POLICY",
 *         sourceDetails: [{
 *             messageType: "ConfigurationItemChangeNotification",
 *         }],
 *         customPolicyDetails: {
 *             policyRuntime: "guard-2.x.x",
 *             policyText: `\\t  rule tableisactive when
 * \\t\\t  resourceType == \\"AWS::DynamoDB::Table\\" {
 * \\t\\t  configuration.tableStatus == ['ACTIVE']
 * \\t  }
 * \\t
 * \\t  rule checkcompliance when
 * \\t\\t  resourceType == \\"AWS::DynamoDB::Table\\"
 * \\t\\t  tableisactive {
 * \\t\\t\\t  supplementaryConfiguration.ContinuousBackupsDescription.pointInTimeRecoveryDescription.pointInTimeRecoveryStatus == \\"ENABLED\\"
 * \\t  }
 * `,
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Config Rule using the name. For example:
 *
 * ```sh
 * $ pulumi import aws:cfg/rule:Rule foo example
 * ```
 */
export declare class Rule extends pulumi.CustomResource {
    /**
     * Get an existing Rule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RuleState, opts?: pulumi.CustomResourceOptions): Rule;
    /**
     * Returns true if the given object is an instance of Rule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Rule;
    /**
     * The ARN of the config rule
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Description of the rule
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The modes the Config rule can be evaluated in. See Evaluation Mode for more details.
     */
    readonly evaluationModes: pulumi.Output<outputs.cfg.RuleEvaluationMode[]>;
    /**
     * A string in JSON format that is passed to the AWS Config rule Lambda function.
     */
    readonly inputParameters: pulumi.Output<string | undefined>;
    /**
     * The maximum frequency with which AWS Config runs evaluations for a rule.
     */
    readonly maximumExecutionFrequency: pulumi.Output<string | undefined>;
    /**
     * The name of the rule
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the config rule
     */
    readonly ruleId: pulumi.Output<string>;
    /**
     * Scope defines which resources can trigger an evaluation for the rule. See Scope Below.
     */
    readonly scope: pulumi.Output<outputs.cfg.RuleScope | undefined>;
    /**
     * Source specifies the rule owner, the rule identifier, and the notifications that cause the function to evaluate your AWS resources. See Source Below.
     */
    readonly source: pulumi.Output<outputs.cfg.RuleSource>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Rule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Rule resources.
 */
export interface RuleState {
    /**
     * The ARN of the config rule
     */
    arn?: pulumi.Input<string>;
    /**
     * Description of the rule
     */
    description?: pulumi.Input<string>;
    /**
     * The modes the Config rule can be evaluated in. See Evaluation Mode for more details.
     */
    evaluationModes?: pulumi.Input<pulumi.Input<inputs.cfg.RuleEvaluationMode>[]>;
    /**
     * A string in JSON format that is passed to the AWS Config rule Lambda function.
     */
    inputParameters?: pulumi.Input<string>;
    /**
     * The maximum frequency with which AWS Config runs evaluations for a rule.
     */
    maximumExecutionFrequency?: pulumi.Input<string>;
    /**
     * The name of the rule
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the config rule
     */
    ruleId?: pulumi.Input<string>;
    /**
     * Scope defines which resources can trigger an evaluation for the rule. See Scope Below.
     */
    scope?: pulumi.Input<inputs.cfg.RuleScope>;
    /**
     * Source specifies the rule owner, the rule identifier, and the notifications that cause the function to evaluate your AWS resources. See Source Below.
     */
    source?: pulumi.Input<inputs.cfg.RuleSource>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a Rule resource.
 */
export interface RuleArgs {
    /**
     * Description of the rule
     */
    description?: pulumi.Input<string>;
    /**
     * The modes the Config rule can be evaluated in. See Evaluation Mode for more details.
     */
    evaluationModes?: pulumi.Input<pulumi.Input<inputs.cfg.RuleEvaluationMode>[]>;
    /**
     * A string in JSON format that is passed to the AWS Config rule Lambda function.
     */
    inputParameters?: pulumi.Input<string>;
    /**
     * The maximum frequency with which AWS Config runs evaluations for a rule.
     */
    maximumExecutionFrequency?: pulumi.Input<string>;
    /**
     * The name of the rule
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Scope defines which resources can trigger an evaluation for the rule. See Scope Below.
     */
    scope?: pulumi.Input<inputs.cfg.RuleScope>;
    /**
     * Source specifies the rule owner, the rule identifier, and the notifications that cause the function to evaluate your AWS resources. See Source Below.
     */
    source: pulumi.Input<inputs.cfg.RuleSource>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
