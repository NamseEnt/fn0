import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Agents for Amazon Bedrock Knowledge Base.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentKnowledgeBase("example", {
 *     name: "example",
 *     roleArn: exampleAwsIamRole.arn,
 *     knowledgeBaseConfiguration: {
 *         vectorKnowledgeBaseConfiguration: {
 *             embeddingModelArn: "arn:aws:bedrock:us-west-2::foundation-model/amazon.titan-embed-text-v2:0",
 *         },
 *         type: "VECTOR",
 *     },
 *     storageConfiguration: {
 *         type: "OPENSEARCH_SERVERLESS",
 *         opensearchServerlessConfiguration: {
 *             collectionArn: "arn:aws:aoss:us-west-2:123456789012:collection/142bezjddq707i5stcrf",
 *             vectorIndexName: "bedrock-knowledge-base-default-index",
 *             fieldMapping: {
 *                 vectorField: "bedrock-knowledge-base-default-vector",
 *                 textField: "AMAZON_BEDROCK_TEXT_CHUNK",
 *                 metadataField: "AMAZON_BEDROCK_METADATA",
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ### With Supplemental Data Storage Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentKnowledgeBase("example", {
 *     name: "example",
 *     roleArn: exampleAwsIamRole.arn,
 *     knowledgeBaseConfiguration: {
 *         vectorKnowledgeBaseConfiguration: {
 *             embeddingModelArn: "arn:aws:bedrock:us-west-2::foundation-model/amazon.titan-embed-text-v2:0",
 *             embeddingModelConfiguration: {
 *                 bedrockEmbeddingModelConfiguration: {
 *                     dimensions: 1024,
 *                     embeddingDataType: "FLOAT32",
 *                 },
 *             },
 *             supplementalDataStorageConfiguration: {
 *                 storageLocations: [{
 *                     type: "S3",
 *                     s3Location: {
 *                         uri: "s3://my-bucket/chunk-processor/",
 *                     },
 *                 }],
 *             },
 *         },
 *         type: "VECTOR",
 *     },
 *     storageConfiguration: {
 *         type: "OPENSEARCH_SERVERLESS",
 *         opensearchServerlessConfiguration: {
 *             collectionArn: "arn:aws:aoss:us-west-2:123456789012:collection/142bezjddq707i5stcrf",
 *             vectorIndexName: "bedrock-knowledge-base-default-index",
 *             fieldMapping: {
 *                 vectorField: "bedrock-knowledge-base-default-vector",
 *                 textField: "AMAZON_BEDROCK_TEXT_CHUNK",
 *                 metadataField: "AMAZON_BEDROCK_METADATA",
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Agents for Amazon Bedrock Knowledge Base using the knowledge base ID. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/agentKnowledgeBase:AgentKnowledgeBase example EMDPPAYPZI
 * ```
 */
export declare class AgentKnowledgeBase extends pulumi.CustomResource {
    /**
     * Get an existing AgentKnowledgeBase resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentKnowledgeBaseState, opts?: pulumi.CustomResourceOptions): AgentKnowledgeBase;
    /**
     * Returns true if the given object is an instance of AgentKnowledgeBase.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AgentKnowledgeBase;
    /**
     * ARN of the knowledge base.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Time at which the knowledge base was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Description of the knowledge base.
     */
    readonly description: pulumi.Output<string | undefined>;
    readonly failureReasons: pulumi.Output<string[]>;
    /**
     * Details about the embeddings configuration of the knowledge base. See `knowledgeBaseConfiguration` block for details.
     */
    readonly knowledgeBaseConfiguration: pulumi.Output<outputs.bedrock.AgentKnowledgeBaseKnowledgeBaseConfiguration | undefined>;
    /**
     * Name of the knowledge base.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the IAM role with permissions to invoke API operations on the knowledge base.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * Details about the storage configuration of the knowledge base. See `storageConfiguration` block for details.
     *
     * The following arguments are optional:
     */
    readonly storageConfiguration: pulumi.Output<outputs.bedrock.AgentKnowledgeBaseStorageConfiguration | undefined>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timeouts: pulumi.Output<outputs.bedrock.AgentKnowledgeBaseTimeouts | undefined>;
    /**
     * Time at which the knowledge base was last updated.
     */
    readonly updatedAt: pulumi.Output<string>;
    /**
     * Create a AgentKnowledgeBase resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AgentKnowledgeBaseArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AgentKnowledgeBase resources.
 */
export interface AgentKnowledgeBaseState {
    /**
     * ARN of the knowledge base.
     */
    arn?: pulumi.Input<string>;
    /**
     * Time at which the knowledge base was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Description of the knowledge base.
     */
    description?: pulumi.Input<string>;
    failureReasons?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Details about the embeddings configuration of the knowledge base. See `knowledgeBaseConfiguration` block for details.
     */
    knowledgeBaseConfiguration?: pulumi.Input<inputs.bedrock.AgentKnowledgeBaseKnowledgeBaseConfiguration>;
    /**
     * Name of the knowledge base.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the IAM role with permissions to invoke API operations on the knowledge base.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * Details about the storage configuration of the knowledge base. See `storageConfiguration` block for details.
     *
     * The following arguments are optional:
     */
    storageConfiguration?: pulumi.Input<inputs.bedrock.AgentKnowledgeBaseStorageConfiguration>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timeouts?: pulumi.Input<inputs.bedrock.AgentKnowledgeBaseTimeouts>;
    /**
     * Time at which the knowledge base was last updated.
     */
    updatedAt?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AgentKnowledgeBase resource.
 */
export interface AgentKnowledgeBaseArgs {
    /**
     * Description of the knowledge base.
     */
    description?: pulumi.Input<string>;
    /**
     * Details about the embeddings configuration of the knowledge base. See `knowledgeBaseConfiguration` block for details.
     */
    knowledgeBaseConfiguration?: pulumi.Input<inputs.bedrock.AgentKnowledgeBaseKnowledgeBaseConfiguration>;
    /**
     * Name of the knowledge base.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the IAM role with permissions to invoke API operations on the knowledge base.
     */
    roleArn: pulumi.Input<string>;
    /**
     * Details about the storage configuration of the knowledge base. See `storageConfiguration` block for details.
     *
     * The following arguments are optional:
     */
    storageConfiguration?: pulumi.Input<inputs.bedrock.AgentKnowledgeBaseStorageConfiguration>;
    /**
     * Map of tags assigned to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.bedrock.AgentKnowledgeBaseTimeouts>;
}
