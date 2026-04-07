import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Creates a MWAA Environment resource.
 *
 * ## Example Usage
 *
 * A MWAA Environment requires an IAM role (`aws.iam.Role`), two subnets in the private zone (`aws.ec2.Subnet`) and a versioned S3 bucket (`aws.s3.Bucket`).
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.mwaa.Environment("example", {
 *     dagS3Path: "dags/",
 *     executionRoleArn: exampleAwsIamRole.arn,
 *     name: "example",
 *     networkConfiguration: {
 *         securityGroupIds: [exampleAwsSecurityGroup.id],
 *         subnetIds: _private.map(__item => __item.id),
 *     },
 *     sourceBucketArn: exampleAwsS3Bucket.arn,
 * });
 * ```
 *
 * ### Example with Airflow configuration options
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.mwaa.Environment("example", {
 *     airflowConfigurationOptions: {
 *         "core.default_task_retries": "16",
 *         "core.parallelism": "1",
 *     },
 *     dagS3Path: "dags/",
 *     executionRoleArn: exampleAwsIamRole.arn,
 *     name: "example",
 *     networkConfiguration: {
 *         securityGroupIds: [exampleAwsSecurityGroup.id],
 *         subnetIds: _private.map(__item => __item.id),
 *     },
 *     sourceBucketArn: exampleAwsS3Bucket.arn,
 * });
 * ```
 *
 * ### Example with logging configurations
 *
 * Note that Airflow task logs are enabled by default with the `INFO` log level.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.mwaa.Environment("example", {
 *     dagS3Path: "dags/",
 *     executionRoleArn: exampleAwsIamRole.arn,
 *     loggingConfiguration: {
 *         dagProcessingLogs: {
 *             enabled: true,
 *             logLevel: "DEBUG",
 *         },
 *         schedulerLogs: {
 *             enabled: true,
 *             logLevel: "INFO",
 *         },
 *         taskLogs: {
 *             enabled: true,
 *             logLevel: "WARNING",
 *         },
 *         webserverLogs: {
 *             enabled: true,
 *             logLevel: "ERROR",
 *         },
 *         workerLogs: {
 *             enabled: true,
 *             logLevel: "CRITICAL",
 *         },
 *     },
 *     name: "example",
 *     networkConfiguration: {
 *         securityGroupIds: [exampleAwsSecurityGroup.id],
 *         subnetIds: _private.map(__item => __item.id),
 *     },
 *     sourceBucketArn: exampleAwsS3Bucket.arn,
 * });
 * ```
 *
 * ### Example with tags
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.mwaa.Environment("example", {
 *     dagS3Path: "dags/",
 *     executionRoleArn: exampleAwsIamRole.arn,
 *     name: "example",
 *     networkConfiguration: {
 *         securityGroupIds: [exampleAwsSecurityGroup.id],
 *         subnetIds: _private.map(__item => __item.id),
 *     },
 *     sourceBucketArn: exampleAwsS3Bucket.arn,
 *     tags: {
 *         Name: "example",
 *         Environment: "production",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import MWAA Environment using `Name`. For example:
 *
 * ```sh
 * $ pulumi import aws:mwaa/environment:Environment example MyAirflowEnvironment
 * ```
 */
export declare class Environment extends pulumi.CustomResource {
    /**
     * Get an existing Environment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EnvironmentState, opts?: pulumi.CustomResourceOptions): Environment;
    /**
     * Returns true if the given object is an instance of Environment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Environment;
    /**
     * The `airflowConfigurationOptions` parameter specifies airflow override options. Check the [Official documentation](https://docs.aws.amazon.com/mwaa/latest/userguide/configuring-env-variables.html#configuring-env-variables-reference) for all possible configuration options.
     */
    readonly airflowConfigurationOptions: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Airflow version of your environment, will be set by default to the latest version that MWAA supports.
     */
    readonly airflowVersion: pulumi.Output<string>;
    /**
     * The ARN of the MWAA Environment
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The Created At date of the MWAA Environment
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * The relative path to the DAG folder on your Amazon S3 storage bucket. For example, dags. For more information, see [Importing DAGs on Amazon MWAA](https://docs.aws.amazon.com/mwaa/latest/userguide/configuring-dag-import.html).
     */
    readonly dagS3Path: pulumi.Output<string>;
    /**
     * The VPC endpoint for the environment's Amazon RDS database
     * * `logging_configuration[0].<LOG_CONFIGURATION_TYPE>[0].cloud_watch_log_group_arn` - Provides the ARN for the CloudWatch group where the logs will be published
     */
    readonly databaseVpcEndpointService: pulumi.Output<string>;
    /**
     * Defines whether the VPC endpoints configured for the environment are created and managed by the customer or by AWS. If set to `SERVICE`, Amazon MWAA will create and manage the required VPC endpoints in your VPC. If set to `CUSTOMER`, you must create, and manage, the VPC endpoints for your VPC. Defaults to `SERVICE` if not set.
     */
    readonly endpointManagement: pulumi.Output<string>;
    /**
     * Environment class for the cluster. Possible options are `mw1.micro`, `mw1.small`, `mw1.medium`, `mw1.large`. Will be set by default to `mw1.small`. Please check the [AWS Pricing](https://aws.amazon.com/de/managed-workflows-for-apache-airflow/pricing/) for more information about the environment classes.
     */
    readonly environmentClass: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the task execution role that the Amazon MWAA and its environment can assume. Check the [official AWS documentation](https://docs.aws.amazon.com/mwaa/latest/userguide/mwaa-create-role.html) for the detailed role specification.
     */
    readonly executionRoleArn: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of your KMS key that you want to use for encryption. Will be set to the ARN of the managed KMS key `aws/airflow` by default. Please check the [Official Documentation](https://docs.aws.amazon.com/mwaa/latest/userguide/custom-keys-certs.html) for more information.
     */
    readonly kmsKey: pulumi.Output<string | undefined>;
    readonly lastUpdateds: pulumi.Output<outputs.mwaa.EnvironmentLastUpdated[]>;
    /**
     * The Apache Airflow logs you want to send to Amazon CloudWatch Logs. See `loggingConfiguration` Block for details.
     */
    readonly loggingConfiguration: pulumi.Output<outputs.mwaa.EnvironmentLoggingConfiguration>;
    /**
     * The maximum number of web servers that you want to run in your environment. Value need to be between `2` and `5` if `environmentClass` is not `mw1.micro`, `1` otherwise.
     */
    readonly maxWebservers: pulumi.Output<number>;
    /**
     * The maximum number of workers that can be automatically scaled up. Value need to be between `1` and `25`. Will be `10` by default.
     */
    readonly maxWorkers: pulumi.Output<number>;
    /**
     * The minimum number of web servers that you want to run in your environment. Value need to be between `2` and `5` if `environmentClass` is not `mw1.micro`, `1` otherwise.
     */
    readonly minWebservers: pulumi.Output<number>;
    /**
     * The minimum number of workers that you want to run in your environment. Will be `1` by default.
     */
    readonly minWorkers: pulumi.Output<number>;
    /**
     * The name of the Apache Airflow Environment
     */
    readonly name: pulumi.Output<string>;
    /**
     * Specifies the network configuration for your Apache Airflow Environment. This includes two private subnets as well as security groups for the Airflow environment. Each subnet requires internet connection, otherwise the deployment will fail. See `networkConfiguration` Block for details.
     */
    readonly networkConfiguration: pulumi.Output<outputs.mwaa.EnvironmentNetworkConfiguration>;
    /**
     * The plugins.zip file version you want to use.
     */
    readonly pluginsS3ObjectVersion: pulumi.Output<string>;
    /**
     * The relative path to the plugins.zip file on your Amazon S3 storage bucket. For example, plugins.zip. If a relative path is provided in the request, then pluginsS3ObjectVersion is required. For more information, see [Importing DAGs on Amazon MWAA](https://docs.aws.amazon.com/mwaa/latest/userguide/configuring-dag-import.html).
     */
    readonly pluginsS3Path: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The requirements.txt file version you want to use.
     */
    readonly requirementsS3ObjectVersion: pulumi.Output<string>;
    /**
     * The relative path to the requirements.txt file on your Amazon S3 storage bucket. For example, requirements.txt. If a relative path is provided in the request, then requirementsS3ObjectVersion is required. For more information, see [Importing DAGs on Amazon MWAA](https://docs.aws.amazon.com/mwaa/latest/userguide/configuring-dag-import.html).
     */
    readonly requirementsS3Path: pulumi.Output<string | undefined>;
    /**
     * The number of schedulers that you want to run in your environment. v2.0.2 and above accepts `2` - `5`, default `2`. v1.10.12 accepts `1`.
     */
    readonly schedulers: pulumi.Output<number>;
    /**
     * The Service Role ARN of the Amazon MWAA Environment
     */
    readonly serviceRoleArn: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of your Amazon S3 storage bucket. For example, arn:aws:s3:::airflow-mybucketname.
     */
    readonly sourceBucketArn: pulumi.Output<string>;
    /**
     * The version of the startup shell script you want to use. You must specify the version ID that Amazon S3 assigns to the file every time you update the script.
     */
    readonly startupScriptS3ObjectVersion: pulumi.Output<string>;
    /**
     * The relative path to the script hosted in your bucket. The script runs as your environment starts before starting the Apache Airflow process. Use this script to install dependencies, modify configuration options, and set environment variables. See [Using a startup script](https://docs.aws.amazon.com/mwaa/latest/userguide/using-startup-script.html). Supported for environment versions 2.x and later.
     */
    readonly startupScriptS3Path: pulumi.Output<string | undefined>;
    /**
     * The status of the Amazon MWAA Environment
     */
    readonly status: pulumi.Output<string>;
    /**
     * A map of resource tags to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Specifies whether the webserver should be accessible over the internet or via your specified VPC. Possible options: `PRIVATE_ONLY` (default) and `PUBLIC_ONLY`.
     */
    readonly webserverAccessMode: pulumi.Output<string>;
    /**
     * The webserver URL of the MWAA Environment
     */
    readonly webserverUrl: pulumi.Output<string>;
    /**
     * The VPC endpoint for the environment's web server
     */
    readonly webserverVpcEndpointService: pulumi.Output<string>;
    /**
     * Specifies the start date for the weekly maintenance window.
     */
    readonly weeklyMaintenanceWindowStart: pulumi.Output<string>;
    /**
     * Worker replacement strategy. Valid values: `FORCED`, `GRACEFUL`.
     */
    readonly workerReplacementStrategy: pulumi.Output<string>;
    /**
     * Create a Environment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EnvironmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Environment resources.
 */
export interface EnvironmentState {
    /**
     * The `airflowConfigurationOptions` parameter specifies airflow override options. Check the [Official documentation](https://docs.aws.amazon.com/mwaa/latest/userguide/configuring-env-variables.html#configuring-env-variables-reference) for all possible configuration options.
     */
    airflowConfigurationOptions?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Airflow version of your environment, will be set by default to the latest version that MWAA supports.
     */
    airflowVersion?: pulumi.Input<string>;
    /**
     * The ARN of the MWAA Environment
     */
    arn?: pulumi.Input<string>;
    /**
     * The Created At date of the MWAA Environment
     */
    createdAt?: pulumi.Input<string>;
    /**
     * The relative path to the DAG folder on your Amazon S3 storage bucket. For example, dags. For more information, see [Importing DAGs on Amazon MWAA](https://docs.aws.amazon.com/mwaa/latest/userguide/configuring-dag-import.html).
     */
    dagS3Path?: pulumi.Input<string>;
    /**
     * The VPC endpoint for the environment's Amazon RDS database
     * * `logging_configuration[0].<LOG_CONFIGURATION_TYPE>[0].cloud_watch_log_group_arn` - Provides the ARN for the CloudWatch group where the logs will be published
     */
    databaseVpcEndpointService?: pulumi.Input<string>;
    /**
     * Defines whether the VPC endpoints configured for the environment are created and managed by the customer or by AWS. If set to `SERVICE`, Amazon MWAA will create and manage the required VPC endpoints in your VPC. If set to `CUSTOMER`, you must create, and manage, the VPC endpoints for your VPC. Defaults to `SERVICE` if not set.
     */
    endpointManagement?: pulumi.Input<string>;
    /**
     * Environment class for the cluster. Possible options are `mw1.micro`, `mw1.small`, `mw1.medium`, `mw1.large`. Will be set by default to `mw1.small`. Please check the [AWS Pricing](https://aws.amazon.com/de/managed-workflows-for-apache-airflow/pricing/) for more information about the environment classes.
     */
    environmentClass?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the task execution role that the Amazon MWAA and its environment can assume. Check the [official AWS documentation](https://docs.aws.amazon.com/mwaa/latest/userguide/mwaa-create-role.html) for the detailed role specification.
     */
    executionRoleArn?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of your KMS key that you want to use for encryption. Will be set to the ARN of the managed KMS key `aws/airflow` by default. Please check the [Official Documentation](https://docs.aws.amazon.com/mwaa/latest/userguide/custom-keys-certs.html) for more information.
     */
    kmsKey?: pulumi.Input<string>;
    lastUpdateds?: pulumi.Input<pulumi.Input<inputs.mwaa.EnvironmentLastUpdated>[]>;
    /**
     * The Apache Airflow logs you want to send to Amazon CloudWatch Logs. See `loggingConfiguration` Block for details.
     */
    loggingConfiguration?: pulumi.Input<inputs.mwaa.EnvironmentLoggingConfiguration>;
    /**
     * The maximum number of web servers that you want to run in your environment. Value need to be between `2` and `5` if `environmentClass` is not `mw1.micro`, `1` otherwise.
     */
    maxWebservers?: pulumi.Input<number>;
    /**
     * The maximum number of workers that can be automatically scaled up. Value need to be between `1` and `25`. Will be `10` by default.
     */
    maxWorkers?: pulumi.Input<number>;
    /**
     * The minimum number of web servers that you want to run in your environment. Value need to be between `2` and `5` if `environmentClass` is not `mw1.micro`, `1` otherwise.
     */
    minWebservers?: pulumi.Input<number>;
    /**
     * The minimum number of workers that you want to run in your environment. Will be `1` by default.
     */
    minWorkers?: pulumi.Input<number>;
    /**
     * The name of the Apache Airflow Environment
     */
    name?: pulumi.Input<string>;
    /**
     * Specifies the network configuration for your Apache Airflow Environment. This includes two private subnets as well as security groups for the Airflow environment. Each subnet requires internet connection, otherwise the deployment will fail. See `networkConfiguration` Block for details.
     */
    networkConfiguration?: pulumi.Input<inputs.mwaa.EnvironmentNetworkConfiguration>;
    /**
     * The plugins.zip file version you want to use.
     */
    pluginsS3ObjectVersion?: pulumi.Input<string>;
    /**
     * The relative path to the plugins.zip file on your Amazon S3 storage bucket. For example, plugins.zip. If a relative path is provided in the request, then pluginsS3ObjectVersion is required. For more information, see [Importing DAGs on Amazon MWAA](https://docs.aws.amazon.com/mwaa/latest/userguide/configuring-dag-import.html).
     */
    pluginsS3Path?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The requirements.txt file version you want to use.
     */
    requirementsS3ObjectVersion?: pulumi.Input<string>;
    /**
     * The relative path to the requirements.txt file on your Amazon S3 storage bucket. For example, requirements.txt. If a relative path is provided in the request, then requirementsS3ObjectVersion is required. For more information, see [Importing DAGs on Amazon MWAA](https://docs.aws.amazon.com/mwaa/latest/userguide/configuring-dag-import.html).
     */
    requirementsS3Path?: pulumi.Input<string>;
    /**
     * The number of schedulers that you want to run in your environment. v2.0.2 and above accepts `2` - `5`, default `2`. v1.10.12 accepts `1`.
     */
    schedulers?: pulumi.Input<number>;
    /**
     * The Service Role ARN of the Amazon MWAA Environment
     */
    serviceRoleArn?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of your Amazon S3 storage bucket. For example, arn:aws:s3:::airflow-mybucketname.
     */
    sourceBucketArn?: pulumi.Input<string>;
    /**
     * The version of the startup shell script you want to use. You must specify the version ID that Amazon S3 assigns to the file every time you update the script.
     */
    startupScriptS3ObjectVersion?: pulumi.Input<string>;
    /**
     * The relative path to the script hosted in your bucket. The script runs as your environment starts before starting the Apache Airflow process. Use this script to install dependencies, modify configuration options, and set environment variables. See [Using a startup script](https://docs.aws.amazon.com/mwaa/latest/userguide/using-startup-script.html). Supported for environment versions 2.x and later.
     */
    startupScriptS3Path?: pulumi.Input<string>;
    /**
     * The status of the Amazon MWAA Environment
     */
    status?: pulumi.Input<string>;
    /**
     * A map of resource tags to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * Specifies whether the webserver should be accessible over the internet or via your specified VPC. Possible options: `PRIVATE_ONLY` (default) and `PUBLIC_ONLY`.
     */
    webserverAccessMode?: pulumi.Input<string>;
    /**
     * The webserver URL of the MWAA Environment
     */
    webserverUrl?: pulumi.Input<string>;
    /**
     * The VPC endpoint for the environment's web server
     */
    webserverVpcEndpointService?: pulumi.Input<string>;
    /**
     * Specifies the start date for the weekly maintenance window.
     */
    weeklyMaintenanceWindowStart?: pulumi.Input<string>;
    /**
     * Worker replacement strategy. Valid values: `FORCED`, `GRACEFUL`.
     */
    workerReplacementStrategy?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Environment resource.
 */
export interface EnvironmentArgs {
    /**
     * The `airflowConfigurationOptions` parameter specifies airflow override options. Check the [Official documentation](https://docs.aws.amazon.com/mwaa/latest/userguide/configuring-env-variables.html#configuring-env-variables-reference) for all possible configuration options.
     */
    airflowConfigurationOptions?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Airflow version of your environment, will be set by default to the latest version that MWAA supports.
     */
    airflowVersion?: pulumi.Input<string>;
    /**
     * The relative path to the DAG folder on your Amazon S3 storage bucket. For example, dags. For more information, see [Importing DAGs on Amazon MWAA](https://docs.aws.amazon.com/mwaa/latest/userguide/configuring-dag-import.html).
     */
    dagS3Path: pulumi.Input<string>;
    /**
     * Defines whether the VPC endpoints configured for the environment are created and managed by the customer or by AWS. If set to `SERVICE`, Amazon MWAA will create and manage the required VPC endpoints in your VPC. If set to `CUSTOMER`, you must create, and manage, the VPC endpoints for your VPC. Defaults to `SERVICE` if not set.
     */
    endpointManagement?: pulumi.Input<string>;
    /**
     * Environment class for the cluster. Possible options are `mw1.micro`, `mw1.small`, `mw1.medium`, `mw1.large`. Will be set by default to `mw1.small`. Please check the [AWS Pricing](https://aws.amazon.com/de/managed-workflows-for-apache-airflow/pricing/) for more information about the environment classes.
     */
    environmentClass?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the task execution role that the Amazon MWAA and its environment can assume. Check the [official AWS documentation](https://docs.aws.amazon.com/mwaa/latest/userguide/mwaa-create-role.html) for the detailed role specification.
     */
    executionRoleArn: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of your KMS key that you want to use for encryption. Will be set to the ARN of the managed KMS key `aws/airflow` by default. Please check the [Official Documentation](https://docs.aws.amazon.com/mwaa/latest/userguide/custom-keys-certs.html) for more information.
     */
    kmsKey?: pulumi.Input<string>;
    /**
     * The Apache Airflow logs you want to send to Amazon CloudWatch Logs. See `loggingConfiguration` Block for details.
     */
    loggingConfiguration?: pulumi.Input<inputs.mwaa.EnvironmentLoggingConfiguration>;
    /**
     * The maximum number of web servers that you want to run in your environment. Value need to be between `2` and `5` if `environmentClass` is not `mw1.micro`, `1` otherwise.
     */
    maxWebservers?: pulumi.Input<number>;
    /**
     * The maximum number of workers that can be automatically scaled up. Value need to be between `1` and `25`. Will be `10` by default.
     */
    maxWorkers?: pulumi.Input<number>;
    /**
     * The minimum number of web servers that you want to run in your environment. Value need to be between `2` and `5` if `environmentClass` is not `mw1.micro`, `1` otherwise.
     */
    minWebservers?: pulumi.Input<number>;
    /**
     * The minimum number of workers that you want to run in your environment. Will be `1` by default.
     */
    minWorkers?: pulumi.Input<number>;
    /**
     * The name of the Apache Airflow Environment
     */
    name?: pulumi.Input<string>;
    /**
     * Specifies the network configuration for your Apache Airflow Environment. This includes two private subnets as well as security groups for the Airflow environment. Each subnet requires internet connection, otherwise the deployment will fail. See `networkConfiguration` Block for details.
     */
    networkConfiguration: pulumi.Input<inputs.mwaa.EnvironmentNetworkConfiguration>;
    /**
     * The plugins.zip file version you want to use.
     */
    pluginsS3ObjectVersion?: pulumi.Input<string>;
    /**
     * The relative path to the plugins.zip file on your Amazon S3 storage bucket. For example, plugins.zip. If a relative path is provided in the request, then pluginsS3ObjectVersion is required. For more information, see [Importing DAGs on Amazon MWAA](https://docs.aws.amazon.com/mwaa/latest/userguide/configuring-dag-import.html).
     */
    pluginsS3Path?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The requirements.txt file version you want to use.
     */
    requirementsS3ObjectVersion?: pulumi.Input<string>;
    /**
     * The relative path to the requirements.txt file on your Amazon S3 storage bucket. For example, requirements.txt. If a relative path is provided in the request, then requirementsS3ObjectVersion is required. For more information, see [Importing DAGs on Amazon MWAA](https://docs.aws.amazon.com/mwaa/latest/userguide/configuring-dag-import.html).
     */
    requirementsS3Path?: pulumi.Input<string>;
    /**
     * The number of schedulers that you want to run in your environment. v2.0.2 and above accepts `2` - `5`, default `2`. v1.10.12 accepts `1`.
     */
    schedulers?: pulumi.Input<number>;
    /**
     * The Amazon Resource Name (ARN) of your Amazon S3 storage bucket. For example, arn:aws:s3:::airflow-mybucketname.
     */
    sourceBucketArn: pulumi.Input<string>;
    /**
     * The version of the startup shell script you want to use. You must specify the version ID that Amazon S3 assigns to the file every time you update the script.
     */
    startupScriptS3ObjectVersion?: pulumi.Input<string>;
    /**
     * The relative path to the script hosted in your bucket. The script runs as your environment starts before starting the Apache Airflow process. Use this script to install dependencies, modify configuration options, and set environment variables. See [Using a startup script](https://docs.aws.amazon.com/mwaa/latest/userguide/using-startup-script.html). Supported for environment versions 2.x and later.
     */
    startupScriptS3Path?: pulumi.Input<string>;
    /**
     * A map of resource tags to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Specifies whether the webserver should be accessible over the internet or via your specified VPC. Possible options: `PRIVATE_ONLY` (default) and `PUBLIC_ONLY`.
     */
    webserverAccessMode?: pulumi.Input<string>;
    /**
     * Specifies the start date for the weekly maintenance window.
     */
    weeklyMaintenanceWindowStart?: pulumi.Input<string>;
    /**
     * Worker replacement strategy. Valid values: `FORCED`, `GRACEFUL`.
     */
    workerReplacementStrategy?: pulumi.Input<string>;
}
