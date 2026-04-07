import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Creates and manages an AWS IoT topic rule.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const mytopic = new aws.sns.Topic("mytopic", {name: "mytopic"});
 * const myerrortopic = new aws.sns.Topic("myerrortopic", {name: "myerrortopic"});
 * const rule = new aws.iot.TopicRule("rule", {
 *     name: "MyRule",
 *     description: "Example rule",
 *     enabled: true,
 *     sql: "SELECT * FROM 'topic/test'",
 *     sqlVersion: "2016-03-23",
 *     sns: [{
 *         messageFormat: "RAW",
 *         roleArn: role.arn,
 *         targetArn: mytopic.arn,
 *     }],
 *     errorAction: {
 *         sns: {
 *             messageFormat: "RAW",
 *             roleArn: role.arn,
 *             targetArn: myerrortopic.arn,
 *         },
 *     },
 * });
 * const assumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["iot.amazonaws.com"],
 *         }],
 *         actions: ["sts:AssumeRole"],
 *     }],
 * });
 * const myrole = new aws.iam.Role("myrole", {
 *     name: "myrole",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const mypolicy = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         effect: "Allow",
 *         actions: ["sns:Publish"],
 *         resources: [mytopic.arn],
 *     }],
 * });
 * const mypolicyRolePolicy = new aws.iam.RolePolicy("mypolicy", {
 *     name: "mypolicy",
 *     role: myrole.id,
 *     policy: mypolicy.apply(mypolicy => mypolicy.json),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import IoT Topic Rules using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:iot/topicRule:TopicRule rule <name>
 * ```
 */
export declare class TopicRule extends pulumi.CustomResource {
    /**
     * Get an existing TopicRule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TopicRuleState, opts?: pulumi.CustomResourceOptions): TopicRule;
    /**
     * Returns true if the given object is an instance of TopicRule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TopicRule;
    /**
     * The ARN of the topic rule
     */
    readonly arn: pulumi.Output<string>;
    readonly cloudwatchAlarms: pulumi.Output<outputs.iot.TopicRuleCloudwatchAlarm[] | undefined>;
    readonly cloudwatchLogs: pulumi.Output<outputs.iot.TopicRuleCloudwatchLog[] | undefined>;
    readonly cloudwatchMetrics: pulumi.Output<outputs.iot.TopicRuleCloudwatchMetric[] | undefined>;
    /**
     * The description of the rule.
     */
    readonly description: pulumi.Output<string | undefined>;
    readonly dynamodbs: pulumi.Output<outputs.iot.TopicRuleDynamodb[] | undefined>;
    readonly dynamodbv2s: pulumi.Output<outputs.iot.TopicRuleDynamodbv2[] | undefined>;
    readonly elasticsearch: pulumi.Output<outputs.iot.TopicRuleElasticsearch[] | undefined>;
    /**
     * Specifies whether the rule is enabled.
     */
    readonly enabled: pulumi.Output<boolean>;
    /**
     * Configuration block with error action to be associated with the rule. See the documentation for `cloudwatchAlarm`, `cloudwatchLogs`, `cloudwatchMetric`, `dynamodb`, `dynamodbv2`, `elasticsearch`, `firehose`, `http`, `iotAnalytics`, `iotEvents`, `kafka`, `kinesis`, `lambda`, `republish`, `s3`, `sns`, `sqs`, `stepFunctions`, `timestream` configuration blocks for further configuration details.
     */
    readonly errorAction: pulumi.Output<outputs.iot.TopicRuleErrorAction | undefined>;
    readonly firehoses: pulumi.Output<outputs.iot.TopicRuleFirehose[] | undefined>;
    readonly https: pulumi.Output<outputs.iot.TopicRuleHttp[] | undefined>;
    readonly iotAnalytics: pulumi.Output<outputs.iot.TopicRuleIotAnalytic[] | undefined>;
    readonly iotEvents: pulumi.Output<outputs.iot.TopicRuleIotEvent[] | undefined>;
    readonly kafkas: pulumi.Output<outputs.iot.TopicRuleKafka[] | undefined>;
    readonly kineses: pulumi.Output<outputs.iot.TopicRuleKinesis[] | undefined>;
    readonly lambdas: pulumi.Output<outputs.iot.TopicRuleLambda[] | undefined>;
    /**
     * The name of the rule.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly republishes: pulumi.Output<outputs.iot.TopicRuleRepublish[] | undefined>;
    readonly s3: pulumi.Output<outputs.iot.TopicRuleS3[] | undefined>;
    readonly sns: pulumi.Output<outputs.iot.TopicRuleSns[] | undefined>;
    /**
     * The SQL statement used to query the topic. For more information, see AWS IoT SQL Reference (http://docs.aws.amazon.com/iot/latest/developerguide/iot-rules.html#aws-iot-sql-reference) in the AWS IoT Developer Guide.
     */
    readonly sql: pulumi.Output<string>;
    /**
     * The version of the SQL rules engine to use when evaluating the rule.
     */
    readonly sqlVersion: pulumi.Output<string>;
    readonly sqs: pulumi.Output<outputs.iot.TopicRuleSqs[] | undefined>;
    readonly stepFunctions: pulumi.Output<outputs.iot.TopicRuleStepFunction[] | undefined>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timestreams: pulumi.Output<outputs.iot.TopicRuleTimestream[] | undefined>;
    /**
     * Create a TopicRule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TopicRuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TopicRule resources.
 */
export interface TopicRuleState {
    /**
     * The ARN of the topic rule
     */
    arn?: pulumi.Input<string>;
    cloudwatchAlarms?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleCloudwatchAlarm>[]>;
    cloudwatchLogs?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleCloudwatchLog>[]>;
    cloudwatchMetrics?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleCloudwatchMetric>[]>;
    /**
     * The description of the rule.
     */
    description?: pulumi.Input<string>;
    dynamodbs?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleDynamodb>[]>;
    dynamodbv2s?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleDynamodbv2>[]>;
    elasticsearch?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleElasticsearch>[]>;
    /**
     * Specifies whether the rule is enabled.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Configuration block with error action to be associated with the rule. See the documentation for `cloudwatchAlarm`, `cloudwatchLogs`, `cloudwatchMetric`, `dynamodb`, `dynamodbv2`, `elasticsearch`, `firehose`, `http`, `iotAnalytics`, `iotEvents`, `kafka`, `kinesis`, `lambda`, `republish`, `s3`, `sns`, `sqs`, `stepFunctions`, `timestream` configuration blocks for further configuration details.
     */
    errorAction?: pulumi.Input<inputs.iot.TopicRuleErrorAction>;
    firehoses?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleFirehose>[]>;
    https?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleHttp>[]>;
    iotAnalytics?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleIotAnalytic>[]>;
    iotEvents?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleIotEvent>[]>;
    kafkas?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleKafka>[]>;
    kineses?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleKinesis>[]>;
    lambdas?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleLambda>[]>;
    /**
     * The name of the rule.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    republishes?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleRepublish>[]>;
    s3?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleS3>[]>;
    sns?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleSns>[]>;
    /**
     * The SQL statement used to query the topic. For more information, see AWS IoT SQL Reference (http://docs.aws.amazon.com/iot/latest/developerguide/iot-rules.html#aws-iot-sql-reference) in the AWS IoT Developer Guide.
     */
    sql?: pulumi.Input<string>;
    /**
     * The version of the SQL rules engine to use when evaluating the rule.
     */
    sqlVersion?: pulumi.Input<string>;
    sqs?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleSqs>[]>;
    stepFunctions?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleStepFunction>[]>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timestreams?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleTimestream>[]>;
}
/**
 * The set of arguments for constructing a TopicRule resource.
 */
export interface TopicRuleArgs {
    cloudwatchAlarms?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleCloudwatchAlarm>[]>;
    cloudwatchLogs?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleCloudwatchLog>[]>;
    cloudwatchMetrics?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleCloudwatchMetric>[]>;
    /**
     * The description of the rule.
     */
    description?: pulumi.Input<string>;
    dynamodbs?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleDynamodb>[]>;
    dynamodbv2s?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleDynamodbv2>[]>;
    elasticsearch?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleElasticsearch>[]>;
    /**
     * Specifies whether the rule is enabled.
     */
    enabled: pulumi.Input<boolean>;
    /**
     * Configuration block with error action to be associated with the rule. See the documentation for `cloudwatchAlarm`, `cloudwatchLogs`, `cloudwatchMetric`, `dynamodb`, `dynamodbv2`, `elasticsearch`, `firehose`, `http`, `iotAnalytics`, `iotEvents`, `kafka`, `kinesis`, `lambda`, `republish`, `s3`, `sns`, `sqs`, `stepFunctions`, `timestream` configuration blocks for further configuration details.
     */
    errorAction?: pulumi.Input<inputs.iot.TopicRuleErrorAction>;
    firehoses?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleFirehose>[]>;
    https?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleHttp>[]>;
    iotAnalytics?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleIotAnalytic>[]>;
    iotEvents?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleIotEvent>[]>;
    kafkas?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleKafka>[]>;
    kineses?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleKinesis>[]>;
    lambdas?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleLambda>[]>;
    /**
     * The name of the rule.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    republishes?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleRepublish>[]>;
    s3?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleS3>[]>;
    sns?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleSns>[]>;
    /**
     * The SQL statement used to query the topic. For more information, see AWS IoT SQL Reference (http://docs.aws.amazon.com/iot/latest/developerguide/iot-rules.html#aws-iot-sql-reference) in the AWS IoT Developer Guide.
     */
    sql: pulumi.Input<string>;
    /**
     * The version of the SQL rules engine to use when evaluating the rule.
     */
    sqlVersion: pulumi.Input<string>;
    sqs?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleSqs>[]>;
    stepFunctions?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleStepFunction>[]>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timestreams?: pulumi.Input<pulumi.Input<inputs.iot.TopicRuleTimestream>[]>;
}
