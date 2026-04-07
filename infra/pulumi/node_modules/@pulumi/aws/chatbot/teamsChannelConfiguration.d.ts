import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Chatbot Microsoft Teams Channel Configuration.
 *
 * > **NOTE:** We provide this resource on a best-effort basis. If you are able to test it and find it useful, we welcome your input at GitHub.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.chatbot.TeamsChannelConfiguration("test", {
 *     channelId: "C07EZ1ABC23",
 *     configurationName: "mitt-lags-kanal",
 *     iamRoleArn: testAwsIamRole.arn,
 *     teamId: "74361522-da01-538d-aa2e-ac7918c6bb92",
 *     tenantId: "1234",
 *     tags: {
 *         Name: "mitt-lags-kanal",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Chatbot Microsoft Teams Channel Configuration using the `team_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:chatbot/teamsChannelConfiguration:TeamsChannelConfiguration example 5f4f15d2-b958-522a-8333-124aa8bf0925
 * ```
 */
export declare class TeamsChannelConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing TeamsChannelConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TeamsChannelConfigurationState, opts?: pulumi.CustomResourceOptions): TeamsChannelConfiguration;
    /**
     * Returns true if the given object is an instance of TeamsChannelConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TeamsChannelConfiguration;
    /**
     * ID of the Microsoft Teams channel.
     */
    readonly channelId: pulumi.Output<string>;
    /**
     * Name of the Microsoft Teams channel.
     */
    readonly channelName: pulumi.Output<string>;
    /**
     * ARN of the Microsoft Teams channel configuration.
     */
    readonly chatConfigurationArn: pulumi.Output<string>;
    /**
     * Name of the Microsoft Teams channel configuration.
     */
    readonly configurationName: pulumi.Output<string>;
    /**
     * List of IAM policy ARNs that are applied as channel guardrails. The AWS managed `AdministratorAccess` policy is applied by default if this is not set.
     */
    readonly guardrailPolicyArns: pulumi.Output<string[]>;
    /**
     * ARN of the IAM role that defines the permissions for AWS Chatbot. This is a user-defined role that AWS Chatbot will assume. This is not the service-linked role.
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
    /**
     * ID of the Microsoft Team authorized with AWS Chatbot. To get the team ID, you must perform the initial authorization flow with Microsoft Teams in the AWS Chatbot console. Then you can copy and paste the team ID from the console.
     */
    readonly teamId: pulumi.Output<string>;
    /**
     * Name of the Microsoft Teams team.
     */
    readonly teamName: pulumi.Output<string>;
    /**
     * ID of the Microsoft Teams tenant.
     *
     * The following arguments are optional:
     */
    readonly tenantId: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.chatbot.TeamsChannelConfigurationTimeouts | undefined>;
    /**
     * Enables use of a user role requirement in your chat configuration.
     */
    readonly userAuthorizationRequired: pulumi.Output<boolean>;
    /**
     * Create a TeamsChannelConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TeamsChannelConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TeamsChannelConfiguration resources.
 */
export interface TeamsChannelConfigurationState {
    /**
     * ID of the Microsoft Teams channel.
     */
    channelId?: pulumi.Input<string>;
    /**
     * Name of the Microsoft Teams channel.
     */
    channelName?: pulumi.Input<string>;
    /**
     * ARN of the Microsoft Teams channel configuration.
     */
    chatConfigurationArn?: pulumi.Input<string>;
    /**
     * Name of the Microsoft Teams channel configuration.
     */
    configurationName?: pulumi.Input<string>;
    /**
     * List of IAM policy ARNs that are applied as channel guardrails. The AWS managed `AdministratorAccess` policy is applied by default if this is not set.
     */
    guardrailPolicyArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ARN of the IAM role that defines the permissions for AWS Chatbot. This is a user-defined role that AWS Chatbot will assume. This is not the service-linked role.
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
    /**
     * ID of the Microsoft Team authorized with AWS Chatbot. To get the team ID, you must perform the initial authorization flow with Microsoft Teams in the AWS Chatbot console. Then you can copy and paste the team ID from the console.
     */
    teamId?: pulumi.Input<string>;
    /**
     * Name of the Microsoft Teams team.
     */
    teamName?: pulumi.Input<string>;
    /**
     * ID of the Microsoft Teams tenant.
     *
     * The following arguments are optional:
     */
    tenantId?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.chatbot.TeamsChannelConfigurationTimeouts>;
    /**
     * Enables use of a user role requirement in your chat configuration.
     */
    userAuthorizationRequired?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a TeamsChannelConfiguration resource.
 */
export interface TeamsChannelConfigurationArgs {
    /**
     * ID of the Microsoft Teams channel.
     */
    channelId: pulumi.Input<string>;
    /**
     * Name of the Microsoft Teams channel.
     */
    channelName?: pulumi.Input<string>;
    /**
     * Name of the Microsoft Teams channel configuration.
     */
    configurationName: pulumi.Input<string>;
    /**
     * List of IAM policy ARNs that are applied as channel guardrails. The AWS managed `AdministratorAccess` policy is applied by default if this is not set.
     */
    guardrailPolicyArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ARN of the IAM role that defines the permissions for AWS Chatbot. This is a user-defined role that AWS Chatbot will assume. This is not the service-linked role.
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
     * ID of the Microsoft Team authorized with AWS Chatbot. To get the team ID, you must perform the initial authorization flow with Microsoft Teams in the AWS Chatbot console. Then you can copy and paste the team ID from the console.
     */
    teamId: pulumi.Input<string>;
    /**
     * Name of the Microsoft Teams team.
     */
    teamName?: pulumi.Input<string>;
    /**
     * ID of the Microsoft Teams tenant.
     *
     * The following arguments are optional:
     */
    tenantId: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.chatbot.TeamsChannelConfigurationTimeouts>;
    /**
     * Enables use of a user role requirement in your chat configuration.
     */
    userAuthorizationRequired?: pulumi.Input<boolean>;
}
