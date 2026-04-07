import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS Config Remediation Configuration.
 *
 * > **Note:** Config Remediation Configuration requires an existing Config Rule to be present.
 *
 * ## Example Usage
 *
 * AWS managed rules can be used by setting the source owner to `AWS` and the source identifier to the name of the managed rule. More information about AWS managed rules can be found in the [AWS Config Developer Guide](https://docs.aws.amazon.com/config/latest/developerguide/evaluate-config_use-managed-rules.html).
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const _this = new aws.cfg.Rule("this", {
 *     name: "example",
 *     source: {
 *         owner: "AWS",
 *         sourceIdentifier: "S3_BUCKET_VERSIONING_ENABLED",
 *     },
 * });
 * const thisRemediationConfiguration = new aws.cfg.RemediationConfiguration("this", {
 *     configRuleName: _this.name,
 *     resourceType: "AWS::S3::Bucket",
 *     targetType: "SSM_DOCUMENT",
 *     targetId: "AWS-EnableS3BucketEncryption",
 *     targetVersion: "1",
 *     parameters: [
 *         {
 *             name: "AutomationAssumeRole",
 *             staticValue: "arn:aws:iam::875924563244:role/security_config",
 *         },
 *         {
 *             name: "BucketName",
 *             resourceValue: "RESOURCE_ID",
 *         },
 *         {
 *             name: "SSEAlgorithm",
 *             staticValue: "AES256",
 *         },
 *     ],
 *     automatic: true,
 *     maximumAutomaticAttempts: 10,
 *     retryAttemptSeconds: 600,
 *     executionControls: {
 *         ssmControls: {
 *             concurrentExecutionRatePercentage: 25,
 *             errorPercentage: 20,
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Remediation Configurations using the name config_rule_name. For example:
 *
 * ```sh
 * $ pulumi import aws:cfg/remediationConfiguration:RemediationConfiguration this example
 * ```
 */
export declare class RemediationConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing RemediationConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RemediationConfigurationState, opts?: pulumi.CustomResourceOptions): RemediationConfiguration;
    /**
     * Returns true if the given object is an instance of RemediationConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RemediationConfiguration;
    /**
     * ARN of the Config Remediation Configuration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Remediation is triggered automatically if `true`.
     */
    readonly automatic: pulumi.Output<boolean | undefined>;
    /**
     * Name of the AWS Config rule.
     */
    readonly configRuleName: pulumi.Output<string>;
    /**
     * Configuration block for execution controls. See below.
     */
    readonly executionControls: pulumi.Output<outputs.cfg.RemediationConfigurationExecutionControls | undefined>;
    /**
     * Maximum number of failed attempts for auto-remediation. If you do not select a number, the default is 5.
     */
    readonly maximumAutomaticAttempts: pulumi.Output<number | undefined>;
    /**
     * Can be specified multiple times for each parameter. Each parameter block supports arguments below.
     */
    readonly parameters: pulumi.Output<outputs.cfg.RemediationConfigurationParameter[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Type of resource.
     */
    readonly resourceType: pulumi.Output<string | undefined>;
    /**
     * Maximum time in seconds that AWS Config runs auto-remediation. If you do not select a number, the default is 60 seconds.
     */
    readonly retryAttemptSeconds: pulumi.Output<number | undefined>;
    /**
     * Target ID is the name of the public document.
     */
    readonly targetId: pulumi.Output<string>;
    /**
     * Type of the target. Target executes remediation. For example, SSM document.
     *
     * The following arguments are optional:
     */
    readonly targetType: pulumi.Output<string>;
    /**
     * Version of the target. For example, version of the SSM document
     */
    readonly targetVersion: pulumi.Output<string | undefined>;
    /**
     * Create a RemediationConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RemediationConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RemediationConfiguration resources.
 */
export interface RemediationConfigurationState {
    /**
     * ARN of the Config Remediation Configuration.
     */
    arn?: pulumi.Input<string>;
    /**
     * Remediation is triggered automatically if `true`.
     */
    automatic?: pulumi.Input<boolean>;
    /**
     * Name of the AWS Config rule.
     */
    configRuleName?: pulumi.Input<string>;
    /**
     * Configuration block for execution controls. See below.
     */
    executionControls?: pulumi.Input<inputs.cfg.RemediationConfigurationExecutionControls>;
    /**
     * Maximum number of failed attempts for auto-remediation. If you do not select a number, the default is 5.
     */
    maximumAutomaticAttempts?: pulumi.Input<number>;
    /**
     * Can be specified multiple times for each parameter. Each parameter block supports arguments below.
     */
    parameters?: pulumi.Input<pulumi.Input<inputs.cfg.RemediationConfigurationParameter>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Type of resource.
     */
    resourceType?: pulumi.Input<string>;
    /**
     * Maximum time in seconds that AWS Config runs auto-remediation. If you do not select a number, the default is 60 seconds.
     */
    retryAttemptSeconds?: pulumi.Input<number>;
    /**
     * Target ID is the name of the public document.
     */
    targetId?: pulumi.Input<string>;
    /**
     * Type of the target. Target executes remediation. For example, SSM document.
     *
     * The following arguments are optional:
     */
    targetType?: pulumi.Input<string>;
    /**
     * Version of the target. For example, version of the SSM document
     */
    targetVersion?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RemediationConfiguration resource.
 */
export interface RemediationConfigurationArgs {
    /**
     * Remediation is triggered automatically if `true`.
     */
    automatic?: pulumi.Input<boolean>;
    /**
     * Name of the AWS Config rule.
     */
    configRuleName: pulumi.Input<string>;
    /**
     * Configuration block for execution controls. See below.
     */
    executionControls?: pulumi.Input<inputs.cfg.RemediationConfigurationExecutionControls>;
    /**
     * Maximum number of failed attempts for auto-remediation. If you do not select a number, the default is 5.
     */
    maximumAutomaticAttempts?: pulumi.Input<number>;
    /**
     * Can be specified multiple times for each parameter. Each parameter block supports arguments below.
     */
    parameters?: pulumi.Input<pulumi.Input<inputs.cfg.RemediationConfigurationParameter>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Type of resource.
     */
    resourceType?: pulumi.Input<string>;
    /**
     * Maximum time in seconds that AWS Config runs auto-remediation. If you do not select a number, the default is 60 seconds.
     */
    retryAttemptSeconds?: pulumi.Input<number>;
    /**
     * Target ID is the name of the public document.
     */
    targetId: pulumi.Input<string>;
    /**
     * Type of the target. Target executes remediation. For example, SSM document.
     *
     * The following arguments are optional:
     */
    targetType: pulumi.Input<string>;
    /**
     * Version of the target. For example, version of the SSM document
     */
    targetVersion?: pulumi.Input<string>;
}
