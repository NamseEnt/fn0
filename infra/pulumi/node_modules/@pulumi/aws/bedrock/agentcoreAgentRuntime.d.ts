import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Bedrock AgentCore Agent Runtime. Agent Runtime provides a containerized execution environment for AI agents.
 *
 * ## Example Usage
 *
 * ### Basic Usage
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
 * const ecrPermissions = aws.iam.getPolicyDocument({
 *     statements: [
 *         {
 *             actions: ["ecr:GetAuthorizationToken"],
 *             effect: "Allow",
 *             resources: ["*"],
 *         },
 *         {
 *             actions: [
 *                 "ecr:BatchGetImage",
 *                 "ecr:GetDownloadUrlForLayer",
 *             ],
 *             effect: "Allow",
 *             resources: [exampleAwsEcrRepository.arn],
 *         },
 *     ],
 * });
 * const example = new aws.iam.Role("example", {
 *     name: "bedrock-agentcore-runtime-role",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const exampleRolePolicy = new aws.iam.RolePolicy("example", {
 *     role: example.id,
 *     policy: ecrPermissions.then(ecrPermissions => ecrPermissions.json),
 * });
 * const exampleAgentcoreAgentRuntime = new aws.bedrock.AgentcoreAgentRuntime("example", {
 *     agentRuntimeName: "example_agent_runtime",
 *     roleArn: example.arn,
 *     agentRuntimeArtifact: {
 *         containerConfiguration: {
 *             containerUri: `${exampleAwsEcrRepository.repositoryUrl}:latest`,
 *         },
 *     },
 *     networkConfiguration: {
 *         networkMode: "PUBLIC",
 *     },
 * });
 * ```
 *
 * ### MCP Server With Custom JWT Authorizer
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentcoreAgentRuntime("example", {
 *     agentRuntimeName: "example_agent_runtime",
 *     description: "Agent runtime with JWT authorization",
 *     roleArn: exampleAwsIamRole.arn,
 *     agentRuntimeArtifact: {
 *         containerConfiguration: {
 *             containerUri: `${exampleAwsEcrRepository.repositoryUrl}:v1.0`,
 *         },
 *     },
 *     environmentVariables: {
 *         LOG_LEVEL: "INFO",
 *         ENV: "production",
 *     },
 *     authorizerConfiguration: {
 *         customJwtAuthorizer: {
 *             discoveryUrl: "https://accounts.google.com/.well-known/openid-configuration",
 *             allowedAudiences: [
 *                 "my-app",
 *                 "mobile-app",
 *             ],
 *             allowedClients: [
 *                 "client-123",
 *                 "client-456",
 *             ],
 *         },
 *     },
 *     networkConfiguration: {
 *         networkMode: "PUBLIC",
 *     },
 *     protocolConfiguration: {
 *         serverProtocol: "MCP",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Bedrock AgentCore Agent Runtime using `agent_runtime_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/agentcoreAgentRuntime:AgentcoreAgentRuntime example agent-runtime-12345
 * ```
 */
export declare class AgentcoreAgentRuntime extends pulumi.CustomResource {
    /**
     * Get an existing AgentcoreAgentRuntime resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentcoreAgentRuntimeState, opts?: pulumi.CustomResourceOptions): AgentcoreAgentRuntime;
    /**
     * Returns true if the given object is an instance of AgentcoreAgentRuntime.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AgentcoreAgentRuntime;
    /**
     * ARN of the Agent Runtime.
     */
    readonly agentRuntimeArn: pulumi.Output<string>;
    /**
     * Container artifact configuration. See `agentRuntimeArtifact` below.
     */
    readonly agentRuntimeArtifact: pulumi.Output<outputs.bedrock.AgentcoreAgentRuntimeAgentRuntimeArtifact | undefined>;
    /**
     * Unique identifier of the Agent Runtime.
     */
    readonly agentRuntimeId: pulumi.Output<string>;
    /**
     * Name of the agent runtime.
     */
    readonly agentRuntimeName: pulumi.Output<string>;
    /**
     * Version of the Agent Runtime.
     */
    readonly agentRuntimeVersion: pulumi.Output<string>;
    /**
     * Authorization configuration for authenticating incoming requests. See `authorizerConfiguration` below.
     */
    readonly authorizerConfiguration: pulumi.Output<outputs.bedrock.AgentcoreAgentRuntimeAuthorizerConfiguration | undefined>;
    /**
     * Description of the agent runtime.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Map of environment variables to pass to the container.
     */
    readonly environmentVariables: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Runtime session and resource lifecycle configuration for the agent runtime. See `lifecycleConfiguration` below.
     */
    readonly lifecycleConfigurations: pulumi.Output<outputs.bedrock.AgentcoreAgentRuntimeLifecycleConfiguration[]>;
    /**
     * Network configuration for the agent runtime. See `networkConfiguration` below.
     *
     * The following arguments are optional:
     */
    readonly networkConfiguration: pulumi.Output<outputs.bedrock.AgentcoreAgentRuntimeNetworkConfiguration | undefined>;
    /**
     * Protocol configuration for the agent runtime. See `protocolConfiguration` below.
     */
    readonly protocolConfiguration: pulumi.Output<outputs.bedrock.AgentcoreAgentRuntimeProtocolConfiguration | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration for HTTP request headers that will be passed through to the runtime. See `requestHeaderConfiguration` below.
     */
    readonly requestHeaderConfiguration: pulumi.Output<outputs.bedrock.AgentcoreAgentRuntimeRequestHeaderConfiguration | undefined>;
    /**
     * ARN of the IAM role that the agent runtime assumes to access AWS services.
     */
    readonly roleArn: pulumi.Output<string>;
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
    readonly timeouts: pulumi.Output<outputs.bedrock.AgentcoreAgentRuntimeTimeouts | undefined>;
    /**
     * Workload identity details for the agent runtime. See `workloadIdentityDetails` below.
     */
    readonly workloadIdentityDetails: pulumi.Output<outputs.bedrock.AgentcoreAgentRuntimeWorkloadIdentityDetail[]>;
    /**
     * Create a AgentcoreAgentRuntime resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AgentcoreAgentRuntimeArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AgentcoreAgentRuntime resources.
 */
export interface AgentcoreAgentRuntimeState {
    /**
     * ARN of the Agent Runtime.
     */
    agentRuntimeArn?: pulumi.Input<string>;
    /**
     * Container artifact configuration. See `agentRuntimeArtifact` below.
     */
    agentRuntimeArtifact?: pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeAgentRuntimeArtifact>;
    /**
     * Unique identifier of the Agent Runtime.
     */
    agentRuntimeId?: pulumi.Input<string>;
    /**
     * Name of the agent runtime.
     */
    agentRuntimeName?: pulumi.Input<string>;
    /**
     * Version of the Agent Runtime.
     */
    agentRuntimeVersion?: pulumi.Input<string>;
    /**
     * Authorization configuration for authenticating incoming requests. See `authorizerConfiguration` below.
     */
    authorizerConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeAuthorizerConfiguration>;
    /**
     * Description of the agent runtime.
     */
    description?: pulumi.Input<string>;
    /**
     * Map of environment variables to pass to the container.
     */
    environmentVariables?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Runtime session and resource lifecycle configuration for the agent runtime. See `lifecycleConfiguration` below.
     */
    lifecycleConfigurations?: pulumi.Input<pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeLifecycleConfiguration>[]>;
    /**
     * Network configuration for the agent runtime. See `networkConfiguration` below.
     *
     * The following arguments are optional:
     */
    networkConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeNetworkConfiguration>;
    /**
     * Protocol configuration for the agent runtime. See `protocolConfiguration` below.
     */
    protocolConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeProtocolConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration for HTTP request headers that will be passed through to the runtime. See `requestHeaderConfiguration` below.
     */
    requestHeaderConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeRequestHeaderConfiguration>;
    /**
     * ARN of the IAM role that the agent runtime assumes to access AWS services.
     */
    roleArn?: pulumi.Input<string>;
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
    timeouts?: pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeTimeouts>;
    /**
     * Workload identity details for the agent runtime. See `workloadIdentityDetails` below.
     */
    workloadIdentityDetails?: pulumi.Input<pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeWorkloadIdentityDetail>[]>;
}
/**
 * The set of arguments for constructing a AgentcoreAgentRuntime resource.
 */
export interface AgentcoreAgentRuntimeArgs {
    /**
     * Container artifact configuration. See `agentRuntimeArtifact` below.
     */
    agentRuntimeArtifact?: pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeAgentRuntimeArtifact>;
    /**
     * Name of the agent runtime.
     */
    agentRuntimeName: pulumi.Input<string>;
    /**
     * Authorization configuration for authenticating incoming requests. See `authorizerConfiguration` below.
     */
    authorizerConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeAuthorizerConfiguration>;
    /**
     * Description of the agent runtime.
     */
    description?: pulumi.Input<string>;
    /**
     * Map of environment variables to pass to the container.
     */
    environmentVariables?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Runtime session and resource lifecycle configuration for the agent runtime. See `lifecycleConfiguration` below.
     */
    lifecycleConfigurations?: pulumi.Input<pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeLifecycleConfiguration>[]>;
    /**
     * Network configuration for the agent runtime. See `networkConfiguration` below.
     *
     * The following arguments are optional:
     */
    networkConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeNetworkConfiguration>;
    /**
     * Protocol configuration for the agent runtime. See `protocolConfiguration` below.
     */
    protocolConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeProtocolConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration for HTTP request headers that will be passed through to the runtime. See `requestHeaderConfiguration` below.
     */
    requestHeaderConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeRequestHeaderConfiguration>;
    /**
     * ARN of the IAM role that the agent runtime assumes to access AWS services.
     */
    roleArn: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.AgentcoreAgentRuntimeTimeouts>;
}
