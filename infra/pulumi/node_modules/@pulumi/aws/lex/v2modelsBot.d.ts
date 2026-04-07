import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Lex V2 Models Bot.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleRole = new aws.iam.Role("example", {
 *     name: "example",
 *     assumeRolePolicy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: "sts:AssumeRole",
 *             Effect: "Allow",
 *             Sid: "",
 *             Principal: {
 *                 Service: "lexv2.amazonaws.com",
 *             },
 *         }],
 *     }),
 *     tags: {
 *         created_by: "aws",
 *     },
 * });
 * const example = new aws.lex.V2modelsBot("example", {
 *     name: "example",
 *     description: "Example description",
 *     dataPrivacies: [{
 *         childDirected: false,
 *     }],
 *     idleSessionTtlInSeconds: 60,
 *     roleArn: exampleRole.arn,
 *     type: "Bot",
 *     tags: {
 *         foo: "bar",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Lex V2 Models Bot using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:lex/v2modelsBot:V2modelsBot example bot-id-12345678
 * ```
 */
export declare class V2modelsBot extends pulumi.CustomResource {
    /**
     * Get an existing V2modelsBot resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: V2modelsBotState, opts?: pulumi.CustomResourceOptions): V2modelsBot;
    /**
     * Returns true if the given object is an instance of V2modelsBot.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is V2modelsBot;
    readonly arn: pulumi.Output<string>;
    /**
     * Provides information on additional privacy protections Amazon Lex should use with the bot's data. See `dataPrivacy`
     */
    readonly dataPrivacies: pulumi.Output<outputs.lex.V2modelsBotDataPrivacy[] | undefined>;
    /**
     * Description of the bot. It appears in lists to help you identify a particular bot.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Time, in seconds, that Amazon Lex should keep information about a user's conversation with the bot. You can specify between 60 (1 minute) and 86,400 (24 hours) seconds.
     */
    readonly idleSessionTtlInSeconds: pulumi.Output<number>;
    /**
     * List of bot members in a network to be created. See `botMembers`.
     */
    readonly members: pulumi.Output<outputs.lex.V2modelsBotMember[] | undefined>;
    /**
     * Name of the bot. The bot name must be unique in the account that creates the bot. Type String. Length Constraints: Minimum length of 1. Maximum length of 100.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of an IAM role that has permission to access the bot.
     *
     * The following arguments are optional:
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * List of tags to add to the bot. You can only add tags when you create a bot.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * List of tags to add to the test alias for a bot. You can only add tags when you create a bot.
     */
    readonly testBotAliasTags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly timeouts: pulumi.Output<outputs.lex.V2modelsBotTimeouts | undefined>;
    /**
     * Type of a bot to create. Possible values are `"Bot"` and `"BotNetwork"`.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a V2modelsBot resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: V2modelsBotArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering V2modelsBot resources.
 */
export interface V2modelsBotState {
    arn?: pulumi.Input<string>;
    /**
     * Provides information on additional privacy protections Amazon Lex should use with the bot's data. See `dataPrivacy`
     */
    dataPrivacies?: pulumi.Input<pulumi.Input<inputs.lex.V2modelsBotDataPrivacy>[]>;
    /**
     * Description of the bot. It appears in lists to help you identify a particular bot.
     */
    description?: pulumi.Input<string>;
    /**
     * Time, in seconds, that Amazon Lex should keep information about a user's conversation with the bot. You can specify between 60 (1 minute) and 86,400 (24 hours) seconds.
     */
    idleSessionTtlInSeconds?: pulumi.Input<number>;
    /**
     * List of bot members in a network to be created. See `botMembers`.
     */
    members?: pulumi.Input<pulumi.Input<inputs.lex.V2modelsBotMember>[]>;
    /**
     * Name of the bot. The bot name must be unique in the account that creates the bot. Type String. Length Constraints: Minimum length of 1. Maximum length of 100.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of an IAM role that has permission to access the bot.
     *
     * The following arguments are optional:
     */
    roleArn?: pulumi.Input<string>;
    /**
     * List of tags to add to the bot. You can only add tags when you create a bot.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * List of tags to add to the test alias for a bot. You can only add tags when you create a bot.
     */
    testBotAliasTags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.lex.V2modelsBotTimeouts>;
    /**
     * Type of a bot to create. Possible values are `"Bot"` and `"BotNetwork"`.
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a V2modelsBot resource.
 */
export interface V2modelsBotArgs {
    /**
     * Provides information on additional privacy protections Amazon Lex should use with the bot's data. See `dataPrivacy`
     */
    dataPrivacies?: pulumi.Input<pulumi.Input<inputs.lex.V2modelsBotDataPrivacy>[]>;
    /**
     * Description of the bot. It appears in lists to help you identify a particular bot.
     */
    description?: pulumi.Input<string>;
    /**
     * Time, in seconds, that Amazon Lex should keep information about a user's conversation with the bot. You can specify between 60 (1 minute) and 86,400 (24 hours) seconds.
     */
    idleSessionTtlInSeconds: pulumi.Input<number>;
    /**
     * List of bot members in a network to be created. See `botMembers`.
     */
    members?: pulumi.Input<pulumi.Input<inputs.lex.V2modelsBotMember>[]>;
    /**
     * Name of the bot. The bot name must be unique in the account that creates the bot. Type String. Length Constraints: Minimum length of 1. Maximum length of 100.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of an IAM role that has permission to access the bot.
     *
     * The following arguments are optional:
     */
    roleArn: pulumi.Input<string>;
    /**
     * List of tags to add to the bot. You can only add tags when you create a bot.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * List of tags to add to the test alias for a bot. You can only add tags when you create a bot.
     */
    testBotAliasTags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.lex.V2modelsBotTimeouts>;
    /**
     * Type of a bot to create. Possible values are `"Bot"` and `"BotNetwork"`.
     */
    type?: pulumi.Input<string>;
}
