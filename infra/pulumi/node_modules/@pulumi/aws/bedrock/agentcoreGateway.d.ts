import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Bedrock AgentCore Gateway. With Gateway, developers can convert APIs, Lambda functions, and existing services into Model Context Protocol (MCP)-compatible tools.
 *
 * ## Example Usage
 *
 * ### Gateway with JWT Authorization
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
 *     name: "bedrock-agentcore-gateway-role",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const exampleAgentcoreGateway = new aws.bedrock.AgentcoreGateway("example", {
 *     name: "example-gateway",
 *     roleArn: example.arn,
 *     authorizerType: "CUSTOM_JWT",
 *     authorizerConfiguration: {
 *         customJwtAuthorizer: {
 *             discoveryUrl: "https://accounts.google.com/.well-known/openid-configuration",
 *             allowedAudiences: [
 *                 "test1",
 *                 "test2",
 *             ],
 *         },
 *     },
 *     protocolType: "MCP",
 * });
 * ```
 *
 * ### Gateway with advanced JWT Authorization and MCP Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentcoreGateway("example", {
 *     name: "mcp-gateway",
 *     description: "Gateway for MCP communication",
 *     roleArn: exampleAwsIamRole.arn,
 *     authorizerType: "CUSTOM_JWT",
 *     authorizerConfiguration: {
 *         customJwtAuthorizer: {
 *             discoveryUrl: "https://auth.example.com/.well-known/openid-configuration",
 *             allowedAudiences: [
 *                 "app-client",
 *                 "web-client",
 *             ],
 *             allowedClients: [
 *                 "client-123",
 *                 "client-456",
 *             ],
 *         },
 *     },
 *     protocolType: "MCP",
 *     protocolConfiguration: {
 *         mcp: {
 *             instructions: "Gateway for handling MCP requests",
 *             searchType: "HYBRID",
 *             supportedVersions: [
 *                 "2025-03-26",
 *                 "2025-06-18",
 *             ],
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Bedrock AgentCore Gateway using the gateway ID. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/agentcoreGateway:AgentcoreGateway example GATEWAY1234567890
 * ```
 */
export declare class AgentcoreGateway extends pulumi.CustomResource {
    /**
     * Get an existing AgentcoreGateway resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentcoreGatewayState, opts?: pulumi.CustomResourceOptions): AgentcoreGateway;
    /**
     * Returns true if the given object is an instance of AgentcoreGateway.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AgentcoreGateway;
    /**
     * Configuration for request authorization. Required when `authorizerType` is set to `CUSTOM_JWT`. See `authorizerConfiguration` below.
     */
    readonly authorizerConfiguration: pulumi.Output<outputs.bedrock.AgentcoreGatewayAuthorizerConfiguration | undefined>;
    /**
     * Type of authorizer to use. Valid values: `CUSTOM_JWT`, `AWS_IAM`. When set to `CUSTOM_JWT`, `authorizerConfiguration` block is required.
     */
    readonly authorizerType: pulumi.Output<string>;
    /**
     * Description of the gateway.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Exception level for the gateway. Valid values: `INFO`, `WARN`, `ERROR`.
     */
    readonly exceptionLevel: pulumi.Output<string | undefined>;
    /**
     * ARN of the Gateway.
     */
    readonly gatewayArn: pulumi.Output<string>;
    /**
     * Unique identifier of the Gateway.
     */
    readonly gatewayId: pulumi.Output<string>;
    /**
     * URL endpoint for the gateway.
     */
    readonly gatewayUrl: pulumi.Output<string>;
    /**
     * ARN of the KMS key used to encrypt the gateway data.
     */
    readonly kmsKeyArn: pulumi.Output<string | undefined>;
    /**
     * Name of the gateway.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Protocol-specific configuration for the gateway. See `protocolConfiguration` below.
     */
    readonly protocolConfiguration: pulumi.Output<outputs.bedrock.AgentcoreGatewayProtocolConfiguration | undefined>;
    /**
     * Protocol type for the gateway. Valid values: `MCP`.
     */
    readonly protocolType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the IAM role that the gateway assumes to access AWS services.
     *
     * The following arguments are optional:
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
    readonly timeouts: pulumi.Output<outputs.bedrock.AgentcoreGatewayTimeouts | undefined>;
    /**
     * Workload identity details for the gateway. See `workloadIdentityDetails` below.
     */
    readonly workloadIdentityDetails: pulumi.Output<outputs.bedrock.AgentcoreGatewayWorkloadIdentityDetail[]>;
    /**
     * Create a AgentcoreGateway resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AgentcoreGatewayArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AgentcoreGateway resources.
 */
export interface AgentcoreGatewayState {
    /**
     * Configuration for request authorization. Required when `authorizerType` is set to `CUSTOM_JWT`. See `authorizerConfiguration` below.
     */
    authorizerConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreGatewayAuthorizerConfiguration>;
    /**
     * Type of authorizer to use. Valid values: `CUSTOM_JWT`, `AWS_IAM`. When set to `CUSTOM_JWT`, `authorizerConfiguration` block is required.
     */
    authorizerType?: pulumi.Input<string>;
    /**
     * Description of the gateway.
     */
    description?: pulumi.Input<string>;
    /**
     * Exception level for the gateway. Valid values: `INFO`, `WARN`, `ERROR`.
     */
    exceptionLevel?: pulumi.Input<string>;
    /**
     * ARN of the Gateway.
     */
    gatewayArn?: pulumi.Input<string>;
    /**
     * Unique identifier of the Gateway.
     */
    gatewayId?: pulumi.Input<string>;
    /**
     * URL endpoint for the gateway.
     */
    gatewayUrl?: pulumi.Input<string>;
    /**
     * ARN of the KMS key used to encrypt the gateway data.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Name of the gateway.
     */
    name?: pulumi.Input<string>;
    /**
     * Protocol-specific configuration for the gateway. See `protocolConfiguration` below.
     */
    protocolConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreGatewayProtocolConfiguration>;
    /**
     * Protocol type for the gateway. Valid values: `MCP`.
     */
    protocolType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the IAM role that the gateway assumes to access AWS services.
     *
     * The following arguments are optional:
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
    timeouts?: pulumi.Input<inputs.bedrock.AgentcoreGatewayTimeouts>;
    /**
     * Workload identity details for the gateway. See `workloadIdentityDetails` below.
     */
    workloadIdentityDetails?: pulumi.Input<pulumi.Input<inputs.bedrock.AgentcoreGatewayWorkloadIdentityDetail>[]>;
}
/**
 * The set of arguments for constructing a AgentcoreGateway resource.
 */
export interface AgentcoreGatewayArgs {
    /**
     * Configuration for request authorization. Required when `authorizerType` is set to `CUSTOM_JWT`. See `authorizerConfiguration` below.
     */
    authorizerConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreGatewayAuthorizerConfiguration>;
    /**
     * Type of authorizer to use. Valid values: `CUSTOM_JWT`, `AWS_IAM`. When set to `CUSTOM_JWT`, `authorizerConfiguration` block is required.
     */
    authorizerType: pulumi.Input<string>;
    /**
     * Description of the gateway.
     */
    description?: pulumi.Input<string>;
    /**
     * Exception level for the gateway. Valid values: `INFO`, `WARN`, `ERROR`.
     */
    exceptionLevel?: pulumi.Input<string>;
    /**
     * ARN of the KMS key used to encrypt the gateway data.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Name of the gateway.
     */
    name?: pulumi.Input<string>;
    /**
     * Protocol-specific configuration for the gateway. See `protocolConfiguration` below.
     */
    protocolConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreGatewayProtocolConfiguration>;
    /**
     * Protocol type for the gateway. Valid values: `MCP`.
     */
    protocolType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the IAM role that the gateway assumes to access AWS services.
     *
     * The following arguments are optional:
     */
    roleArn: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.AgentcoreGatewayTimeouts>;
}
