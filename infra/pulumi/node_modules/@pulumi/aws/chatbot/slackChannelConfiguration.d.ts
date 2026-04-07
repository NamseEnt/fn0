import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Chatbot Slack Channel Configuration.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.chatbot.SlackChannelConfiguration("test", {
 *     configurationName: "min-slaka-kanal",
 *     iamRoleArn: testAwsIamRole.arn,
 *     slackChannelId: "C07EZ1ABC23",
 *     slackTeamId: "T07EA123LEP",
 *     tags: {
 *         Name: "min-slaka-kanal",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Chatbot Slack Channel Configuration using the `chat_configuration_arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:chatbot/slackChannelConfiguration:SlackChannelConfiguration example arn:aws:chatbot::123456789012:chat-configuration/slack-channel/min-slaka-kanal
 * ```
 */
export declare class SlackChannelConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing SlackChannelConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SlackChannelConfigurationState, opts?: pulumi.CustomResourceOptions): SlackChannelConfiguration;
    /**
     * Returns true if the given object is an instance of SlackChannelConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SlackChannelConfiguration;
    /**
     * ARN of the Slack channel configuration.
     */
    readonly chatConfigurationArn: pulumi.Output<string>;
    /**
     * Name of the Slack channel configuration.
     */
    readonly configurationName: pulumi.Output<string>;
    /**
     * List of IAM policy ARNs that are applied as channel guardrails. The AWS managed `AdministratorAccess` policy is applied by default if this is not set.
     */
    readonly guardrailPolicyArns: pulumi.Output<string[]>;
    /**
     * User-defined role that AWS Chatbot assumes. This is not the service-linked role.
     */
    readonly iamRoleArn: pulumi.Output<string>;
    /**
     * Logging levels include `ERROR`, `INFO`, or `NONE`.
     */
    readonly loggingLevel: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID of the Slack channel. For example, `C07EZ1ABC23`.
     */
    readonly slackChannelId: pulumi.Output<string>;
    /**
     * Name of the Slack channel.
     */
    readonly slackChannelName: pulumi.Output<string>;
    /**
     * ID of the Slack workspace authorized with AWS Chatbot. For example, `T07EA123LEP`.
     *
     * The following arguments are optional:
     */
    readonly slackTeamId: pulumi.Output<string>;
    /**
     * Name of the Slack team.
     */
    readonly slackTeamName: pulumi.Output<string>;
    /**
     * ARNs of the SNS topics that deliver notifications to AWS Chatbot.
     */
    readonly snsTopicArns: pulumi.Output<string[]>;
    /**
     * Map of tags assigned to the resource.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.chatbot.SlackChannelConfigurationTimeouts | undefined>;
    /**
     * Enables use of a user role requirement in your chat configuration.
     */
    readonly userAuthorizationRequired: pulumi.Output<boolean>;
    /**
     * Create a SlackChannelConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SlackChannelConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SlackChannelConfiguration resources.
 */
export interface SlackChannelConfigurationState {
    /**
     * ARN of the Slack channel configuration.
     */
    chatConfigurationArn?: pulumi.Input<string>;
    /**
     * Name of the Slack channel configuration.
     */
    configurationName?: pulumi.Input<string>;
    /**
     * List of IAM policy ARNs that are applied as channel guardrails. The AWS managed `AdministratorAccess` policy is applied by default if this is not set.
     */
    guardrailPolicyArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * User-defined role that AWS Chatbot assumes. This is not the service-linked role.
     */
    iamRoleArn?: pulumi.Input<string>;
    /**
     * Logging levels include `ERROR`, `INFO`, or `NONE`.
     */
    loggingLevel?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the Slack channel. For example, `C07EZ1ABC23`.
     */
    slackChannelId?: pulumi.Input<string>;
    /**
     * Name of the Slack channel.
     */
    slackChannelName?: pulumi.Input<string>;
    /**
     * ID of the Slack workspace authorized with AWS Chatbot. For example, `T07EA123LEP`.
     *
     * The following arguments are optional:
     */
    slackTeamId?: pulumi.Input<string>;
    /**
     * Name of the Slack team.
     */
    slackTeamName?: pulumi.Input<string>;
    /**
     * ARNs of the SNS topics that deliver notifications to AWS Chatbot.
     */
    snsTopicArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Map of tags assigned to the resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.chatbot.SlackChannelConfigurationTimeouts>;
    /**
     * Enables use of a user role requirement in your chat configuration.
     */
    userAuthorizationRequired?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a SlackChannelConfiguration resource.
 */
export interface SlackChannelConfigurationArgs {
    /**
     * Name of the Slack channel configuration.
     */
    configurationName: pulumi.Input<string>;
    /**
     * List of IAM policy ARNs that are applied as channel guardrails. The AWS managed `AdministratorAccess` policy is applied by default if this is not set.
     */
    guardrailPolicyArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * User-defined role that AWS Chatbot assumes. This is not the service-linked role.
     */
    iamRoleArn: pulumi.Input<string>;
    /**
     * Logging levels include `ERROR`, `INFO`, or `NONE`.
     */
    loggingLevel?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the Slack channel. For example, `C07EZ1ABC23`.
     */
    slackChannelId: pulumi.Input<string>;
    /**
     * ID of the Slack workspace authorized with AWS Chatbot. For example, `T07EA123LEP`.
     *
     * The following arguments are optional:
     */
    slackTeamId: pulumi.Input<string>;
    /**
     * ARNs of the SNS topics that deliver notifications to AWS Chatbot.
     */
    snsTopicArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Map of tags assigned to the resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.chatbot.SlackChannelConfigurationTimeouts>;
    /**
     * Enables use of a user role requirement in your chat configuration.
     */
    userAuthorizationRequired?: pulumi.Input<boolean>;
}
