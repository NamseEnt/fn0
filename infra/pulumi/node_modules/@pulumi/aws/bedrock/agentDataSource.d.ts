import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Agents for Amazon Bedrock Data Source.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.bedrock.AgentDataSource("example", {
 *     knowledgeBaseId: "EMDPPAYPZI",
 *     name: "example",
 *     dataSourceConfiguration: {
 *         type: "S3",
 *         s3Configuration: {
 *             bucketArn: "arn:aws:s3:::example-bucket",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Agents for Amazon Bedrock Data Source using the data source ID and the knowledge base ID. For example:
 *
 * ```sh
 * $ pulumi import aws:bedrock/agentDataSource:AgentDataSource example GWCMFMQF6T,EMDPPAYPZI
 * ```
 */
export declare class AgentDataSource extends pulumi.CustomResource {
    /**
     * Get an existing AgentDataSource resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgentDataSourceState, opts?: pulumi.CustomResourceOptions): AgentDataSource;
    /**
     * Returns true if the given object is an instance of AgentDataSource.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AgentDataSource;
    /**
     * Data deletion policy for a data source. Valid values: `RETAIN`, `DELETE`.
     */
    readonly dataDeletionPolicy: pulumi.Output<string>;
    /**
     * Details about how the data source is stored. See `dataSourceConfiguration` block for details.
     */
    readonly dataSourceConfiguration: pulumi.Output<outputs.bedrock.AgentDataSourceDataSourceConfiguration | undefined>;
    /**
     * Unique identifier of the data source.
     */
    readonly dataSourceId: pulumi.Output<string>;
    /**
     * Description of the data source.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Unique identifier of the knowledge base to which the data source belongs.
     */
    readonly knowledgeBaseId: pulumi.Output<string>;
    /**
     * Name of the data source.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Details about the configuration of the server-side encryption. See `serverSideEncryptionConfiguration` block for details.
     */
    readonly serverSideEncryptionConfiguration: pulumi.Output<outputs.bedrock.AgentDataSourceServerSideEncryptionConfiguration | undefined>;
    readonly timeouts: pulumi.Output<outputs.bedrock.AgentDataSourceTimeouts | undefined>;
    /**
     * Details about the configuration of the server-side encryption. See `vectorIngestionConfiguration` block for details.
     */
    readonly vectorIngestionConfiguration: pulumi.Output<outputs.bedrock.AgentDataSourceVectorIngestionConfiguration | undefined>;
    /**
     * Create a AgentDataSource resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AgentDataSourceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AgentDataSource resources.
 */
export interface AgentDataSourceState {
    /**
     * Data deletion policy for a data source. Valid values: `RETAIN`, `DELETE`.
     */
    dataDeletionPolicy?: pulumi.Input<string>;
    /**
     * Details about how the data source is stored. See `dataSourceConfiguration` block for details.
     */
    dataSourceConfiguration?: pulumi.Input<inputs.bedrock.AgentDataSourceDataSourceConfiguration>;
    /**
     * Unique identifier of the data source.
     */
    dataSourceId?: pulumi.Input<string>;
    /**
     * Description of the data source.
     */
    description?: pulumi.Input<string>;
    /**
     * Unique identifier of the knowledge base to which the data source belongs.
     */
    knowledgeBaseId?: pulumi.Input<string>;
    /**
     * Name of the data source.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Details about the configuration of the server-side encryption. See `serverSideEncryptionConfiguration` block for details.
     */
    serverSideEncryptionConfiguration?: pulumi.Input<inputs.bedrock.AgentDataSourceServerSideEncryptionConfiguration>;
    timeouts?: pulumi.Input<inputs.bedrock.AgentDataSourceTimeouts>;
    /**
     * Details about the configuration of the server-side encryption. See `vectorIngestionConfiguration` block for details.
     */
    vectorIngestionConfiguration?: pulumi.Input<inputs.bedrock.AgentDataSourceVectorIngestionConfiguration>;
}
/**
 * The set of arguments for constructing a AgentDataSource resource.
 */
export interface AgentDataSourceArgs {
    /**
     * Data deletion policy for a data source. Valid values: `RETAIN`, `DELETE`.
     */
    dataDeletionPolicy?: pulumi.Input<string>;
    /**
     * Details about how the data source is stored. See `dataSourceConfiguration` block for details.
     */
    dataSourceConfiguration?: pulumi.Input<inputs.bedrock.AgentDataSourceDataSourceConfiguration>;
    /**
     * Description of the data source.
     */
    description?: pulumi.Input<string>;
    /**
     * Unique identifier of the knowledge base to which the data source belongs.
     */
    knowledgeBaseId: pulumi.Input<string>;
    /**
     * Name of the data source.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Details about the configuration of the server-side encryption. See `serverSideEncryptionConfiguration` block for details.
     */
    serverSideEncryptionConfiguration?: pulumi.Input<inputs.bedrock.AgentDataSourceServerSideEncryptionConfiguration>;
    timeouts?: pulumi.Input<inputs.bedrock.AgentDataSourceTimeouts>;
    /**
     * Details about the configuration of the server-side encryption. See `vectorIngestionConfiguration` block for details.
     */
    vectorIngestionConfiguration?: pulumi.Input<inputs.bedrock.AgentDataSourceVectorIngestionConfiguration>;
}
