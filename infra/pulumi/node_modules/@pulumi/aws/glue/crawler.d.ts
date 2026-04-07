import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Glue Crawler. More information can be found in the [AWS Glue Developer Guide](https://docs.aws.amazon.com/glue/latest/dg/add-crawler.html)
 *
 * ## Example Usage
 *
 * ### DynamoDB Target Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Crawler("example", {
 *     databaseName: exampleAwsGlueCatalogDatabase.name,
 *     name: "example",
 *     role: exampleAwsIamRole.arn,
 *     dynamodbTargets: [{
 *         path: "table-name",
 *     }],
 * });
 * ```
 *
 * ### JDBC Target Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Crawler("example", {
 *     databaseName: exampleAwsGlueCatalogDatabase.name,
 *     name: "example",
 *     role: exampleAwsIamRole.arn,
 *     jdbcTargets: [{
 *         connectionName: exampleAwsGlueConnection.name,
 *         path: "database-name/%",
 *     }],
 * });
 * ```
 *
 * ### S3 Target Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Crawler("example", {
 *     databaseName: exampleAwsGlueCatalogDatabase.name,
 *     name: "example",
 *     role: exampleAwsIamRole.arn,
 *     s3Targets: [{
 *         path: `s3://${exampleAwsS3Bucket.bucket}`,
 *     }],
 * });
 * ```
 *
 * ### Catalog Target Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Crawler("example", {
 *     databaseName: exampleAwsGlueCatalogDatabase.name,
 *     name: "example",
 *     role: exampleAwsIamRole.arn,
 *     catalogTargets: [{
 *         databaseName: exampleAwsGlueCatalogDatabase.name,
 *         tables: [exampleAwsGlueCatalogTable.name],
 *     }],
 *     schemaChangePolicy: {
 *         deleteBehavior: "LOG",
 *     },
 *     configuration: `{
 *   \\"Version\\":1.0,
 *   \\"Grouping\\": {
 *     \\"TableGroupingPolicy\\": \\"CombineCompatibleSchemas\\"
 *   }
 * }
 * `,
 * });
 * ```
 *
 * ### MongoDB Target Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Crawler("example", {
 *     databaseName: exampleAwsGlueCatalogDatabase.name,
 *     name: "example",
 *     role: exampleAwsIamRole.arn,
 *     mongodbTargets: [{
 *         connectionName: exampleAwsGlueConnection.name,
 *         path: "database-name/%",
 *     }],
 * });
 * ```
 *
 * ### Configuration Settings Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const eventsCrawler = new aws.glue.Crawler("events_crawler", {
 *     databaseName: glueDatabase.name,
 *     schedule: "cron(0 1 * * ? *)",
 *     name: `events_crawler_${environmentName}`,
 *     role: glueRole.arn,
 *     tags: tags,
 *     configuration: JSON.stringify({
 *         Grouping: {
 *             TableGroupingPolicy: "CombineCompatibleSchemas",
 *         },
 *         CrawlerOutput: {
 *             Partitions: {
 *                 AddOrUpdateBehavior: "InheritFromTable",
 *             },
 *         },
 *         Version: 1,
 *     }),
 *     s3Targets: [{
 *         path: `s3://${dataLakeBucket.bucket}`,
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Glue Crawlers using `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:glue/crawler:Crawler MyJob MyJob
 * ```
 */
export declare class Crawler extends pulumi.CustomResource {
    /**
     * Get an existing Crawler resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CrawlerState, opts?: pulumi.CustomResourceOptions): Crawler;
    /**
     * Returns true if the given object is an instance of Crawler.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Crawler;
    /**
     * The ARN of the crawler
     */
    readonly arn: pulumi.Output<string>;
    /**
     * List of nested AWS Glue Data Catalog target arguments. See Catalog Target below.
     */
    readonly catalogTargets: pulumi.Output<outputs.glue.CrawlerCatalogTarget[] | undefined>;
    /**
     * List of custom classifiers. By default, all AWS classifiers are included in a crawl, but these custom classifiers always override the default classifiers for a given classification.
     */
    readonly classifiers: pulumi.Output<string[] | undefined>;
    /**
     * JSON string of configuration information. For more details see [Setting Crawler Configuration Options](https://docs.aws.amazon.com/glue/latest/dg/crawler-configuration.html).
     */
    readonly configuration: pulumi.Output<string | undefined>;
    /**
     * Glue database where results are written.
     */
    readonly databaseName: pulumi.Output<string>;
    /**
     * List of nested Delta Lake target arguments. See Delta Target below.
     */
    readonly deltaTargets: pulumi.Output<outputs.glue.CrawlerDeltaTarget[] | undefined>;
    /**
     * Description of the crawler.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * List of nested DynamoDB target arguments. See Dynamodb Target below.
     */
    readonly dynamodbTargets: pulumi.Output<outputs.glue.CrawlerDynamodbTarget[] | undefined>;
    /**
     * List of nested Hudi target arguments. See Iceberg Target below.
     */
    readonly hudiTargets: pulumi.Output<outputs.glue.CrawlerHudiTarget[] | undefined>;
    /**
     * List of nested Iceberg target arguments. See Iceberg Target below.
     */
    readonly icebergTargets: pulumi.Output<outputs.glue.CrawlerIcebergTarget[] | undefined>;
    /**
     * List of nested JDBC target arguments. See JDBC Target below.
     */
    readonly jdbcTargets: pulumi.Output<outputs.glue.CrawlerJdbcTarget[] | undefined>;
    /**
     * Specifies Lake Formation configuration settings for the crawler. See Lake Formation Configuration below.
     */
    readonly lakeFormationConfiguration: pulumi.Output<outputs.glue.CrawlerLakeFormationConfiguration | undefined>;
    /**
     * Specifies data lineage configuration settings for the crawler. See Lineage Configuration below.
     */
    readonly lineageConfiguration: pulumi.Output<outputs.glue.CrawlerLineageConfiguration | undefined>;
    /**
     * List of nested MongoDB target arguments. See MongoDB Target below.
     */
    readonly mongodbTargets: pulumi.Output<outputs.glue.CrawlerMongodbTarget[] | undefined>;
    /**
     * Name of the crawler.
     */
    readonly name: pulumi.Output<string>;
    /**
     * A policy that specifies whether to crawl the entire dataset again, or to crawl only folders that were added since the last crawler run.. See Recrawl Policy below.
     */
    readonly recrawlPolicy: pulumi.Output<outputs.glue.CrawlerRecrawlPolicy | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The IAM role friendly name (including path without leading slash), or ARN of an IAM role, used by the crawler to access other resources.
     */
    readonly role: pulumi.Output<string>;
    /**
     * List of nested Amazon S3 target arguments. See S3 Target below.
     */
    readonly s3Targets: pulumi.Output<outputs.glue.CrawlerS3Target[] | undefined>;
    /**
     * A cron expression used to specify the schedule. For more information, see [Time-Based Schedules for Jobs and Crawlers](https://docs.aws.amazon.com/glue/latest/dg/monitor-data-warehouse-schedule.html). For example, to run something every day at 12:15 UTC, you would specify: `cron(15 12 * * ? *)`.
     */
    readonly schedule: pulumi.Output<string | undefined>;
    /**
     * Policy for the crawler's update and deletion behavior. See Schema Change Policy below.
     */
    readonly schemaChangePolicy: pulumi.Output<outputs.glue.CrawlerSchemaChangePolicy | undefined>;
    /**
     * The name of Security Configuration to be used by the crawler
     */
    readonly securityConfiguration: pulumi.Output<string | undefined>;
    /**
     * The table prefix used for catalog tables that are created.
     */
    readonly tablePrefix: pulumi.Output<string | undefined>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * > **NOTE:** Must specify at least one of `dynamodbTarget`, `jdbcTarget`, `s3Target`, `mongodbTarget` or `catalogTarget`.
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
    /**
     * Create a Crawler resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CrawlerArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Crawler resources.
 */
export interface CrawlerState {
    /**
     * The ARN of the crawler
     */
    arn?: pulumi.Input<string>;
    /**
     * List of nested AWS Glue Data Catalog target arguments. See Catalog Target below.
     */
    catalogTargets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerCatalogTarget>[]>;
    /**
     * List of custom classifiers. By default, all AWS classifiers are included in a crawl, but these custom classifiers always override the default classifiers for a given classification.
     */
    classifiers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * JSON string of configuration information. For more details see [Setting Crawler Configuration Options](https://docs.aws.amazon.com/glue/latest/dg/crawler-configuration.html).
     */
    configuration?: pulumi.Input<string>;
    /**
     * Glue database where results are written.
     */
    databaseName?: pulumi.Input<string>;
    /**
     * List of nested Delta Lake target arguments. See Delta Target below.
     */
    deltaTargets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerDeltaTarget>[]>;
    /**
     * Description of the crawler.
     */
    description?: pulumi.Input<string>;
    /**
     * List of nested DynamoDB target arguments. See Dynamodb Target below.
     */
    dynamodbTargets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerDynamodbTarget>[]>;
    /**
     * List of nested Hudi target arguments. See Iceberg Target below.
     */
    hudiTargets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerHudiTarget>[]>;
    /**
     * List of nested Iceberg target arguments. See Iceberg Target below.
     */
    icebergTargets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerIcebergTarget>[]>;
    /**
     * List of nested JDBC target arguments. See JDBC Target below.
     */
    jdbcTargets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerJdbcTarget>[]>;
    /**
     * Specifies Lake Formation configuration settings for the crawler. See Lake Formation Configuration below.
     */
    lakeFormationConfiguration?: pulumi.Input<inputs.glue.CrawlerLakeFormationConfiguration>;
    /**
     * Specifies data lineage configuration settings for the crawler. See Lineage Configuration below.
     */
    lineageConfiguration?: pulumi.Input<inputs.glue.CrawlerLineageConfiguration>;
    /**
     * List of nested MongoDB target arguments. See MongoDB Target below.
     */
    mongodbTargets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerMongodbTarget>[]>;
    /**
     * Name of the crawler.
     */
    name?: pulumi.Input<string>;
    /**
     * A policy that specifies whether to crawl the entire dataset again, or to crawl only folders that were added since the last crawler run.. See Recrawl Policy below.
     */
    recrawlPolicy?: pulumi.Input<inputs.glue.CrawlerRecrawlPolicy>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM role friendly name (including path without leading slash), or ARN of an IAM role, used by the crawler to access other resources.
     */
    role?: pulumi.Input<string>;
    /**
     * List of nested Amazon S3 target arguments. See S3 Target below.
     */
    s3Targets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerS3Target>[]>;
    /**
     * A cron expression used to specify the schedule. For more information, see [Time-Based Schedules for Jobs and Crawlers](https://docs.aws.amazon.com/glue/latest/dg/monitor-data-warehouse-schedule.html). For example, to run something every day at 12:15 UTC, you would specify: `cron(15 12 * * ? *)`.
     */
    schedule?: pulumi.Input<string>;
    /**
     * Policy for the crawler's update and deletion behavior. See Schema Change Policy below.
     */
    schemaChangePolicy?: pulumi.Input<inputs.glue.CrawlerSchemaChangePolicy>;
    /**
     * The name of Security Configuration to be used by the crawler
     */
    securityConfiguration?: pulumi.Input<string>;
    /**
     * The table prefix used for catalog tables that are created.
     */
    tablePrefix?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * > **NOTE:** Must specify at least one of `dynamodbTarget`, `jdbcTarget`, `s3Target`, `mongodbTarget` or `catalogTarget`.
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
}
/**
 * The set of arguments for constructing a Crawler resource.
 */
export interface CrawlerArgs {
    /**
     * List of nested AWS Glue Data Catalog target arguments. See Catalog Target below.
     */
    catalogTargets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerCatalogTarget>[]>;
    /**
     * List of custom classifiers. By default, all AWS classifiers are included in a crawl, but these custom classifiers always override the default classifiers for a given classification.
     */
    classifiers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * JSON string of configuration information. For more details see [Setting Crawler Configuration Options](https://docs.aws.amazon.com/glue/latest/dg/crawler-configuration.html).
     */
    configuration?: pulumi.Input<string>;
    /**
     * Glue database where results are written.
     */
    databaseName: pulumi.Input<string>;
    /**
     * List of nested Delta Lake target arguments. See Delta Target below.
     */
    deltaTargets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerDeltaTarget>[]>;
    /**
     * Description of the crawler.
     */
    description?: pulumi.Input<string>;
    /**
     * List of nested DynamoDB target arguments. See Dynamodb Target below.
     */
    dynamodbTargets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerDynamodbTarget>[]>;
    /**
     * List of nested Hudi target arguments. See Iceberg Target below.
     */
    hudiTargets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerHudiTarget>[]>;
    /**
     * List of nested Iceberg target arguments. See Iceberg Target below.
     */
    icebergTargets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerIcebergTarget>[]>;
    /**
     * List of nested JDBC target arguments. See JDBC Target below.
     */
    jdbcTargets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerJdbcTarget>[]>;
    /**
     * Specifies Lake Formation configuration settings for the crawler. See Lake Formation Configuration below.
     */
    lakeFormationConfiguration?: pulumi.Input<inputs.glue.CrawlerLakeFormationConfiguration>;
    /**
     * Specifies data lineage configuration settings for the crawler. See Lineage Configuration below.
     */
    lineageConfiguration?: pulumi.Input<inputs.glue.CrawlerLineageConfiguration>;
    /**
     * List of nested MongoDB target arguments. See MongoDB Target below.
     */
    mongodbTargets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerMongodbTarget>[]>;
    /**
     * Name of the crawler.
     */
    name?: pulumi.Input<string>;
    /**
     * A policy that specifies whether to crawl the entire dataset again, or to crawl only folders that were added since the last crawler run.. See Recrawl Policy below.
     */
    recrawlPolicy?: pulumi.Input<inputs.glue.CrawlerRecrawlPolicy>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM role friendly name (including path without leading slash), or ARN of an IAM role, used by the crawler to access other resources.
     */
    role: pulumi.Input<string>;
    /**
     * List of nested Amazon S3 target arguments. See S3 Target below.
     */
    s3Targets?: pulumi.Input<pulumi.Input<inputs.glue.CrawlerS3Target>[]>;
    /**
     * A cron expression used to specify the schedule. For more information, see [Time-Based Schedules for Jobs and Crawlers](https://docs.aws.amazon.com/glue/latest/dg/monitor-data-warehouse-schedule.html). For example, to run something every day at 12:15 UTC, you would specify: `cron(15 12 * * ? *)`.
     */
    schedule?: pulumi.Input<string>;
    /**
     * Policy for the crawler's update and deletion behavior. See Schema Change Policy below.
     */
    schemaChangePolicy?: pulumi.Input<inputs.glue.CrawlerSchemaChangePolicy>;
    /**
     * The name of Security Configuration to be used by the crawler
     */
    securityConfiguration?: pulumi.Input<string>;
    /**
     * The table prefix used for catalog tables that are created.
     */
    tablePrefix?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * > **NOTE:** Must specify at least one of `dynamodbTarget`, `jdbcTarget`, `s3Target`, `mongodbTarget` or `catalogTarget`.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
