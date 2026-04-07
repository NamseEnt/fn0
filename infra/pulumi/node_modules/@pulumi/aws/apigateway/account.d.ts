import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a settings of an API Gateway Account. Settings is applied region-wide per `provider` block.
 *
 * ## Example Usage
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
 *             identifiers: ["apigateway.amazonaws.com"],
 *         }],
 *         actions: ["sts:AssumeRole"],
 *     }],
 * });
 * const cloudwatchRole = new aws.iam.Role("cloudwatch", {
 *     name: "api_gateway_cloudwatch_global",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const demo = new aws.apigateway.Account("demo", {cloudwatchRoleArn: cloudwatchRole.arn});
 * const cloudwatch = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         actions: [
 *             "logs:CreateLogGroup",
 *             "logs:CreateLogStream",
 *             "logs:DescribeLogGroups",
 *             "logs:DescribeLogStreams",
 *             "logs:PutLogEvents",
 *             "logs:GetLogEvents",
 *             "logs:FilterLogEvents",
 *         ],
 *         resources: ["*"],
 *     }],
 * });
 * const cloudwatchRolePolicy = new aws.iam.RolePolicy("cloudwatch", {
 *     name: "default",
 *     role: cloudwatchRole.id,
 *     policy: cloudwatch.then(cloudwatch => cloudwatch.json),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import API Gateway Accounts using the account ID. For example:
 *
 * ```sh
 * $ pulumi import aws:apigateway/account:Account demo 123456789012
 * ```
 */
export declare class Account extends pulumi.CustomResource {
    /**
     * Get an existing Account resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AccountState, opts?: pulumi.CustomResourceOptions): Account;
    /**
     * Returns true if the given object is an instance of Account.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Account;
    /**
     * The version of the API keys used for the account.
     */
    readonly apiKeyVersion: pulumi.Output<string>;
    /**
     * ARN of an IAM role for CloudWatch (to allow logging & monitoring). See more [in AWS Docs](https://docs.aws.amazon.com/apigateway/latest/developerguide/how-to-stage-settings.html#how-to-stage-settings-console). Logging & monitoring can be enabled/disabled and otherwise tuned on the API Gateway Stage level.
     */
    readonly cloudwatchRoleArn: pulumi.Output<string>;
    /**
     * A list of features supported for the account.
     */
    readonly features: pulumi.Output<string[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Account-Level throttle settings. See exported fields below.
     */
    readonly throttleSettings: pulumi.Output<outputs.apigateway.AccountThrottleSetting[]>;
    /**
     * Create a Account resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: AccountArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Account resources.
 */
export interface AccountState {
    /**
     * The version of the API keys used for the account.
     */
    apiKeyVersion?: pulumi.Input<string>;
    /**
     * ARN of an IAM role for CloudWatch (to allow logging & monitoring). See more [in AWS Docs](https://docs.aws.amazon.com/apigateway/latest/developerguide/how-to-stage-settings.html#how-to-stage-settings-console). Logging & monitoring can be enabled/disabled and otherwise tuned on the API Gateway Stage level.
     */
    cloudwatchRoleArn?: pulumi.Input<string>;
    /**
     * A list of features supported for the account.
     */
    features?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Account-Level throttle settings. See exported fields below.
     */
    throttleSettings?: pulumi.Input<pulumi.Input<inputs.apigateway.AccountThrottleSetting>[]>;
}
/**
 * The set of arguments for constructing a Account resource.
 */
export interface AccountArgs {
    /**
     * ARN of an IAM role for CloudWatch (to allow logging & monitoring). See more [in AWS Docs](https://docs.aws.amazon.com/apigateway/latest/developerguide/how-to-stage-settings.html#how-to-stage-settings-console). Logging & monitoring can be enabled/disabled and otherwise tuned on the API Gateway Stage level.
     */
    cloudwatchRoleArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
