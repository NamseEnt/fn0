import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a revision of an ECS task definition to be used in `aws.ecs.Service`.
 *
 * ## Example Usage
 *
 * ### Basic Example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const service = new aws.ecs.TaskDefinition("service", {
 *     family: "service",
 *     containerDefinitions: JSON.stringify([
 *         {
 *             name: "first",
 *             image: "service-first",
 *             cpu: 10,
 *             memory: 512,
 *             essential: true,
 *             portMappings: [{
 *                 containerPort: 80,
 *                 hostPort: 80,
 *             }],
 *         },
 *         {
 *             name: "second",
 *             image: "service-second",
 *             cpu: 10,
 *             memory: 256,
 *             essential: true,
 *             portMappings: [{
 *                 containerPort: 443,
 *                 hostPort: 443,
 *             }],
 *         },
 *     ]),
 *     volumes: [{
 *         name: "service-storage",
 *         hostPath: "/ecs/service-storage",
 *     }],
 *     placementConstraints: [{
 *         type: "memberOf",
 *         expression: "attribute:ecs.availability-zone in [us-west-2a, us-west-2b]",
 *     }],
 * });
 * ```
 *
 * ### With AppMesh Proxy
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const service = new aws.ecs.TaskDefinition("service", {
 *     family: "service",
 *     containerDefinitions: std.file({
 *         input: "task-definitions/service.json",
 *     }).then(invoke => invoke.result),
 *     proxyConfiguration: {
 *         type: "APPMESH",
 *         containerName: "applicationContainerName",
 *         properties: {
 *             AppPorts: "8080",
 *             EgressIgnoredIPs: "169.254.170.2,169.254.169.254",
 *             IgnoredUID: "1337",
 *             ProxyEgressPort: "15001",
 *             ProxyIngressPort: "15000",
 *         },
 *     },
 * });
 * ```
 *
 * ### Example Using `dockerVolumeConfiguration`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const service = new aws.ecs.TaskDefinition("service", {
 *     family: "service",
 *     containerDefinitions: std.file({
 *         input: "task-definitions/service.json",
 *     }).then(invoke => invoke.result),
 *     volumes: [{
 *         name: "service-storage",
 *         dockerVolumeConfiguration: {
 *             scope: "shared",
 *             autoprovision: true,
 *             driver: "local",
 *             driverOpts: {
 *                 type: "nfs",
 *                 device: `${fs.dnsName}:/`,
 *                 o: `addr=${fs.dnsName},rsize=1048576,wsize=1048576,hard,timeo=600,retrans=2,noresvport`,
 *             },
 *         },
 *     }],
 * });
 * ```
 *
 * ### Example Using `efsVolumeConfiguration`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const service = new aws.ecs.TaskDefinition("service", {
 *     family: "service",
 *     containerDefinitions: std.file({
 *         input: "task-definitions/service.json",
 *     }).then(invoke => invoke.result),
 *     volumes: [{
 *         name: "service-storage",
 *         efsVolumeConfiguration: {
 *             fileSystemId: fs.id,
 *             rootDirectory: "/opt/data",
 *             transitEncryption: "ENABLED",
 *             transitEncryptionPort: 2999,
 *             authorizationConfig: {
 *                 accessPointId: test.id,
 *                 iam: "ENABLED",
 *             },
 *         },
 *     }],
 * });
 * ```
 *
 * ### Example Using `fsxWindowsFileServerVolumeConfiguration`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const test = new aws.secretsmanager.SecretVersion("test", {
 *     secretId: testAwsSecretsmanagerSecret.id,
 *     secretString: JSON.stringify({
 *         username: "admin",
 *         password: testAwsDirectoryServiceDirectory.password,
 *     }),
 * });
 * const service = new aws.ecs.TaskDefinition("service", {
 *     family: "service",
 *     containerDefinitions: std.file({
 *         input: "task-definitions/service.json",
 *     }).then(invoke => invoke.result),
 *     volumes: [{
 *         name: "service-storage",
 *         fsxWindowsFileServerVolumeConfiguration: {
 *             fileSystemId: testAwsFsxWindowsFileSystem.id,
 *             rootDirectory: "\\data",
 *             authorizationConfig: {
 *                 credentialsParameter: test.arn,
 *                 domain: testAwsDirectoryServiceDirectory.name,
 *             },
 *         },
 *     }],
 * });
 * ```
 *
 * ### Example Using `containerDefinitions`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.ecs.TaskDefinition("test", {
 *     family: "test",
 *     containerDefinitions: `[
 *   {
 *     "cpu": 10,
 *     "command": ["sleep", "10"],
 *     "entryPoint": ["/"],
 *     "environment": [
 *       {"name": "VARNAME", "value": "VARVAL"}
 *     ],
 *     "essential": true,
 *     "image": "jenkins",
 *     "memory": 128,
 *     "name": "jenkins",
 *     "portMappings": [
 *       {
 *         "containerPort": 80,
 *         "hostPort": 8080
 *       }
 *     ]
 *   }
 * ]
 * `,
 * });
 * ```
 *
 * ### Example Using `runtimePlatform` and `fargate`
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.ecs.TaskDefinition("test", {
 *     family: "test",
 *     requiresCompatibilities: ["FARGATE"],
 *     networkMode: "awsvpc",
 *     cpu: "1024",
 *     memory: "2048",
 *     containerDefinitions: `[
 *   {
 *     "name": "iis",
 *     "image": "mcr.microsoft.com/windows/servercore/iis",
 *     "cpu": 1024,
 *     "memory": 2048,
 *     "essential": true
 *   }
 * ]
 * `,
 *     runtimePlatform: {
 *         operatingSystemFamily: "WINDOWS_SERVER_2019_CORE",
 *         cpuArchitecture: "X86_64",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import ECS Task Definitions using their ARNs. For example:
 *
 * ```sh
 * $ pulumi import aws:ecs/taskDefinition:TaskDefinition example arn:aws:ecs:us-east-1:012345678910:task-definition/mytaskfamily:123
 * ```
 */
export declare class TaskDefinition extends pulumi.CustomResource {
    /**
     * Get an existing TaskDefinition resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TaskDefinitionState, opts?: pulumi.CustomResourceOptions): TaskDefinition;
    /**
     * Returns true if the given object is an instance of TaskDefinition.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TaskDefinition;
    /**
     * Full ARN of the Task Definition (including both `family` and `revision`).
     */
    readonly arn: pulumi.Output<string>;
    /**
     * ARN of the Task Definition with the trailing `revision` removed. This may be useful for situations where the latest task definition is always desired. If a revision isn't specified, the latest ACTIVE revision is used. See the [AWS documentation](https://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_StartTask.html#ECS-StartTask-request-taskDefinition) for details.
     */
    readonly arnWithoutRevision: pulumi.Output<string>;
    /**
     * A list of valid [container definitions](http://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_ContainerDefinition.html) provided as a single valid JSON document. Please note that you should only provide values that are part of the container definition document. For a detailed description of what parameters are available, see the [Task Definition Parameters](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task_definition_parameters.html) section from the official [Developer Guide](https://docs.aws.amazon.com/AmazonECS/latest/developerguide).
     */
    readonly containerDefinitions: pulumi.Output<string>;
    /**
     * Number of cpu units used by the task. If the `requiresCompatibilities` is `FARGATE` this field is required.
     */
    readonly cpu: pulumi.Output<string | undefined>;
    /**
     * Enables fault injection and allows for fault injection requests to be accepted from the task's containers. Default is `false`.
     */
    readonly enableFaultInjection: pulumi.Output<boolean>;
    /**
     * The amount of ephemeral storage to allocate for the task. This parameter is used to expand the total amount of ephemeral storage available, beyond the default amount, for tasks hosted on AWS Fargate. See Ephemeral Storage.
     */
    readonly ephemeralStorage: pulumi.Output<outputs.ecs.TaskDefinitionEphemeralStorage | undefined>;
    /**
     * ARN of the task execution role that the Amazon ECS container agent and the Docker daemon can assume.
     */
    readonly executionRoleArn: pulumi.Output<string | undefined>;
    /**
     * A unique name for your task definition.
     *
     * The following arguments are optional:
     */
    readonly family: pulumi.Output<string>;
    /**
     * IPC resource namespace to be used for the containers in the task The valid values are `host`, `task`, and `none`.
     */
    readonly ipcMode: pulumi.Output<string | undefined>;
    /**
     * Amount (in MiB) of memory used by the task. If the `requiresCompatibilities` is `FARGATE` this field is required.
     */
    readonly memory: pulumi.Output<string | undefined>;
    /**
     * Docker networking mode to use for the containers in the task. Valid values are `none`, `bridge`, `awsvpc`, and `host`.
     */
    readonly networkMode: pulumi.Output<string>;
    /**
     * Process namespace to use for the containers in the task. The valid values are `host` and `task`.
     */
    readonly pidMode: pulumi.Output<string | undefined>;
    /**
     * Configuration block for rules that are taken into consideration during task placement. Maximum number of `placementConstraints` is `10`. Detailed below.
     */
    readonly placementConstraints: pulumi.Output<outputs.ecs.TaskDefinitionPlacementConstraint[] | undefined>;
    /**
     * Configuration block for the App Mesh proxy. Detailed below.
     */
    readonly proxyConfiguration: pulumi.Output<outputs.ecs.TaskDefinitionProxyConfiguration | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Set of launch types required by the task. The valid values are `EC2` and `FARGATE`.
     */
    readonly requiresCompatibilities: pulumi.Output<string[] | undefined>;
    /**
     * Revision of the task in a particular family.
     */
    readonly revision: pulumi.Output<number>;
    /**
     * Configuration block for runtimePlatform that containers in your task may use.
     */
    readonly runtimePlatform: pulumi.Output<outputs.ecs.TaskDefinitionRuntimePlatform | undefined>;
    /**
     * Whether to retain the old revision when the resource is destroyed or replacement is necessary. Default is `false`.
     */
    readonly skipDestroy: pulumi.Output<boolean | undefined>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * ARN of IAM role that allows your Amazon ECS container task to make calls to other AWS services.
     */
    readonly taskRoleArn: pulumi.Output<string | undefined>;
    /**
     * Whether should track latest `ACTIVE` task definition on AWS or the one created with the resource stored in state. Default is `false`. Useful in the event the task definition is modified outside of this resource.
     */
    readonly trackLatest: pulumi.Output<boolean | undefined>;
    /**
     * Repeatable configuration block for volumes that containers in your task may use. Detailed below.
     *
     * > **NOTE:** Proper escaping is required for JSON field values containing quotes (`"`) such as `environment` values. If directly setting the JSON, they should be escaped as `\"` in the JSON,  e.g., `"value": "I \"love\" escaped quotes"`. If using a variable value, they should be escaped as `\\\"` in the variable, e.g., `value = "I \\\"love\\\" escaped quotes"` in the variable and `"value": "${var.myvariable}"` in the JSON.
     *
     * > **Note:** Fault injection only works with tasks using the `awsvpc` or `host` network modes. Fault injection isn't available on Windows.
     */
    readonly volumes: pulumi.Output<outputs.ecs.TaskDefinitionVolume[] | undefined>;
    /**
     * Create a TaskDefinition resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TaskDefinitionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TaskDefinition resources.
 */
export interface TaskDefinitionState {
    /**
     * Full ARN of the Task Definition (including both `family` and `revision`).
     */
    arn?: pulumi.Input<string>;
    /**
     * ARN of the Task Definition with the trailing `revision` removed. This may be useful for situations where the latest task definition is always desired. If a revision isn't specified, the latest ACTIVE revision is used. See the [AWS documentation](https://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_StartTask.html#ECS-StartTask-request-taskDefinition) for details.
     */
    arnWithoutRevision?: pulumi.Input<string>;
    /**
     * A list of valid [container definitions](http://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_ContainerDefinition.html) provided as a single valid JSON document. Please note that you should only provide values that are part of the container definition document. For a detailed description of what parameters are available, see the [Task Definition Parameters](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task_definition_parameters.html) section from the official [Developer Guide](https://docs.aws.amazon.com/AmazonECS/latest/developerguide).
     */
    containerDefinitions?: pulumi.Input<string>;
    /**
     * Number of cpu units used by the task. If the `requiresCompatibilities` is `FARGATE` this field is required.
     */
    cpu?: pulumi.Input<string>;
    /**
     * Enables fault injection and allows for fault injection requests to be accepted from the task's containers. Default is `false`.
     */
    enableFaultInjection?: pulumi.Input<boolean>;
    /**
     * The amount of ephemeral storage to allocate for the task. This parameter is used to expand the total amount of ephemeral storage available, beyond the default amount, for tasks hosted on AWS Fargate. See Ephemeral Storage.
     */
    ephemeralStorage?: pulumi.Input<inputs.ecs.TaskDefinitionEphemeralStorage>;
    /**
     * ARN of the task execution role that the Amazon ECS container agent and the Docker daemon can assume.
     */
    executionRoleArn?: pulumi.Input<string>;
    /**
     * A unique name for your task definition.
     *
     * The following arguments are optional:
     */
    family?: pulumi.Input<string>;
    /**
     * IPC resource namespace to be used for the containers in the task The valid values are `host`, `task`, and `none`.
     */
    ipcMode?: pulumi.Input<string>;
    /**
     * Amount (in MiB) of memory used by the task. If the `requiresCompatibilities` is `FARGATE` this field is required.
     */
    memory?: pulumi.Input<string>;
    /**
     * Docker networking mode to use for the containers in the task. Valid values are `none`, `bridge`, `awsvpc`, and `host`.
     */
    networkMode?: pulumi.Input<string>;
    /**
     * Process namespace to use for the containers in the task. The valid values are `host` and `task`.
     */
    pidMode?: pulumi.Input<string>;
    /**
     * Configuration block for rules that are taken into consideration during task placement. Maximum number of `placementConstraints` is `10`. Detailed below.
     */
    placementConstraints?: pulumi.Input<pulumi.Input<inputs.ecs.TaskDefinitionPlacementConstraint>[]>;
    /**
     * Configuration block for the App Mesh proxy. Detailed below.
     */
    proxyConfiguration?: pulumi.Input<inputs.ecs.TaskDefinitionProxyConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Set of launch types required by the task. The valid values are `EC2` and `FARGATE`.
     */
    requiresCompatibilities?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Revision of the task in a particular family.
     */
    revision?: pulumi.Input<number>;
    /**
     * Configuration block for runtimePlatform that containers in your task may use.
     */
    runtimePlatform?: pulumi.Input<inputs.ecs.TaskDefinitionRuntimePlatform>;
    /**
     * Whether to retain the old revision when the resource is destroyed or replacement is necessary. Default is `false`.
     */
    skipDestroy?: pulumi.Input<boolean>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * ARN of IAM role that allows your Amazon ECS container task to make calls to other AWS services.
     */
    taskRoleArn?: pulumi.Input<string>;
    /**
     * Whether should track latest `ACTIVE` task definition on AWS or the one created with the resource stored in state. Default is `false`. Useful in the event the task definition is modified outside of this resource.
     */
    trackLatest?: pulumi.Input<boolean>;
    /**
     * Repeatable configuration block for volumes that containers in your task may use. Detailed below.
     *
     * > **NOTE:** Proper escaping is required for JSON field values containing quotes (`"`) such as `environment` values. If directly setting the JSON, they should be escaped as `\"` in the JSON,  e.g., `"value": "I \"love\" escaped quotes"`. If using a variable value, they should be escaped as `\\\"` in the variable, e.g., `value = "I \\\"love\\\" escaped quotes"` in the variable and `"value": "${var.myvariable}"` in the JSON.
     *
     * > **Note:** Fault injection only works with tasks using the `awsvpc` or `host` network modes. Fault injection isn't available on Windows.
     */
    volumes?: pulumi.Input<pulumi.Input<inputs.ecs.TaskDefinitionVolume>[]>;
}
/**
 * The set of arguments for constructing a TaskDefinition resource.
 */
export interface TaskDefinitionArgs {
    /**
     * A list of valid [container definitions](http://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_ContainerDefinition.html) provided as a single valid JSON document. Please note that you should only provide values that are part of the container definition document. For a detailed description of what parameters are available, see the [Task Definition Parameters](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task_definition_parameters.html) section from the official [Developer Guide](https://docs.aws.amazon.com/AmazonECS/latest/developerguide).
     */
    containerDefinitions: pulumi.Input<string>;
    /**
     * Number of cpu units used by the task. If the `requiresCompatibilities` is `FARGATE` this field is required.
     */
    cpu?: pulumi.Input<string>;
    /**
     * Enables fault injection and allows for fault injection requests to be accepted from the task's containers. Default is `false`.
     */
    enableFaultInjection?: pulumi.Input<boolean>;
    /**
     * The amount of ephemeral storage to allocate for the task. This parameter is used to expand the total amount of ephemeral storage available, beyond the default amount, for tasks hosted on AWS Fargate. See Ephemeral Storage.
     */
    ephemeralStorage?: pulumi.Input<inputs.ecs.TaskDefinitionEphemeralStorage>;
    /**
     * ARN of the task execution role that the Amazon ECS container agent and the Docker daemon can assume.
     */
    executionRoleArn?: pulumi.Input<string>;
    /**
     * A unique name for your task definition.
     *
     * The following arguments are optional:
     */
    family: pulumi.Input<string>;
    /**
     * IPC resource namespace to be used for the containers in the task The valid values are `host`, `task`, and `none`.
     */
    ipcMode?: pulumi.Input<string>;
    /**
     * Amount (in MiB) of memory used by the task. If the `requiresCompatibilities` is `FARGATE` this field is required.
     */
    memory?: pulumi.Input<string>;
    /**
     * Docker networking mode to use for the containers in the task. Valid values are `none`, `bridge`, `awsvpc`, and `host`.
     */
    networkMode?: pulumi.Input<string>;
    /**
     * Process namespace to use for the containers in the task. The valid values are `host` and `task`.
     */
    pidMode?: pulumi.Input<string>;
    /**
     * Configuration block for rules that are taken into consideration during task placement. Maximum number of `placementConstraints` is `10`. Detailed below.
     */
    placementConstraints?: pulumi.Input<pulumi.Input<inputs.ecs.TaskDefinitionPlacementConstraint>[]>;
    /**
     * Configuration block for the App Mesh proxy. Detailed below.
     */
    proxyConfiguration?: pulumi.Input<inputs.ecs.TaskDefinitionProxyConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Set of launch types required by the task. The valid values are `EC2` and `FARGATE`.
     */
    requiresCompatibilities?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Configuration block for runtimePlatform that containers in your task may use.
     */
    runtimePlatform?: pulumi.Input<inputs.ecs.TaskDefinitionRuntimePlatform>;
    /**
     * Whether to retain the old revision when the resource is destroyed or replacement is necessary. Default is `false`.
     */
    skipDestroy?: pulumi.Input<boolean>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ARN of IAM role that allows your Amazon ECS container task to make calls to other AWS services.
     */
    taskRoleArn?: pulumi.Input<string>;
    /**
     * Whether should track latest `ACTIVE` task definition on AWS or the one created with the resource stored in state. Default is `false`. Useful in the event the task definition is modified outside of this resource.
     */
    trackLatest?: pulumi.Input<boolean>;
    /**
     * Repeatable configuration block for volumes that containers in your task may use. Detailed below.
     *
     * > **NOTE:** Proper escaping is required for JSON field values containing quotes (`"`) such as `environment` values. If directly setting the JSON, they should be escaped as `\"` in the JSON,  e.g., `"value": "I \"love\" escaped quotes"`. If using a variable value, they should be escaped as `\\\"` in the variable, e.g., `value = "I \\\"love\\\" escaped quotes"` in the variable and `"value": "${var.myvariable}"` in the JSON.
     *
     * > **Note:** Fault injection only works with tasks using the `awsvpc` or `host` network modes. Fault injection isn't available on Windows.
     */
    volumes?: pulumi.Input<pulumi.Input<inputs.ecs.TaskDefinitionVolume>[]>;
}
