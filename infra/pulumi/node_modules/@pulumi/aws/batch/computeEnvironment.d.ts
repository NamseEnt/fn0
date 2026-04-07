import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Creates a AWS Batch compute environment. Compute environments contain the Amazon ECS container instances that are used to run containerized batch jobs.
 *
 * For information about AWS Batch, see [What is AWS Batch?](http://docs.aws.amazon.com/batch/latest/userguide/what-is-batch.html) .
 * For information about compute environment, see [Compute Environments](http://docs.aws.amazon.com/batch/latest/userguide/compute_environments.html) .
 *
 * > **Note:** To prevent a race condition during environment deletion, make sure to set `dependsOn` to the related `aws.iam.RolePolicyAttachment`;
 * otherwise, the policy may be destroyed too soon and the compute environment will then get stuck in the `DELETING` state, see [Troubleshooting AWS Batch](http://docs.aws.amazon.com/batch/latest/userguide/troubleshooting.html) .
 *
 * ## Example Usage
 *
 * ### EC2 Type
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const ec2AssumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["ec2.amazonaws.com"],
 *         }],
 *         actions: ["sts:AssumeRole"],
 *     }],
 * });
 * const ecsInstanceRole = new aws.iam.Role("ecs_instance_role", {
 *     name: "ecs_instance_role",
 *     assumeRolePolicy: ec2AssumeRole.then(ec2AssumeRole => ec2AssumeRole.json),
 * });
 * const ecsInstanceRoleRolePolicyAttachment = new aws.iam.RolePolicyAttachment("ecs_instance_role", {
 *     role: ecsInstanceRole.name,
 *     policyArn: "arn:aws:iam::aws:policy/service-role/AmazonEC2ContainerServiceforEC2Role",
 * });
 * const ecsInstanceRoleInstanceProfile = new aws.iam.InstanceProfile("ecs_instance_role", {
 *     name: "ecs_instance_role",
 *     role: ecsInstanceRole.name,
 * });
 * const batchAssumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["batch.amazonaws.com"],
 *         }],
 *         actions: ["sts:AssumeRole"],
 *     }],
 * });
 * const awsBatchServiceRole = new aws.iam.Role("aws_batch_service_role", {
 *     name: "aws_batch_service_role",
 *     assumeRolePolicy: batchAssumeRole.then(batchAssumeRole => batchAssumeRole.json),
 * });
 * const awsBatchServiceRoleRolePolicyAttachment = new aws.iam.RolePolicyAttachment("aws_batch_service_role", {
 *     role: awsBatchServiceRole.name,
 *     policyArn: "arn:aws:iam::aws:policy/service-role/AWSBatchServiceRole",
 * });
 * const sample = new aws.ec2.SecurityGroup("sample", {
 *     name: "aws_batch_compute_environment_security_group",
 *     egress: [{
 *         fromPort: 0,
 *         toPort: 0,
 *         protocol: "-1",
 *         cidrBlocks: ["0.0.0.0/0"],
 *     }],
 * });
 * const sampleVpc = new aws.ec2.Vpc("sample", {cidrBlock: "10.1.0.0/16"});
 * const sampleSubnet = new aws.ec2.Subnet("sample", {
 *     vpcId: sampleVpc.id,
 *     cidrBlock: "10.1.1.0/24",
 * });
 * const samplePlacementGroup = new aws.ec2.PlacementGroup("sample", {
 *     name: "sample",
 *     strategy: aws.ec2.PlacementStrategy.Cluster,
 * });
 * const sampleComputeEnvironment = new aws.batch.ComputeEnvironment("sample", {
 *     name: "sample",
 *     computeResources: {
 *         instanceRole: ecsInstanceRoleInstanceProfile.arn,
 *         instanceTypes: ["c4.large"],
 *         maxVcpus: 16,
 *         minVcpus: 0,
 *         placementGroup: samplePlacementGroup.name,
 *         securityGroupIds: [sample.id],
 *         subnets: [sampleSubnet.id],
 *         type: "EC2",
 *     },
 *     serviceRole: awsBatchServiceRole.arn,
 *     type: "MANAGED",
 * }, {
 *     dependsOn: [awsBatchServiceRoleRolePolicyAttachment],
 * });
 * ```
 *
 * ### Fargate Type
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const sample = new aws.batch.ComputeEnvironment("sample", {
 *     name: "sample",
 *     computeResources: {
 *         maxVcpus: 16,
 *         securityGroupIds: [sampleAwsSecurityGroup.id],
 *         subnets: [sampleAwsSubnet.id],
 *         type: "FARGATE",
 *     },
 *     serviceRole: awsBatchServiceRoleAwsIamRole.arn,
 *     type: "MANAGED",
 * }, {
 *     dependsOn: [awsBatchServiceRole],
 * });
 * ```
 *
 * ### Setting Update Policy
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const sample = new aws.batch.ComputeEnvironment("sample", {
 *     name: "sample",
 *     computeResources: {
 *         allocationStrategy: "BEST_FIT_PROGRESSIVE",
 *         instanceRole: ecsInstance.arn,
 *         instanceTypes: ["optimal"],
 *         maxVcpus: 4,
 *         minVcpus: 0,
 *         securityGroupIds: [sampleAwsSecurityGroup.id],
 *         subnets: [sampleAwsSubnet.id],
 *         type: "EC2",
 *     },
 *     updatePolicy: {
 *         jobExecutionTimeoutMinutes: 30,
 *         terminateJobsOnUpdate: false,
 *     },
 *     type: "MANAGED",
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the compute environment.
 *
 * Using `pulumi import`, import AWS Batch compute using the `name`. For example:
 *
 * console
 *
 * % pulumi import aws_batch_compute_environment.sample sample
 *
 * [1]: http://docs.aws.amazon.com/batch/latest/userguide/what-is-batch.html
 *
 * [2]: http://docs.aws.amazon.com/batch/latest/userguide/compute_environments.html
 *
 * [3]: http://docs.aws.amazon.com/batch/latest/userguide/troubleshooting.html
 */
export declare class ComputeEnvironment extends pulumi.CustomResource {
    /**
     * Get an existing ComputeEnvironment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ComputeEnvironmentState, opts?: pulumi.CustomResourceOptions): ComputeEnvironment;
    /**
     * Returns true if the given object is an instance of ComputeEnvironment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ComputeEnvironment;
    /**
     * The Amazon Resource Name (ARN) of the compute environment.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Details of the compute resources managed by the compute environment. This parameter is required for managed compute environments. See details below.
     */
    readonly computeResources: pulumi.Output<outputs.batch.ComputeEnvironmentComputeResources>;
    /**
     * The Amazon Resource Name (ARN) of the underlying Amazon ECS cluster used by the compute environment.
     */
    readonly ecsClusterArn: pulumi.Output<string>;
    /**
     * Details for the Amazon EKS cluster that supports the compute environment. See details below.
     */
    readonly eksConfiguration: pulumi.Output<outputs.batch.ComputeEnvironmentEksConfiguration | undefined>;
    /**
     * The name for your compute environment. Up to 128 letters (uppercase and lowercase), numbers, and underscores are allowed. If omitted, the provider will assign a random, unique name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique compute environment name beginning with the specified prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The full Amazon Resource Name (ARN) of the IAM role that allows AWS Batch to make calls to other AWS services on your behalf.
     */
    readonly serviceRole: pulumi.Output<string>;
    /**
     * The state of the compute environment. If the state is `ENABLED`, then the compute environment accepts jobs from a queue and can scale out automatically based on queues. Valid items are `ENABLED` or `DISABLED`. Defaults to `ENABLED`.
     */
    readonly state: pulumi.Output<string | undefined>;
    /**
     * The current status of the compute environment (for example, CREATING or VALID).
     */
    readonly status: pulumi.Output<string>;
    /**
     * A short, human-readable string to provide additional details about the current status of the compute environment.
     */
    readonly statusReason: pulumi.Output<string>;
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
    /**
     * The type of the compute environment. Valid items are `MANAGED` or `UNMANAGED`.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Specifies the infrastructure update policy for the compute environment. See details below.
     */
    readonly updatePolicy: pulumi.Output<outputs.batch.ComputeEnvironmentUpdatePolicy>;
    /**
     * Create a ComputeEnvironment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ComputeEnvironmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ComputeEnvironment resources.
 */
export interface ComputeEnvironmentState {
    /**
     * The Amazon Resource Name (ARN) of the compute environment.
     */
    arn?: pulumi.Input<string>;
    /**
     * Details of the compute resources managed by the compute environment. This parameter is required for managed compute environments. See details below.
     */
    computeResources?: pulumi.Input<inputs.batch.ComputeEnvironmentComputeResources>;
    /**
     * The Amazon Resource Name (ARN) of the underlying Amazon ECS cluster used by the compute environment.
     */
    ecsClusterArn?: pulumi.Input<string>;
    /**
     * Details for the Amazon EKS cluster that supports the compute environment. See details below.
     */
    eksConfiguration?: pulumi.Input<inputs.batch.ComputeEnvironmentEksConfiguration>;
    /**
     * The name for your compute environment. Up to 128 letters (uppercase and lowercase), numbers, and underscores are allowed. If omitted, the provider will assign a random, unique name.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique compute environment name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The full Amazon Resource Name (ARN) of the IAM role that allows AWS Batch to make calls to other AWS services on your behalf.
     */
    serviceRole?: pulumi.Input<string>;
    /**
     * The state of the compute environment. If the state is `ENABLED`, then the compute environment accepts jobs from a queue and can scale out automatically based on queues. Valid items are `ENABLED` or `DISABLED`. Defaults to `ENABLED`.
     */
    state?: pulumi.Input<string>;
    /**
     * The current status of the compute environment (for example, CREATING or VALID).
     */
    status?: pulumi.Input<string>;
    /**
     * A short, human-readable string to provide additional details about the current status of the compute environment.
     */
    statusReason?: pulumi.Input<string>;
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
    /**
     * The type of the compute environment. Valid items are `MANAGED` or `UNMANAGED`.
     */
    type?: pulumi.Input<string>;
    /**
     * Specifies the infrastructure update policy for the compute environment. See details below.
     */
    updatePolicy?: pulumi.Input<inputs.batch.ComputeEnvironmentUpdatePolicy>;
}
/**
 * The set of arguments for constructing a ComputeEnvironment resource.
 */
export interface ComputeEnvironmentArgs {
    /**
     * Details of the compute resources managed by the compute environment. This parameter is required for managed compute environments. See details below.
     */
    computeResources?: pulumi.Input<inputs.batch.ComputeEnvironmentComputeResources>;
    /**
     * Details for the Amazon EKS cluster that supports the compute environment. See details below.
     */
    eksConfiguration?: pulumi.Input<inputs.batch.ComputeEnvironmentEksConfiguration>;
    /**
     * The name for your compute environment. Up to 128 letters (uppercase and lowercase), numbers, and underscores are allowed. If omitted, the provider will assign a random, unique name.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique compute environment name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The full Amazon Resource Name (ARN) of the IAM role that allows AWS Batch to make calls to other AWS services on your behalf.
     */
    serviceRole?: pulumi.Input<string>;
    /**
     * The state of the compute environment. If the state is `ENABLED`, then the compute environment accepts jobs from a queue and can scale out automatically based on queues. Valid items are `ENABLED` or `DISABLED`. Defaults to `ENABLED`.
     */
    state?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The type of the compute environment. Valid items are `MANAGED` or `UNMANAGED`.
     */
    type: pulumi.Input<string>;
    /**
     * Specifies the infrastructure update policy for the compute environment. See details below.
     */
    updatePolicy?: pulumi.Input<inputs.batch.ComputeEnvironmentUpdatePolicy>;
}
