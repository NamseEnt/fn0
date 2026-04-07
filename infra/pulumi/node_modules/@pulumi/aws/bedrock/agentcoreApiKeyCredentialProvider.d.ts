import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentcoreApiKeyCredentialProvider("example", {
 *     name: "example-api-key-provider",
 *     apiKey: "your-api-key-here",
 * });
 * ```
 *
 * ### Write-Only API Key (Recommended for Production)
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentcoreApiKeyCredentialProvider("example", {
 *     name: "example-api-key-provider",
 *     apiKeyWo: "your-api-key-here",
 *     apiKeyWoVersion: 1,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Bedrock AgentCore API Key Credential Provider using the provider name. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/agentcoreApiKeyCredentialProvider:AgentcoreApiKeyCredentialProvider example example-api-key-provider
 * ```
 */
export declare class AgentcoreApiKeyCredentialProvider extends pulumi.CustomResource {
    /**
     * Get an existing AgentcoreApiKeyCredentialProvider resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentcoreApiKeyCredentialProviderState, opts?: pulumi.CustomResourceOptions): AgentcoreApiKeyCredentialProvider;
    /**
     * Returns true if the given object is an instance of AgentcoreApiKeyCredentialProvider.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AgentcoreApiKeyCredentialProvider;
    /**
     * API key value. Cannot be used with `apiKeyWo`. This value will be visible in pulumi preview outputs and logs.
     *
     * **Write-Only API Key (choose one approach):**
     */
    readonly apiKey: pulumi.Output<string | undefined>;
    /**
     * ARN of the AWS Secrets Manager secret containing the API key.
     */
    readonly apiKeySecretArns: pulumi.Output<outputs.bedrock.AgentcoreApiKeyCredentialProviderApiKeySecretArn[]>;
    /**
     * **NOTE:** This field is write-only and its value will not be updated in state as part of read operations.
     * Write-only API key value. Cannot be used with `apiKey`. Must be used together with `apiKeyWoVersion`.
     */
    readonly apiKeyWo: pulumi.Output<string | undefined>;
    /**
     * Used together with `apiKeyWo` to trigger an update. Increment this value when an update to `apiKeyWo` is required.
     */
    readonly apiKeyWoVersion: pulumi.Output<number | undefined>;
    /**
     * ARN of the API Key credential provider.
     */
    readonly credentialProviderArn: pulumi.Output<string>;
    /**
     * Name of the API Key credential provider. Forces replacement when changed.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     *
     * **Standard API Key (choose one approach):**
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a AgentcoreApiKeyCredentialProvider resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: AgentcoreApiKeyCredentialProviderArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AgentcoreApiKeyCredentialProvider resources.
 */
export interface AgentcoreApiKeyCredentialProviderState {
    /**
     * API key value. Cannot be used with `apiKeyWo`. This value will be visible in pulumi preview outputs and logs.
     *
     * **Write-Only API Key (choose one approach):**
     */
    apiKey?: pulumi.Input<string>;
    /**
     * ARN of the AWS Secrets Manager secret containing the API key.
     */
    apiKeySecretArns?: pulumi.Input<pulumi.Input<inputs.bedrock.AgentcoreApiKeyCredentialProviderApiKeySecretArn>[]>;
    /**
     * **NOTE:** This field is write-only and its value will not be updated in state as part of read operations.
     * Write-only API key value. Cannot be used with `apiKey`. Must be used together with `apiKeyWoVersion`.
     */
    apiKeyWo?: pulumi.Input<string>;
    /**
     * Used together with `apiKeyWo` to trigger an update. Increment this value when an update to `apiKeyWo` is required.
     */
    apiKeyWoVersion?: pulumi.Input<number>;
    /**
     * ARN of the API Key credential provider.
     */
    credentialProviderArn?: pulumi.Input<string>;
    /**
     * Name of the API Key credential provider. Forces replacement when changed.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     *
     * **Standard API Key (choose one approach):**
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AgentcoreApiKeyCredentialProvider resource.
 */
export interface AgentcoreApiKeyCredentialProviderArgs {
    /**
     * API key value. Cannot be used with `apiKeyWo`. This value will be visible in pulumi preview outputs and logs.
     *
     * **Write-Only API Key (choose one approach):**
     */
    apiKey?: pulumi.Input<string>;
    /**
     * **NOTE:** This field is write-only and its value will not be updated in state as part of read operations.
     * Write-only API key value. Cannot be used with `apiKey`. Must be used together with `apiKeyWoVersion`.
     */
    apiKeyWo?: pulumi.Input<string>;
    /**
     * Used together with `apiKeyWo` to trigger an update. Increment this value when an update to `apiKeyWo` is required.
     */
    apiKeyWoVersion?: pulumi.Input<number>;
    /**
     * Name of the API Key credential provider. Forces replacement when changed.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     *
     * **Standard API Key (choose one approach):**
     */
    region?: pulumi.Input<string>;
}
