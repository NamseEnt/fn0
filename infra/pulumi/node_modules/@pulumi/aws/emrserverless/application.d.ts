import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an EMR Serverless Application.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.emrserverless.Application("example", {
 *     name: "example",
 *     releaseLabel: "emr-6.6.0",
 *     type: "hive",
 * });
 * ```
 *
 * ### Initial Capacity Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.emrserverless.Application("example", {
 *     name: "example",
 *     releaseLabel: "emr-6.6.0",
 *     type: "hive",
 *     initialCapacities: [{
 *         initialCapacityType: "HiveDriver",
 *         initialCapacityConfig: {
 *             workerCount: 1,
 *             workerConfiguration: {
 *                 cpu: "2 vCPU",
 *                 memory: "10 GB",
 *             },
 *         },
 *     }],
 * });
 * ```
 *
 * ### Maximum Capacity Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.emrserverless.Application("example", {
 *     name: "example",
 *     releaseLabel: "emr-6.6.0",
 *     type: "hive",
 *     maximumCapacity: {
 *         cpu: "2 vCPU",
 *         memory: "10 GB",
 *     },
 * });
 * ```
 *
 * ### Monitoring Configuration Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.emrserverless.Application("example", {
 *     name: "example",
 *     releaseLabel: "emr-7.1.0",
 *     type: "spark",
 *     monitoringConfiguration: {
 *         cloudwatchLoggingConfiguration: {
 *             enabled: true,
 *             logGroupName: "/aws/emr-serverless/example",
 *             logStreamNamePrefix: "spark-logs",
 *             logTypes: [
 *                 {
 *                     name: "SPARK_DRIVER",
 *                     values: [
 *                         "STDOUT",
 *                         "STDERR",
 *                     ],
 *                 },
 *                 {
 *                     name: "SPARK_EXECUTOR",
 *                     values: ["STDOUT"],
 *                 },
 *             ],
 *         },
 *         managedPersistenceMonitoringConfiguration: {
 *             enabled: true,
 *         },
 *         prometheusMonitoringConfiguration: {
 *             remoteWriteUrl: "https://prometheus-remote-write-endpoint.example.com/api/v1/write",
 *         },
 *     },
 * });
 * ```
 *
 * ### Runtime Configuration Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.emrserverless.Application("example", {
 *     name: "example",
 *     releaseLabel: "emr-6.8.0",
 *     type: "spark",
 *     runtimeConfigurations: [
 *         {
 *             classification: "spark-executor-log4j2",
 *             properties: {
 *                 "rootLogger.level": "error",
 *                 "logger.IdentifierForClass.name": "classpathForSettingLogger",
 *                 "logger.IdentifierForClass.level": "info",
 *             },
 *         },
 *         {
 *             classification: "spark-defaults",
 *             properties: {
 *                 "spark.executor.memory": "1g",
 *                 "spark.executor.cores": "1",
 *             },
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EMR Serverless applications using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:emrserverless/application:Application example id
 * ```
 */
export declare class Application extends pulumi.CustomResource {
    /**
     * Get an existing Application resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ApplicationState, opts?: pulumi.CustomResourceOptions): Application;
    /**
     * Returns true if the given object is an instance of Application.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Application;
    /**
     * The CPU architecture of an application. Valid values are `ARM64` or `X86_64`. Default value is `X86_64`.
     */
    readonly architecture: pulumi.Output<string | undefined>;
    /**
     * ARN of the cluster.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The configuration for an application to automatically start on job submission.
     */
    readonly autoStartConfiguration: pulumi.Output<outputs.emrserverless.ApplicationAutoStartConfiguration>;
    /**
     * The configuration for an application to automatically stop after a certain amount of time being idle.
     */
    readonly autoStopConfiguration: pulumi.Output<outputs.emrserverless.ApplicationAutoStopConfiguration>;
    /**
     * The image configuration applied to all worker types.
     */
    readonly imageConfiguration: pulumi.Output<outputs.emrserverless.ApplicationImageConfiguration>;
    /**
     * The capacity to initialize when the application is created.
     */
    readonly initialCapacities: pulumi.Output<outputs.emrserverless.ApplicationInitialCapacity[] | undefined>;
    /**
     * Enables the interactive use cases to use when running an application.
     */
    readonly interactiveConfiguration: pulumi.Output<outputs.emrserverless.ApplicationInteractiveConfiguration>;
    /**
     * The maximum capacity to allocate when the application is created. This is cumulative across all workers at any given point in time, not just when an application is created. No new resources will be created once any one of the defined limits is hit.
     */
    readonly maximumCapacity: pulumi.Output<outputs.emrserverless.ApplicationMaximumCapacity>;
    /**
     * The configuration setting for monitoring.
     */
    readonly monitoringConfiguration: pulumi.Output<outputs.emrserverless.ApplicationMonitoringConfiguration | undefined>;
    /**
     * The name of the application.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The network configuration for customer VPC connectivity.
     */
    readonly networkConfiguration: pulumi.Output<outputs.emrserverless.ApplicationNetworkConfiguration | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The EMR release version associated with the application.
     */
    readonly releaseLabel: pulumi.Output<string>;
    /**
     * A configuration specification to be used when provisioning an application. A configuration consists of a classification, properties, and optional nested configurations. A classification refers to an application-specific configuration file. Properties are the settings you want to change in that file.
     */
    readonly runtimeConfigurations: pulumi.Output<outputs.emrserverless.ApplicationRuntimeConfiguration[] | undefined>;
    /**
     * Scheduler configuration for batch and streaming jobs running on this application. Supported with release labels `emr-7.0.0` and above. See schedulerConfiguration Arguments below.
     */
    readonly schedulerConfiguration: pulumi.Output<outputs.emrserverless.ApplicationSchedulerConfiguration | undefined>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The type of application you want to start, such as `spark` or `hive`.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a Application resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ApplicationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Application resources.
 */
export interface ApplicationState {
    /**
     * The CPU architecture of an application. Valid values are `ARM64` or `X86_64`. Default value is `X86_64`.
     */
    architecture?: pulumi.Input<string>;
    /**
     * ARN of the cluster.
     */
    arn?: pulumi.Input<string>;
    /**
     * The configuration for an application to automatically start on job submission.
     */
    autoStartConfiguration?: pulumi.Input<inputs.emrserverless.ApplicationAutoStartConfiguration>;
    /**
     * The configuration for an application to automatically stop after a certain amount of time being idle.
     */
    autoStopConfiguration?: pulumi.Input<inputs.emrserverless.ApplicationAutoStopConfiguration>;
    /**
     * The image configuration applied to all worker types.
     */
    imageConfiguration?: pulumi.Input<inputs.emrserverless.ApplicationImageConfiguration>;
    /**
     * The capacity to initialize when the application is created.
     */
    initialCapacities?: pulumi.Input<pulumi.Input<inputs.emrserverless.ApplicationInitialCapacity>[]>;
    /**
     * Enables the interactive use cases to use when running an application.
     */
    interactiveConfiguration?: pulumi.Input<inputs.emrserverless.ApplicationInteractiveConfiguration>;
    /**
     * The maximum capacity to allocate when the application is created. This is cumulative across all workers at any given point in time, not just when an application is created. No new resources will be created once any one of the defined limits is hit.
     */
    maximumCapacity?: pulumi.Input<inputs.emrserverless.ApplicationMaximumCapacity>;
    /**
     * The configuration setting for monitoring.
     */
    monitoringConfiguration?: pulumi.Input<inputs.emrserverless.ApplicationMonitoringConfiguration>;
    /**
     * The name of the application.
     */
    name?: pulumi.Input<string>;
    /**
     * The network configuration for customer VPC connectivity.
     */
    networkConfiguration?: pulumi.Input<inputs.emrserverless.ApplicationNetworkConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The EMR release version associated with the application.
     */
    releaseLabel?: pulumi.Input<string>;
    /**
     * A configuration specification to be used when provisioning an application. A configuration consists of a classification, properties, and optional nested configurations. A classification refers to an application-specific configuration file. Properties are the settings you want to change in that file.
     */
    runtimeConfigurations?: pulumi.Input<pulumi.Input<inputs.emrserverless.ApplicationRuntimeConfiguration>[]>;
    /**
     * Scheduler configuration for batch and streaming jobs running on this application. Supported with release labels `emr-7.0.0` and above. See schedulerConfiguration Arguments below.
     */
    schedulerConfiguration?: pulumi.Input<inputs.emrserverless.ApplicationSchedulerConfiguration>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The type of application you want to start, such as `spark` or `hive`.
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Application resource.
 */
export interface ApplicationArgs {
    /**
     * The CPU architecture of an application. Valid values are `ARM64` or `X86_64`. Default value is `X86_64`.
     */
    architecture?: pulumi.Input<string>;
    /**
     * The configuration for an application to automatically start on job submission.
     */
    autoStartConfiguration?: pulumi.Input<inputs.emrserverless.ApplicationAutoStartConfiguration>;
    /**
     * The configuration for an application to automatically stop after a certain amount of time being idle.
     */
    autoStopConfiguration?: pulumi.Input<inputs.emrserverless.ApplicationAutoStopConfiguration>;
    /**
     * The image configuration applied to all worker types.
     */
    imageConfiguration?: pulumi.Input<inputs.emrserverless.ApplicationImageConfiguration>;
    /**
     * The capacity to initialize when the application is created.
     */
    initialCapacities?: pulumi.Input<pulumi.Input<inputs.emrserverless.ApplicationInitialCapacity>[]>;
    /**
     * Enables the interactive use cases to use when running an application.
     */
    interactiveConfiguration?: pulumi.Input<inputs.emrserverless.ApplicationInteractiveConfiguration>;
    /**
     * The maximum capacity to allocate when the application is created. This is cumulative across all workers at any given point in time, not just when an application is created. No new resources will be created once any one of the defined limits is hit.
     */
    maximumCapacity?: pulumi.Input<inputs.emrserverless.ApplicationMaximumCapacity>;
    /**
     * The configuration setting for monitoring.
     */
    monitoringConfiguration?: pulumi.Input<inputs.emrserverless.ApplicationMonitoringConfiguration>;
    /**
     * The name of the application.
     */
    name?: pulumi.Input<string>;
    /**
     * The network configuration for customer VPC connectivity.
     */
    networkConfiguration?: pulumi.Input<inputs.emrserverless.ApplicationNetworkConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The EMR release version associated with the application.
     */
    releaseLabel: pulumi.Input<string>;
    /**
     * A configuration specification to be used when provisioning an application. A configuration consists of a classification, properties, and optional nested configurations. A classification refers to an application-specific configuration file. Properties are the settings you want to change in that file.
     */
    runtimeConfigurations?: pulumi.Input<pulumi.Input<inputs.emrserverless.ApplicationRuntimeConfiguration>[]>;
    /**
     * Scheduler configuration for batch and streaming jobs running on this application. Supported with release labels `emr-7.0.0` and above. See schedulerConfiguration Arguments below.
     */
    schedulerConfiguration?: pulumi.Input<inputs.emrserverless.ApplicationSchedulerConfiguration>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The type of application you want to start, such as `spark` or `hive`.
     */
    type: pulumi.Input<string>;
}
