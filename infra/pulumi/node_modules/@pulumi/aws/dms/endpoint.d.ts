import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a DMS (Data Migration Service) endpoint resource. DMS endpoints can be created, updated, deleted, and imported.
 *
 * > **Note:** All arguments including the password will be stored in the raw state as plain-text. ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.dms.Endpoint("test", {
 *     certificateArn: "arn:aws:acm:us-east-1:123456789012:certificate/12345678-1234-1234-1234-123456789012",
 *     databaseName: "test",
 *     endpointId: "test-dms-endpoint-tf",
 *     endpointType: "source",
 *     engineName: "aurora",
 *     extraConnectionAttributes: "",
 *     kmsKeyArn: "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012",
 *     password: "test",
 *     port: 3306,
 *     serverName: "test",
 *     sslMode: "none",
 *     tags: {
 *         Name: "test",
 *     },
 *     username: "test",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import endpoints using the `endpoint_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:dms/endpoint:Endpoint test test-dms-endpoint-tf
 * ```
 */
export declare class Endpoint extends pulumi.CustomResource {
    /**
     * Get an existing Endpoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EndpointState, opts?: pulumi.CustomResourceOptions): Endpoint;
    /**
     * Returns true if the given object is an instance of Endpoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Endpoint;
    /**
     * ARN for the certificate.
     */
    readonly certificateArn: pulumi.Output<string>;
    /**
     * Name of the endpoint database.
     */
    readonly databaseName: pulumi.Output<string | undefined>;
    /**
     * Configuration block for OpenSearch settings. See below.
     */
    readonly elasticsearchSettings: pulumi.Output<outputs.dms.EndpointElasticsearchSettings | undefined>;
    /**
     * ARN for the endpoint.
     */
    readonly endpointArn: pulumi.Output<string>;
    /**
     * Database endpoint identifier. Identifiers must contain from 1 to 255 alphanumeric characters or hyphens, begin with a letter, contain only ASCII letters, digits, and hyphens, not end with a hyphen, and not contain two consecutive hyphens.
     */
    readonly endpointId: pulumi.Output<string>;
    /**
     * Type of endpoint. Valid values are `source`, `target`.
     */
    readonly endpointType: pulumi.Output<string>;
    /**
     * Type of engine for the endpoint. Valid values are `aurora`, `aurora-postgresql`, `aurora-serverless`, `aurora-postgresql-serverless`,`azuredb`, `azure-sql-managed-instance`, `babelfish`, `db2`, `db2-zos`, `docdb`, `dynamodb`, `elasticsearch`, `kafka`, `kinesis`, `mariadb`, `mongodb`, `mysql`, `opensearch`, `oracle`, `postgres`, `redshift`,`redshift-serverless`, `sqlserver`, `neptune` ,`sybase`. Please note that some of engine names are available only for `target` endpoint type (e.g. `redshift`).
     */
    readonly engineName: pulumi.Output<string>;
    /**
     * Additional attributes associated with the connection. For available attributes for a `source` Endpoint, see [Sources for data migration](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Source.html). For available attributes for a `target` Endpoint, see [Targets for data migration](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Target.html).
     */
    readonly extraConnectionAttributes: pulumi.Output<string>;
    /**
     * Configuration block for Kafka settings. See below.
     */
    readonly kafkaSettings: pulumi.Output<outputs.dms.EndpointKafkaSettings | undefined>;
    /**
     * Configuration block for Kinesis settings. See below.
     */
    readonly kinesisSettings: pulumi.Output<outputs.dms.EndpointKinesisSettings | undefined>;
    /**
     * ARN for the KMS key that will be used to encrypt the connection parameters. If you do not specify a value for `kmsKeyArn`, then AWS DMS will use your default encryption key. AWS KMS creates the default encryption key for your AWS account. Your AWS account has a different default encryption key for each AWS region. When `engineName` is `redshift`, `kmsKeyArn` is the KMS Key for the Redshift target and the parameter `redshift_settings.server_side_encryption_kms_key_id` encrypts the S3 intermediate storage.
     *
     * The following arguments are optional:
     */
    readonly kmsKeyArn: pulumi.Output<string>;
    /**
     * Configuration block for MongoDB settings. See below.
     */
    readonly mongodbSettings: pulumi.Output<outputs.dms.EndpointMongodbSettings | undefined>;
    /**
     * Configuration block for MySQL settings. See below.
     */
    readonly mysqlSettings: pulumi.Output<outputs.dms.EndpointMysqlSettings | undefined>;
    /**
     * Configuration block for Oracle settings. See below.
     */
    readonly oracleSettings: pulumi.Output<outputs.dms.EndpointOracleSettings | undefined>;
    /**
     * Password to be used to login to the endpoint database.
     */
    readonly password: pulumi.Output<string | undefined>;
    readonly pauseReplicationTasks: pulumi.Output<boolean | undefined>;
    /**
     * Port used by the endpoint database.
     */
    readonly port: pulumi.Output<number | undefined>;
    /**
     * Configuration block for Postgres settings. See below.
     */
    readonly postgresSettings: pulumi.Output<outputs.dms.EndpointPostgresSettings | undefined>;
    readonly redisSettings: pulumi.Output<outputs.dms.EndpointRedisSettings | undefined>;
    /**
     * Configuration block for Redshift settings. See below.
     */
    readonly redshiftSettings: pulumi.Output<outputs.dms.EndpointRedshiftSettings>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the IAM role that specifies AWS DMS as the trusted entity and has the required permissions to access the value in the Secrets Manager secret referred to by `secretsManagerArn`. The role must allow the `iam:PassRole` action.
     *
     * > **Note:** You can specify one of two sets of values for these permissions. You can specify the values for this setting and `secretsManagerArn`. Or you can specify clear-text values for `username`, `password` , `serverName`, and `port`. You can't specify both.
     */
    readonly secretsManagerAccessRoleArn: pulumi.Output<string | undefined>;
    /**
     * Full ARN, partial ARN, or friendly name of the Secrets Manager secret that contains the endpoint connection details. Supported only when `engineName` is `aurora`, `aurora-postgresql`, `mariadb`, `mongodb`, `mysql`, `oracle`, `postgres`, `redshift`, or `sqlserver`.
     */
    readonly secretsManagerArn: pulumi.Output<string | undefined>;
    /**
     * Host name of the server.
     */
    readonly serverName: pulumi.Output<string | undefined>;
    /**
     * ARN used by the service access IAM role for dynamodb endpoints.
     */
    readonly serviceAccessRole: pulumi.Output<string | undefined>;
    /**
     * SSL mode to use for the connection. Valid values are `none`, `require`, `verify-ca`, `verify-full`
     */
    readonly sslMode: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * User name to be used to login to the endpoint database.
     */
    readonly username: pulumi.Output<string | undefined>;
    /**
     * Create a Endpoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EndpointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Endpoint resources.
 */
export interface EndpointState {
    /**
     * ARN for the certificate.
     */
    certificateArn?: pulumi.Input<string>;
    /**
     * Name of the endpoint database.
     */
    databaseName?: pulumi.Input<string>;
    /**
     * Configuration block for OpenSearch settings. See below.
     */
    elasticsearchSettings?: pulumi.Input<inputs.dms.EndpointElasticsearchSettings>;
    /**
     * ARN for the endpoint.
     */
    endpointArn?: pulumi.Input<string>;
    /**
     * Database endpoint identifier. Identifiers must contain from 1 to 255 alphanumeric characters or hyphens, begin with a letter, contain only ASCII letters, digits, and hyphens, not end with a hyphen, and not contain two consecutive hyphens.
     */
    endpointId?: pulumi.Input<string>;
    /**
     * Type of endpoint. Valid values are `source`, `target`.
     */
    endpointType?: pulumi.Input<string>;
    /**
     * Type of engine for the endpoint. Valid values are `aurora`, `aurora-postgresql`, `aurora-serverless`, `aurora-postgresql-serverless`,`azuredb`, `azure-sql-managed-instance`, `babelfish`, `db2`, `db2-zos`, `docdb`, `dynamodb`, `elasticsearch`, `kafka`, `kinesis`, `mariadb`, `mongodb`, `mysql`, `opensearch`, `oracle`, `postgres`, `redshift`,`redshift-serverless`, `sqlserver`, `neptune` ,`sybase`. Please note that some of engine names are available only for `target` endpoint type (e.g. `redshift`).
     */
    engineName?: pulumi.Input<string>;
    /**
     * Additional attributes associated with the connection. For available attributes for a `source` Endpoint, see [Sources for data migration](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Source.html). For available attributes for a `target` Endpoint, see [Targets for data migration](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Target.html).
     */
    extraConnectionAttributes?: pulumi.Input<string>;
    /**
     * Configuration block for Kafka settings. See below.
     */
    kafkaSettings?: pulumi.Input<inputs.dms.EndpointKafkaSettings>;
    /**
     * Configuration block for Kinesis settings. See below.
     */
    kinesisSettings?: pulumi.Input<inputs.dms.EndpointKinesisSettings>;
    /**
     * ARN for the KMS key that will be used to encrypt the connection parameters. If you do not specify a value for `kmsKeyArn`, then AWS DMS will use your default encryption key. AWS KMS creates the default encryption key for your AWS account. Your AWS account has a different default encryption key for each AWS region. When `engineName` is `redshift`, `kmsKeyArn` is the KMS Key for the Redshift target and the parameter `redshift_settings.server_side_encryption_kms_key_id` encrypts the S3 intermediate storage.
     *
     * The following arguments are optional:
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Configuration block for MongoDB settings. See below.
     */
    mongodbSettings?: pulumi.Input<inputs.dms.EndpointMongodbSettings>;
    /**
     * Configuration block for MySQL settings. See below.
     */
    mysqlSettings?: pulumi.Input<inputs.dms.EndpointMysqlSettings>;
    /**
     * Configuration block for Oracle settings. See below.
     */
    oracleSettings?: pulumi.Input<inputs.dms.EndpointOracleSettings>;
    /**
     * Password to be used to login to the endpoint database.
     */
    password?: pulumi.Input<string>;
    pauseReplicationTasks?: pulumi.Input<boolean>;
    /**
     * Port used by the endpoint database.
     */
    port?: pulumi.Input<number>;
    /**
     * Configuration block for Postgres settings. See below.
     */
    postgresSettings?: pulumi.Input<inputs.dms.EndpointPostgresSettings>;
    redisSettings?: pulumi.Input<inputs.dms.EndpointRedisSettings>;
    /**
     * Configuration block for Redshift settings. See below.
     */
    redshiftSettings?: pulumi.Input<inputs.dms.EndpointRedshiftSettings>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the IAM role that specifies AWS DMS as the trusted entity and has the required permissions to access the value in the Secrets Manager secret referred to by `secretsManagerArn`. The role must allow the `iam:PassRole` action.
     *
     * > **Note:** You can specify one of two sets of values for these permissions. You can specify the values for this setting and `secretsManagerArn`. Or you can specify clear-text values for `username`, `password` , `serverName`, and `port`. You can't specify both.
     */
    secretsManagerAccessRoleArn?: pulumi.Input<string>;
    /**
     * Full ARN, partial ARN, or friendly name of the Secrets Manager secret that contains the endpoint connection details. Supported only when `engineName` is `aurora`, `aurora-postgresql`, `mariadb`, `mongodb`, `mysql`, `oracle`, `postgres`, `redshift`, or `sqlserver`.
     */
    secretsManagerArn?: pulumi.Input<string>;
    /**
     * Host name of the server.
     */
    serverName?: pulumi.Input<string>;
    /**
     * ARN used by the service access IAM role for dynamodb endpoints.
     */
    serviceAccessRole?: pulumi.Input<string>;
    /**
     * SSL mode to use for the connection. Valid values are `none`, `require`, `verify-ca`, `verify-full`
     */
    sslMode?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * User name to be used to login to the endpoint database.
     */
    username?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Endpoint resource.
 */
export interface EndpointArgs {
    /**
     * ARN for the certificate.
     */
    certificateArn?: pulumi.Input<string>;
    /**
     * Name of the endpoint database.
     */
    databaseName?: pulumi.Input<string>;
    /**
     * Configuration block for OpenSearch settings. See below.
     */
    elasticsearchSettings?: pulumi.Input<inputs.dms.EndpointElasticsearchSettings>;
    /**
     * Database endpoint identifier. Identifiers must contain from 1 to 255 alphanumeric characters or hyphens, begin with a letter, contain only ASCII letters, digits, and hyphens, not end with a hyphen, and not contain two consecutive hyphens.
     */
    endpointId: pulumi.Input<string>;
    /**
     * Type of endpoint. Valid values are `source`, `target`.
     */
    endpointType: pulumi.Input<string>;
    /**
     * Type of engine for the endpoint. Valid values are `aurora`, `aurora-postgresql`, `aurora-serverless`, `aurora-postgresql-serverless`,`azuredb`, `azure-sql-managed-instance`, `babelfish`, `db2`, `db2-zos`, `docdb`, `dynamodb`, `elasticsearch`, `kafka`, `kinesis`, `mariadb`, `mongodb`, `mysql`, `opensearch`, `oracle`, `postgres`, `redshift`,`redshift-serverless`, `sqlserver`, `neptune` ,`sybase`. Please note that some of engine names are available only for `target` endpoint type (e.g. `redshift`).
     */
    engineName: pulumi.Input<string>;
    /**
     * Additional attributes associated with the connection. For available attributes for a `source` Endpoint, see [Sources for data migration](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Source.html). For available attributes for a `target` Endpoint, see [Targets for data migration](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Target.html).
     */
    extraConnectionAttributes?: pulumi.Input<string>;
    /**
     * Configuration block for Kafka settings. See below.
     */
    kafkaSettings?: pulumi.Input<inputs.dms.EndpointKafkaSettings>;
    /**
     * Configuration block for Kinesis settings. See below.
     */
    kinesisSettings?: pulumi.Input<inputs.dms.EndpointKinesisSettings>;
    /**
     * ARN for the KMS key that will be used to encrypt the connection parameters. If you do not specify a value for `kmsKeyArn`, then AWS DMS will use your default encryption key. AWS KMS creates the default encryption key for your AWS account. Your AWS account has a different default encryption key for each AWS region. When `engineName` is `redshift`, `kmsKeyArn` is the KMS Key for the Redshift target and the parameter `redshift_settings.server_side_encryption_kms_key_id` encrypts the S3 intermediate storage.
     *
     * The following arguments are optional:
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Configuration block for MongoDB settings. See below.
     */
    mongodbSettings?: pulumi.Input<inputs.dms.EndpointMongodbSettings>;
    /**
     * Configuration block for MySQL settings. See below.
     */
    mysqlSettings?: pulumi.Input<inputs.dms.EndpointMysqlSettings>;
    /**
     * Configuration block for Oracle settings. See below.
     */
    oracleSettings?: pulumi.Input<inputs.dms.EndpointOracleSettings>;
    /**
     * Password to be used to login to the endpoint database.
     */
    password?: pulumi.Input<string>;
    pauseReplicationTasks?: pulumi.Input<boolean>;
    /**
     * Port used by the endpoint database.
     */
    port?: pulumi.Input<number>;
    /**
     * Configuration block for Postgres settings. See below.
     */
    postgresSettings?: pulumi.Input<inputs.dms.EndpointPostgresSettings>;
    redisSettings?: pulumi.Input<inputs.dms.EndpointRedisSettings>;
    /**
     * Configuration block for Redshift settings. See below.
     */
    redshiftSettings?: pulumi.Input<inputs.dms.EndpointRedshiftSettings>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the IAM role that specifies AWS DMS as the trusted entity and has the required permissions to access the value in the Secrets Manager secret referred to by `secretsManagerArn`. The role must allow the `iam:PassRole` action.
     *
     * > **Note:** You can specify one of two sets of values for these permissions. You can specify the values for this setting and `secretsManagerArn`. Or you can specify clear-text values for `username`, `password` , `serverName`, and `port`. You can't specify both.
     */
    secretsManagerAccessRoleArn?: pulumi.Input<string>;
    /**
     * Full ARN, partial ARN, or friendly name of the Secrets Manager secret that contains the endpoint connection details. Supported only when `engineName` is `aurora`, `aurora-postgresql`, `mariadb`, `mongodb`, `mysql`, `oracle`, `postgres`, `redshift`, or `sqlserver`.
     */
    secretsManagerArn?: pulumi.Input<string>;
    /**
     * Host name of the server.
     */
    serverName?: pulumi.Input<string>;
    /**
     * ARN used by the service access IAM role for dynamodb endpoints.
     */
    serviceAccessRole?: pulumi.Input<string>;
    /**
     * SSL mode to use for the connection. Valid values are `none`, `require`, `verify-ca`, `verify-full`
     */
    sslMode?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * User name to be used to login to the endpoint database.
     */
    username?: pulumi.Input<string>;
}
