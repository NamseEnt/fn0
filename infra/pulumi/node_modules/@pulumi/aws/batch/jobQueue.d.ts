import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Batch Job Queue resource.
 *
 * ## Example Usage
 *
 * ### Basic Job Queue
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testQueue = new aws.batch.JobQueue("test_queue", {
 *     name: "tf-test-batch-job-queue",
 *     state: "ENABLED",
 *     priority: 1,
 *     computeEnvironmentOrders: [
 *         {
 *             order: 1,
 *             computeEnvironment: testEnvironment1.arn,
 *         },
 *         {
 *             order: 2,
 *             computeEnvironment: testEnvironment2.arn,
 *         },
 *     ],
 * });
 * ```
 *
 * ### Job Queue with a fair share scheduling policy
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.batch.SchedulingPolicy("example", {
 *     name: "example",
 *     fairSharePolicy: {
 *         computeReservation: 1,
 *         shareDecaySeconds: 3600,
 *         shareDistributions: [{
 *             shareIdentifier: "A1*",
 *             weightFactor: 0.1,
 *         }],
 *     },
 * });
 * const exampleJobQueue = new aws.batch.JobQueue("example", {
 *     name: "tf-test-batch-job-queue",
 *     schedulingPolicyArn: example.arn,
 *     state: "ENABLED",
 *     priority: 1,
 *     computeEnvironmentOrders: [
 *         {
 *             order: 1,
 *             computeEnvironment: testEnvironment1.arn,
 *         },
 *         {
 *             order: 2,
 *             computeEnvironment: testEnvironment2.arn,
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the job queue.
 *
 * Using `pulumi import`, import Batch Job Queue using the `arn`. For example:
 *
 * console
 *
 * % pulumi import aws_batch_job_queue.test_queue arn:aws:batch:us-east-1:123456789012:job-queue/sample
 */
export declare class JobQueue extends pulumi.CustomResource {
    /**
     * Get an existing JobQueue resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: JobQueueState, opts?: pulumi.CustomResourceOptions): JobQueue;
    /**
     * Returns true if the given object is an instance of JobQueue.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is JobQueue;
    /**
     * The Amazon Resource Name of the job queue.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The set of compute environments mapped to a job queue and their order relative to each other. The job scheduler uses this parameter to determine which compute environment runs a specific job. Compute environments must be in the VALID state before you can associate them with a job queue. You can associate up to three compute environments with a job queue.
     */
    readonly computeEnvironmentOrders: pulumi.Output<outputs.batch.JobQueueComputeEnvironmentOrder[] | undefined>;
    /**
     * The set of job state time limit actions mapped to a job queue. Specifies an action that AWS Batch will take after the job has remained at the head of the queue in the specified state for longer than the specified time.
     */
    readonly jobStateTimeLimitActions: pulumi.Output<outputs.batch.JobQueueJobStateTimeLimitAction[] | undefined>;
    /**
     * Specifies the name of the job queue.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The priority of the job queue. Job queues with a higher priority
     * are evaluated first when associated with the same compute environment.
     */
    readonly priority: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ARN of the fair share scheduling policy. If this parameter is specified, the job queue uses a fair share scheduling policy. If this parameter isn't specified, the job queue uses a first in, first out (FIFO) scheduling policy. After a job queue is created, you can replace but can't remove the fair share scheduling policy.
     */
    readonly schedulingPolicyArn: pulumi.Output<string | undefined>;
    /**
     * The state of the job queue. Must be one of: `ENABLED` or `DISABLED`
     */
    readonly state: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timeouts: pulumi.Output<outputs.batch.JobQueueTimeouts | undefined>;
    /**
     * Create a JobQueue resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: JobQueueArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering JobQueue resources.
 */
export interface JobQueueState {
    /**
     * The Amazon Resource Name of the job queue.
     */
    arn?: pulumi.Input<string>;
    /**
     * The set of compute environments mapped to a job queue and their order relative to each other. The job scheduler uses this parameter to determine which compute environment runs a specific job. Compute environments must be in the VALID state before you can associate them with a job queue. You can associate up to three compute environments with a job queue.
     */
    computeEnvironmentOrders?: pulumi.Input<pulumi.Input<inputs.batch.JobQueueComputeEnvironmentOrder>[]>;
    /**
     * The set of job state time limit actions mapped to a job queue. Specifies an action that AWS Batch will take after the job has remained at the head of the queue in the specified state for longer than the specified time.
     */
    jobStateTimeLimitActions?: pulumi.Input<pulumi.Input<inputs.batch.JobQueueJobStateTimeLimitAction>[]>;
    /**
     * Specifies the name of the job queue.
     */
    name?: pulumi.Input<string>;
    /**
     * The priority of the job queue. Job queues with a higher priority
     * are evaluated first when associated with the same compute environment.
     */
    priority?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the fair share scheduling policy. If this parameter is specified, the job queue uses a fair share scheduling policy. If this parameter isn't specified, the job queue uses a first in, first out (FIFO) scheduling policy. After a job queue is created, you can replace but can't remove the fair share scheduling policy.
     */
    schedulingPolicyArn?: pulumi.Input<string>;
    /**
     * The state of the job queue. Must be one of: `ENABLED` or `DISABLED`
     */
    state?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timeouts?: pulumi.Input<inputs.batch.JobQueueTimeouts>;
}
/**
 * The set of arguments for constructing a JobQueue resource.
 */
export interface JobQueueArgs {
    /**
     * The set of compute environments mapped to a job queue and their order relative to each other. The job scheduler uses this parameter to determine which compute environment runs a specific job. Compute environments must be in the VALID state before you can associate them with a job queue. You can associate up to three compute environments with a job queue.
     */
    computeEnvironmentOrders?: pulumi.Input<pulumi.Input<inputs.batch.JobQueueComputeEnvironmentOrder>[]>;
    /**
     * The set of job state time limit actions mapped to a job queue. Specifies an action that AWS Batch will take after the job has remained at the head of the queue in the specified state for longer than the specified time.
     */
    jobStateTimeLimitActions?: pulumi.Input<pulumi.Input<inputs.batch.JobQueueJobStateTimeLimitAction>[]>;
    /**
     * Specifies the name of the job queue.
     */
    name?: pulumi.Input<string>;
    /**
     * The priority of the job queue. Job queues with a higher priority
     * are evaluated first when associated with the same compute environment.
     */
    priority: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of the fair share scheduling policy. If this parameter is specified, the job queue uses a fair share scheduling policy. If this parameter isn't specified, the job queue uses a first in, first out (FIFO) scheduling policy. After a job queue is created, you can replace but can't remove the fair share scheduling policy.
     */
    schedulingPolicyArn?: pulumi.Input<string>;
    /**
     * The state of the job queue. Must be one of: `ENABLED` or `DISABLED`
     */
    state: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.batch.JobQueueTimeouts>;
}
