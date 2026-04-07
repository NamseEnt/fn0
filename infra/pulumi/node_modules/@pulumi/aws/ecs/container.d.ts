import * as pulumi from "@pulumi/pulumi";
export interface ContainerDefinition {
    command?: pulumi.Input<string[]>;
    cpu?: pulumi.Input<number>;
    dependsOn?: ContainerDependency[];
    disableNetworking?: pulumi.Input<boolean>;
    dnsSearchDomains?: pulumi.Input<string[]>;
    dnsServers?: pulumi.Input<string[]>;
    dockerLabels?: {
        [label: string]: pulumi.Input<string>;
    };
    dockerSecurityOptions?: pulumi.Input<string[]>;
    entryPoint?: pulumi.Input<string[]>;
    environment?: KeyValuePair[];
    environmentFiles?: EnvironmentFile[];
    essential?: pulumi.Input<boolean>;
    extraHosts?: HostEntry[];
    firelensConfiguration?: FirelensConfiguration;
    healthCheck?: HealthCheck;
    hostname?: pulumi.Input<string>;
    image?: pulumi.Input<string>;
    interactive?: pulumi.Input<boolean>;
    links?: pulumi.Input<string[]>;
    linuxParameters?: LinuxParameters;
    logConfiguration?: LogConfiguration;
    memory?: pulumi.Input<number>;
    memoryReservation?: pulumi.Input<number>;
    mountPoints?: MountPoint[];
    name: pulumi.Input<string>;
    portMappings?: PortMapping[];
    privileged?: pulumi.Input<boolean>;
    pseudoTerminal?: pulumi.Input<boolean>;
    readonlyRootFilesystem?: pulumi.Input<boolean>;
    repositoryCredentials?: RepositoryCredentials;
    resourceRequirements?: ResourceRequirements[];
    secrets?: Secret[];
    startTimeout?: pulumi.Input<number>;
    stopTimeout?: pulumi.Input<number>;
    systemControls?: SystemControl[];
    ulimits?: Ulimit[];
    user?: pulumi.Input<string>;
    volumesFrom?: VolumeFrom[];
    workingDirectory?: pulumi.Input<string>;
}
export interface EnvironmentFile {
    type: "s3";
    value: pulumi.Input<string>;
}
export interface ContainerDependency {
    containerName?: pulumi.Input<string>;
    condition?: pulumi.Input<string>;
}
export interface KeyValuePair {
    name: pulumi.Input<string>;
    value: pulumi.Input<string>;
}
export interface FirelensConfiguration {
    options?: {
        [key: string]: pulumi.Input<string>;
    };
    type: pulumi.Input<string>;
}
/**
 * The health check command and associated configuration parameters for the container. This
 * parameter maps to HealthCheck in the Create a container section of the Docker Remote API and the
 * HEALTHCHECK parameter of docker run.
 *
 * https://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_HealthCheck.html
 */
export interface HealthCheck {
    /**
     * A string array representing the command that the container runs to determine if it is
     * healthy. The string array must start with CMD to execute the command arguments directly, or
     * CMD-SHELL to run the command with the container's default shell. For example:
     *
     * [ "CMD-SHELL", "curl -f http://localhost/ || exit 1" ]
     *
     * An exit code of 0 indicates success, and non-zero exit code indicates failure. For more
     * information, see HealthCheck in the Create a container section of the Docker Remote API.
     */
    command?: pulumi.Input<string[]>;
    /**
     * The time period in seconds between each health check execution. You may specify between 5 and
     * 300 seconds. The default value is 30 seconds.
     */
    interval?: pulumi.Input<number>;
    /**
     * The number of times to retry a failed health check before the container is considered
     * unhealthy. You may specify between 1 and 10 retries. The default value is 3.
     */
    retries?: pulumi.Input<number>;
    /**
     * The optional grace period within which to provide containers time to bootstrap before failed
     * health checks count towards the maximum number of retries. You may specify between 0 and 300
     * seconds. The startPeriod is disabled by default.
     *
     * Note:
     * If a health check succeeds within the startPeriod, then the container is considered healthy
     * and any subsequent failures count toward the maximum number of retries.
     */
    startPeriod?: pulumi.Input<number>;
    /**
     * The time period in seconds to wait for a health check to succeed before it is considered a
     * failure. You may specify between 2 and 60 seconds. The default value is 5.
     */
    timeout?: pulumi.Input<number>;
}
export interface HostEntry {
    hostname: pulumi.Input<string>;
    ipAddress: pulumi.Input<string>;
}
export interface LinuxParameters {
    capabilities?: KernelCapabilities;
    devices?: Device[];
    initProcessEnabled?: pulumi.Input<boolean>;
}
export interface KernelCapabilities {
    add?: KernelCapability[];
    drop?: KernelCapability[];
}
export type KernelCapability = "ALL" | "AUDIT_CONTROL" | "AUDIT_WRITE" | "BLOCK_SUSPEND" | "CHOWN" | "DAC_OVERRIDE" | "DAC_READ_SEARCH" | "FOWNER" | "FSETID" | "IPC_LOCK" | "IPC_OWNER" | "KILL" | "LEASE" | "LINUX_IMMUTABLE" | "MAC_ADMIN" | "MAC_OVERRIDE" | "MKNOD" | "NET_ADMIN" | "NET_BIND_SERVICE" | "NET_BROADCAST" | "NET_RAW" | "SETFCAP" | "SETGID" | "SETPCAP" | "SETUID" | "SYS_ADMIN" | "SYS_BOOT" | "SYS_CHROOT" | "SYS_MODULE" | "SYS_NICE" | "SYS_PACCT" | "SYS_PTRACE" | "SYS_RAWIO" | "SYS_RESOURCE" | "SYS_TIME" | "SYS_TTY_CONFIG" | "SYSLOG" | "WAKE_ALARM";
export interface Device {
    containerPath?: pulumi.Input<string>;
    hostPath: pulumi.Input<string>;
    permissions?: pulumi.Input<string[]>;
}
export interface LogConfiguration {
    logDriver: LogDriver;
    options?: {
        [key: string]: pulumi.Input<string>;
    };
    secretOptions?: {
        name: pulumi.Input<string>;
        valueFrom: pulumi.Input<string>;
    }[];
}
export type LogDriver = "json-file" | "syslog" | "journald" | "gelf" | "fluentd" | "awslogs" | "splunk" | "awsfirelens";
export interface MountPoint {
    containerPath?: pulumi.Input<string>;
    readOnly?: pulumi.Input<boolean>;
    sourceVolume?: pulumi.Input<string>;
}
export interface PortMapping {
    appProtocol?: AppProtocol;
    containerPort?: pulumi.Input<number>;
    containerPortRange?: pulumi.Input<string>;
    hostPort?: pulumi.Input<number>;
    name?: pulumi.Input<string>;
    protocol?: Protocol;
}
export type AppProtocol = "http" | "http2" | "grpc";
export type Protocol = "tcp" | "udp";
export interface Ulimit {
    name: UlimitName;
    hardLimit: pulumi.Input<number>;
    softLimit: pulumi.Input<number>;
}
export type UlimitName = "core" | "cpu" | "data" | "fsize" | "locks" | "memlock" | "msgqueue" | "nice" | "nofile" | "nproc" | "rss" | "rtprio" | "rttime" | "sigpending" | "stack";
export interface VolumeFrom {
    sourceContainer?: pulumi.Input<string>;
    readOnly?: pulumi.Input<boolean>;
}
export interface RepositoryCredentials {
    credentialsParameter: pulumi.Input<string>;
}
/**
 * The type and amount of a resource to assign to a container. The only supported resource is a GPU.
 * For more information, see [Working with GPUs on Amazon
 * ECS](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/ecs-gpu.html) in the Amazon
 * Elastic Container Service Developer Guide.
 *
 * See https://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_ResourceRequirement.html.
 */
export interface ResourceRequirements {
    /**
     * The type of resource to assign to a container. The only supported value is GPU.
     */
    type: pulumi.Input<string>;
    /**
     * The number of physical GPUs the Amazon ECS container agent will reserve for the container.
     * The number of GPUs reserved for all containers in a task should not exceed the number of
     * available GPUs on the container instance the task is launched on.
     */
    value: pulumi.Input<string>;
}
/**
 * An object representing the secret to expose to your container. Secrets can be exposed to a
 * container in the following ways:
 *
 * * To inject sensitive data into your containers as environment variables, use the secrets
 *   container definition parameter.
 *
 * * To reference sensitive information in the log configuration of a container, use the
 *   secretOptions container definition parameter.
 *
 * For more information, see [Specifying Sensitive
 * Data](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/specifying-sensitive-data.html)
 * in the Amazon Elastic Container Service Developer Guide.
 *
 * See https://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_Secret.html.
 */
export interface Secret {
    /**
     * The name of the secret.
     */
    name: pulumi.Input<string>;
    /**
     * The secret to expose to the container. The supported values are either the full ARN of the
     * AWS Secrets Manager secret or the full ARN of the parameter in the AWS Systems Manager
     * Parameter Store.
     *
     * Note: If the AWS Systems Manager Parameter Store parameter exists in the same Region as the
     * task you are launching, then you can use either the full ARN or name of the parameter. If the
     * parameter exists in a different Region, then the full ARN must be specified.
     */
    valueFrom: pulumi.Input<string>;
}
/**
 * A list of namespaced kernel parameters to set in the container. This parameter maps to Sysctls in
 * the [Create a container](https://docs.docker.com/engine/api/v1.35/#operation/ContainerCreate)
 * section of the Docker Remote API and the --sysctl option to docker run.
 *
 * It is not recommended that you specify network-related systemControls parameters for multiple
 * containers in a single task that also uses either the awsvpc or host network mode for the
 * following reasons:
 *
 * * For tasks that use the awsvpc network mode, if you set systemControls for any container, it
 *   applies to all containers in the task. If you set different systemControls for multiple
 *   containers in a single task, the container that is started last determines which systemControls
 *   take effect.
 * * For tasks that use the host network mode, the systemControls parameter applies to the container
 *   instance's kernel parameter as well as that of all containers of any tasks running on that
 *   container instance.
 *
 * See https://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_SystemControl.html.
 */
export interface SystemControl {
    /**
     * The namespaced kernel parameter for which to set a value.
     */
    namespace?: pulumi.Input<string>;
    /**
     * The value for the namespaced kernel parameter specified in namespace.
     */
    value?: pulumi.Input<string>;
}
