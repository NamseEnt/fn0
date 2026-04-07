import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Bedrock AgentCore Code Interpreter. Code Interpreter provides a secure environment for AI agents to execute Python code, enabling data analysis, calculations, and file processing capabilities.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentcoreCodeInterpreter("example", {
 *     name: "example-code-interpreter",
 *     description: "Code interpreter for data analysis",
 *     networkConfiguration: {
 *         networkMode: "PUBLIC",
 *     },
 * });
 * ```
 *
 * ### Code Interpreter with Execution Role
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const assumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         actions: ["sts:AssumeRole"],
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["bedrock-agentcore.amazonaws.com"],
 *         }],
 *     }],
 * });
 * const example = new aws.iam.Role("example", {
 *     name: "bedrock-agentcore-code-interpreter-role",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const exampleAgentcoreCodeInterpreter = new aws.bedrock.AgentcoreCodeInterpreter("example", {
 *     name: "example-code-interpreter",
 *     description: "Code interpreter with custom execution role",
 *     executionRoleArn: example.arn,
 *     networkConfiguration: {
 *         networkMode: "SANDBOX",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Bedrock AgentCore Code Interpreter using the code interpreter ID. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/agentcoreCodeInterpreter:AgentcoreCodeInterpreter example CODEINTERPRETER1234567890
 * ```
 */
export declare class AgentcoreCodeInterpreter extends pulumi.CustomResource {
    /**
     * Get an existing AgentcoreCodeInterpreter resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentcoreCodeInterpreterState, opts?: pulumi.CustomResourceOptions): AgentcoreCodeInterpreter;
    /**
     * Returns true if the given object is an instance of AgentcoreCodeInterpreter.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AgentcoreCodeInterpreter;
    /**
     * ARN of the Code Interpreter.
     */
    readonly codeInterpreterArn: pulumi.Output<string>;
    /**
     * Unique identifier of the Code Interpreter.
     */
    readonly codeInterpreterId: pulumi.Output<string>;
    /**
     * Description of the code interpreter.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * ARN of the IAM role that the code interpreter assumes for execution. Required when using `SANDBOX` network mode.
     */
    readonly executionRoleArn: pulumi.Output<string | undefined>;
    /**
     * Name of the code interpreter.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Network configuration for the code interpreter. See `networkConfiguration` below.
     *
     * The following arguments are optional:
     */
    readonly networkConfiguration: pulumi.Output<outputs.bedrock.AgentcoreCodeInterpreterNetworkConfiguration | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
    readonly timeouts: pulumi.Output<outputs.bedrock.AgentcoreCodeInterpreterTimeouts | undefined>;
    /**
     * Create a AgentcoreCodeInterpreter resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: AgentcoreCodeInterpreterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AgentcoreCodeInterpreter resources.
 */
export interface AgentcoreCodeInterpreterState {
    /**
     * ARN of the Code Interpreter.
     */
    codeInterpreterArn?: pulumi.Input<string>;
    /**
     * Unique identifier of the Code Interpreter.
     */
    codeInterpreterId?: pulumi.Input<string>;
    /**
     * Description of the code interpreter.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the IAM role that the code interpreter assumes for execution. Required when using `SANDBOX` network mode.
     */
    executionRoleArn?: pulumi.Input<string>;
    /**
     * Name of the code interpreter.
     */
    name?: pulumi.Input<string>;
    /**
     * Network configuration for the code interpreter. See `networkConfiguration` below.
     *
     * The following arguments are optional:
     */
    networkConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreCodeInterpreterNetworkConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
    timeouts?: pulumi.Input<inputs.bedrock.AgentcoreCodeInterpreterTimeouts>;
}
/**
 * The set of arguments for constructing a AgentcoreCodeInterpreter resource.
 */
export interface AgentcoreCodeInterpreterArgs {
    /**
     * Description of the code interpreter.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the IAM role that the code interpreter assumes for execution. Required when using `SANDBOX` network mode.
     */
    executionRoleArn?: pulumi.Input<string>;
    /**
     * Name of the code interpreter.
     */
    name?: pulumi.Input<string>;
    /**
     * Network configuration for the code interpreter. See `networkConfiguration` below.
     *
     * The following arguments are optional:
     */
    networkConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreCodeInterpreterNetworkConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.AgentcoreCodeInterpreterTimeouts>;
}
