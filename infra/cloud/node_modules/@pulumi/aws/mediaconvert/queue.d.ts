import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS Elemental MediaConvert Queue.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.mediaconvert.Queue("test", {name: "tf-test-queue"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Media Convert Queue using the queue name. For example:
 *
 * ```sh
 * $ pulumi import aws:mediaconvert/queue:Queue test tf-test-queue
 * ```
 */
export declare class Queue extends pulumi.CustomResource {
    /**
     * Get an existing Queue resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: QueueState, opts?: pulumi.CustomResourceOptions): Queue;
    /**
     * Returns true if the given object is an instance of Queue.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Queue;
    /**
     * The Arn of the queue
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The maximum number of jobs your queue can process concurrently. For on-demand queues, the value you enter is constrained by your service quotas for Maximum concurrent jobs, per on-demand queue and Maximum concurrent jobs, per account. For reserved queues, specify the number of jobs you can process concurrently in your reservation plan instead.
     */
    readonly concurrentJobs: pulumi.Output<number>;
    /**
     * A description of the queue
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * A unique identifier describing the queue
     */
    readonly name: pulumi.Output<string>;
    /**
     * Specifies whether the pricing plan for the queue is on-demand or reserved. Valid values are `ON_DEMAND` or `RESERVED`. Default to `ON_DEMAND`.
     */
    readonly pricingPlan: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A detail pricing plan of the  reserved queue. See below.
     */
    readonly reservationPlanSettings: pulumi.Output<outputs.mediaconvert.QueueReservationPlanSettings>;
    /**
     * A status of the queue. Valid values are `ACTIVE` or `RESERVED`. Default to `PAUSED`.
     */
    readonly status: pulumi.Output<string | undefined>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Queue resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: QueueArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Queue resources.
 */
export interface QueueState {
    /**
     * The Arn of the queue
     */
    arn?: pulumi.Input<string>;
    /**
     * The maximum number of jobs your queue can process concurrently. For on-demand queues, the value you enter is constrained by your service quotas for Maximum concurrent jobs, per on-demand queue and Maximum concurrent jobs, per account. For reserved queues, specify the number of jobs you can process concurrently in your reservation plan instead.
     */
    concurrentJobs?: pulumi.Input<number>;
    /**
     * A description of the queue
     */
    description?: pulumi.Input<string>;
    /**
     * A unique identifier describing the queue
     */
    name?: pulumi.Input<string>;
    /**
     * Specifies whether the pricing plan for the queue is on-demand or reserved. Valid values are `ON_DEMAND` or `RESERVED`. Default to `ON_DEMAND`.
     */
    pricingPlan?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A detail pricing plan of the  reserved queue. See below.
     */
    reservationPlanSettings?: pulumi.Input<inputs.mediaconvert.QueueReservationPlanSettings>;
    /**
     * A status of the queue. Valid values are `ACTIVE` or `RESERVED`. Default to `PAUSED`.
     */
    status?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Queue resource.
 */
export interface QueueArgs {
    /**
     * The maximum number of jobs your queue can process concurrently. For on-demand queues, the value you enter is constrained by your service quotas for Maximum concurrent jobs, per on-demand queue and Maximum concurrent jobs, per account. For reserved queues, specify the number of jobs you can process concurrently in your reservation plan instead.
     */
    concurrentJobs?: pulumi.Input<number>;
    /**
     * A description of the queue
     */
    description?: pulumi.Input<string>;
    /**
     * A unique identifier describing the queue
     */
    name?: pulumi.Input<string>;
    /**
     * Specifies whether the pricing plan for the queue is on-demand or reserved. Valid values are `ON_DEMAND` or `RESERVED`. Default to `ON_DEMAND`.
     */
    pricingPlan?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A detail pricing plan of the  reserved queue. See below.
     */
    reservationPlanSettings?: pulumi.Input<inputs.mediaconvert.QueueReservationPlanSettings>;
    /**
     * A status of the queue. Valid values are `ACTIVE` or `RESERVED`. Default to `PAUSED`.
     */
    status?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
