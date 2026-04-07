import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a CloudWatch Evidently Project resource.
 *
 * > **Warning:** This resource is deprecated. Use [AWS AppConfig feature flags](https://aws.amazon.com/blogs/mt/using-aws-appconfig-feature-flags/) instead.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.evidently.Project("example", {
 *     name: "Example",
 *     description: "Example Description",
 *     tags: {
 *         Key1: "example Project",
 *     },
 * });
 * ```
 *
 * ### Store evaluation events in a CloudWatch Log Group
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.evidently.Project("example", {
 *     name: "Example",
 *     description: "Example Description",
 *     dataDelivery: {
 *         cloudwatchLogs: {
 *             logGroup: "example-log-group-name",
 *         },
 *     },
 *     tags: {
 *         Key1: "example Project",
 *     },
 * });
 * ```
 *
 * ### Store evaluation events in an S3 bucket
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.evidently.Project("example", {
 *     name: "Example",
 *     description: "Example Description",
 *     dataDelivery: {
 *         s3Destination: {
 *             bucket: "example-bucket-name",
 *             prefix: "example",
 *         },
 *     },
 *     tags: {
 *         Key1: "example Project",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudWatch Evidently Project using the `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:evidently/project:Project example arn:aws:evidently:us-east-1:123456789012:segment/example
 * ```
 */
export declare class Project extends pulumi.CustomResource {
    /**
     * Get an existing Project resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProjectState, opts?: pulumi.CustomResourceOptions): Project;
    /**
     * Returns true if the given object is an instance of Project.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Project;
    /**
     * The number of ongoing experiments currently in the project.
     */
    readonly activeExperimentCount: pulumi.Output<number>;
    /**
     * The number of ongoing launches currently in the project.
     */
    readonly activeLaunchCount: pulumi.Output<number>;
    /**
     * The ARN of the project.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The date and time that the project is created.
     */
    readonly createdTime: pulumi.Output<string>;
    /**
     * A block that contains information about where Evidently is to store evaluation events for longer term storage, if you choose to do so. If you choose not to store these events, Evidently deletes them after using them to produce metrics and other experiment results that you can view. See below.
     */
    readonly dataDelivery: pulumi.Output<outputs.evidently.ProjectDataDelivery | undefined>;
    /**
     * Specifies the description of the project.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The number of experiments currently in the project. This includes all experiments that have been created and not deleted, whether they are ongoing or not.
     */
    readonly experimentCount: pulumi.Output<number>;
    /**
     * The number of features currently in the project.
     */
    readonly featureCount: pulumi.Output<number>;
    /**
     * The date and time that the project was most recently updated.
     */
    readonly lastUpdatedTime: pulumi.Output<string>;
    /**
     * The number of launches currently in the project. This includes all launches that have been created and not deleted, whether they are ongoing or not.
     */
    readonly launchCount: pulumi.Output<number>;
    /**
     * A name for the project.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The current state of the project. Valid values are `AVAILABLE` and `UPDATING`.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Tags to apply to the project. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Project resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ProjectArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Project resources.
 */
export interface ProjectState {
    /**
     * The number of ongoing experiments currently in the project.
     */
    activeExperimentCount?: pulumi.Input<number>;
    /**
     * The number of ongoing launches currently in the project.
     */
    activeLaunchCount?: pulumi.Input<number>;
    /**
     * The ARN of the project.
     */
    arn?: pulumi.Input<string>;
    /**
     * The date and time that the project is created.
     */
    createdTime?: pulumi.Input<string>;
    /**
     * A block that contains information about where Evidently is to store evaluation events for longer term storage, if you choose to do so. If you choose not to store these events, Evidently deletes them after using them to produce metrics and other experiment results that you can view. See below.
     */
    dataDelivery?: pulumi.Input<inputs.evidently.ProjectDataDelivery>;
    /**
     * Specifies the description of the project.
     */
    description?: pulumi.Input<string>;
    /**
     * The number of experiments currently in the project. This includes all experiments that have been created and not deleted, whether they are ongoing or not.
     */
    experimentCount?: pulumi.Input<number>;
    /**
     * The number of features currently in the project.
     */
    featureCount?: pulumi.Input<number>;
    /**
     * The date and time that the project was most recently updated.
     */
    lastUpdatedTime?: pulumi.Input<string>;
    /**
     * The number of launches currently in the project. This includes all launches that have been created and not deleted, whether they are ongoing or not.
     */
    launchCount?: pulumi.Input<number>;
    /**
     * A name for the project.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The current state of the project. Valid values are `AVAILABLE` and `UPDATING`.
     */
    status?: pulumi.Input<string>;
    /**
     * Tags to apply to the project. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Project resource.
 */
export interface ProjectArgs {
    /**
     * A block that contains information about where Evidently is to store evaluation events for longer term storage, if you choose to do so. If you choose not to store these events, Evidently deletes them after using them to produce metrics and other experiment results that you can view. See below.
     */
    dataDelivery?: pulumi.Input<inputs.evidently.ProjectDataDelivery>;
    /**
     * Specifies the description of the project.
     */
    description?: pulumi.Input<string>;
    /**
     * A name for the project.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the project. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
