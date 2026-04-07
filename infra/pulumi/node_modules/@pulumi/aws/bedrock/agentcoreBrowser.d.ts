import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Bedrock AgentCore Browser. Browser provides AI agents with web browsing capabilities, allowing them to navigate websites, extract information, and interact with web content in a controlled environment.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentcoreBrowser("example", {
 *     name: "example-browser",
 *     description: "Browser for web data extraction",
 *     networkConfiguration: {
 *         networkMode: "PUBLIC",
 *     },
 * });
 * ```
 *
 * ### Browser with VPC Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const vpcExample = new aws.bedrock.AgentcoreBrowser("vpc_example", {
 *     name: "vpc-browser",
 *     description: "Browser with VPC configuration",
 *     networkConfiguration: {
 *         networkMode: "VPC",
 *         vpcConfig: {
 *             securityGroups: ["sg-12345678"],
 *             subnets: [
 *                 "subnet-12345678",
 *                 "subnet-87654321",
 *             ],
 *         },
 *     },
 * });
 * ```
 *
 * ### Browser with Execution Role and Recording
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
 *     name: "bedrock-agentcore-browser-role",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const recording = new aws.s3.Bucket("recording", {bucket: "browser-recording-bucket"});
 * const exampleAgentcoreBrowser = new aws.bedrock.AgentcoreBrowser("example", {
 *     name: "example-browser",
 *     description: "Browser with recording enabled",
 *     executionRoleArn: example.arn,
 *     networkConfiguration: {
 *         networkMode: "PUBLIC",
 *     },
 *     recording: {
 *         enabled: true,
 *         s3Location: {
 *             bucket: recording.bucket,
 *             prefix: "browser-sessions/",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Bedrock AgentCore Browser using the browser ID. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/agentcoreBrowser:AgentcoreBrowser example BROWSER1234567890
 * ```
 */
export declare class AgentcoreBrowser extends pulumi.CustomResource {
    /**
     * Get an existing AgentcoreBrowser resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentcoreBrowserState, opts?: pulumi.CustomResourceOptions): AgentcoreBrowser;
    /**
     * Returns true if the given object is an instance of AgentcoreBrowser.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AgentcoreBrowser;
    /**
     * ARN of the Browser.
     */
    readonly browserArn: pulumi.Output<string>;
    /**
     * Unique identifier of the Browser.
     */
    readonly browserId: pulumi.Output<string>;
    /**
     * Description of the browser.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * ARN of the IAM role that the browser assumes for execution.
     */
    readonly executionRoleArn: pulumi.Output<string | undefined>;
    /**
     * Name of the browser.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Network configuration for the browser. See `networkConfiguration` below.
     *
     * The following arguments are optional:
     */
    readonly networkConfiguration: pulumi.Output<outputs.bedrock.AgentcoreBrowserNetworkConfiguration | undefined>;
    /**
     * Recording configuration for browser sessions. See `recording` below.
     */
    readonly recording: pulumi.Output<outputs.bedrock.AgentcoreBrowserRecording | undefined>;
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
    readonly timeouts: pulumi.Output<outputs.bedrock.AgentcoreBrowserTimeouts | undefined>;
    /**
     * Create a AgentcoreBrowser resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: AgentcoreBrowserArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AgentcoreBrowser resources.
 */
export interface AgentcoreBrowserState {
    /**
     * ARN of the Browser.
     */
    browserArn?: pulumi.Input<string>;
    /**
     * Unique identifier of the Browser.
     */
    browserId?: pulumi.Input<string>;
    /**
     * Description of the browser.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the IAM role that the browser assumes for execution.
     */
    executionRoleArn?: pulumi.Input<string>;
    /**
     * Name of the browser.
     */
    name?: pulumi.Input<string>;
    /**
     * Network configuration for the browser. See `networkConfiguration` below.
     *
     * The following arguments are optional:
     */
    networkConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreBrowserNetworkConfiguration>;
    /**
     * Recording configuration for browser sessions. See `recording` below.
     */
    recording?: pulumi.Input<inputs.bedrock.AgentcoreBrowserRecording>;
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
    timeouts?: pulumi.Input<inputs.bedrock.AgentcoreBrowserTimeouts>;
}
/**
 * The set of arguments for constructing a AgentcoreBrowser resource.
 */
export interface AgentcoreBrowserArgs {
    /**
     * Description of the browser.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN of the IAM role that the browser assumes for execution.
     */
    executionRoleArn?: pulumi.Input<string>;
    /**
     * Name of the browser.
     */
    name?: pulumi.Input<string>;
    /**
     * Network configuration for the browser. See `networkConfiguration` below.
     *
     * The following arguments are optional:
     */
    networkConfiguration?: pulumi.Input<inputs.bedrock.AgentcoreBrowserNetworkConfiguration>;
    /**
     * Recording configuration for browser sessions. See `recording` below.
     */
    recording?: pulumi.Input<inputs.bedrock.AgentcoreBrowserRecording>;
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
    timeouts?: pulumi.Input<inputs.bedrock.AgentcoreBrowserTimeouts>;
}
