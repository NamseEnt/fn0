import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Bedrock AgentCore Agent Runtime Endpoint. Agent Runtime Endpoints provide a network-accessible interface for interacting with agent runtimes, enabling external systems to communicate with and invoke agent capabilities.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentcoreAgentRuntimeEndpoint("example", {
 *     name: "example-endpoint",
 *     agentRuntimeId: exampleAwsBedrockagentcoreAgentRuntime.agentRuntimeId,
 *     description: "Endpoint for agent runtime communication",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Bedrock AgentCore Agent Runtime Endpoint using the `agent_runtime_id` and `name` separated by a comma. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/agentcoreAgentRuntimeEndpoint:AgentcoreAgentRuntimeEndpoint example AGENTRUNTIME1234567890,example-endpoint
 * ```
 */
export declare class AgentcoreAgentRuntimeEndpoint extends pulumi.CustomResource {
    /**
     * Get an existing AgentcoreAgentRuntimeEndpoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentcoreAgentRuntimeEndpointState, opts?: pulumi.CustomResourceOptions): AgentcoreAgentRuntimeEndpoint;
    /**
     * Returns true if the given object is an instance of AgentcoreAgentRuntimeEndpoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AgentcoreAgentRuntimeEndpoint;
    /**
     * ARN of the associated Agent Runtime.
     */
    readonly agentRuntimeArn: pulumi.Output<string>;
    /**
     * ARN of the Agent Runtime Endpoint.
     */
    readonly agentRuntimeEndpointArn: pulumi.Output<string>;
    /**
     * ID of the agent runtime this endpoint belongs to.
     *
     * The following arguments are optional:
     */
    readonly agentRuntimeId: pulumi.Output<string>;
    /**
     * Version of the agent runtime to use for this endpoint.
     */
    readonly agentRuntimeVersion: pulumi.Output<string>;
    /**
     * Description of the agent runtime endpoint.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Name of the agent runtime endpoint.
     */
    readonly name: pulumi.Output<string>;
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
    readonly timeouts: pulumi.Output<outputs.bedrock.AgentcoreAgentRuntimeEndpointTimeouts | undefined>;
    /**
     * Create a AgentcoreAgentRuntimeEndpoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AgentcoreAgentRuntimeEndpointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AgentcoreAgentRuntimeEndpoint resources.
 */
export interface AgentcoreAgentRuntimeEndpointState {
    /**
     * ARN of the associated Agent Runtime.
     */
    agentRuntimeArn?: pulumi.Input<string>;
    /**
     * ARN of the Agent Runtime Endpoint.
     */
    agentRuntimeEndpointArn?: pulumi.Input<string>;
    /**
     * ID of the agent runtime this endpoint belongs to.
     *
     * The following arguments are optional:
     */
    agentRuntimeId?: pulumi.Input<string>;
    /**
     * Version of the agent runtime to use for this endpoint.
     */
    agentRuntimeVersion?: pulumi.Input<string>;
    /**
     * Description of the agent runtime endpoint.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the agent runtime endpoint.
     */
    name?: pulumi.Input<string>;
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
    timeouts?: pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeEndpointTimeouts>;
}
/**
 * The set of arguments for constructing a AgentcoreAgentRuntimeEndpoint resource.
 */
export interface AgentcoreAgentRuntimeEndpointArgs {
    /**
     * ID of the agent runtime this endpoint belongs to.
     *
     * The following arguments are optional:
     */
    agentRuntimeId: pulumi.Input<string>;
    /**
     * Version of the agent runtime to use for this endpoint.
     */
    agentRuntimeVersion?: pulumi.Input<string>;
    /**
     * Description of the agent runtime endpoint.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the agent runtime endpoint.
     */
    name?: pulumi.Input<string>;
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
    timeouts?: pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeEndpointTimeouts>;
}
