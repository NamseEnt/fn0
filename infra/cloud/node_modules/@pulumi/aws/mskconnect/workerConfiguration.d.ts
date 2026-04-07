import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Amazon MSK Connect Worker Configuration Resource.
 *
 * ## Example Usage
 *
 * ### Basic configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.mskconnect.WorkerConfiguration("example", {
 *     name: "example",
 *     propertiesFileContent: `key.converter=org.apache.kafka.connect.storage.StringConverter
 * value.converter=org.apache.kafka.connect.storage.StringConverter
 * `,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import MSK Connect Worker Configuration using the plugin's `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:mskconnect/workerConfiguration:WorkerConfiguration example 'arn:aws:kafkaconnect:eu-central-1:123456789012:worker-configuration/example/8848493b-7fcc-478c-a646-4a52634e3378-4'
 * ```
 */
export declare class WorkerConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing WorkerConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WorkerConfigurationState, opts?: pulumi.CustomResourceOptions): WorkerConfiguration;
    /**
     * Returns true if the given object is an instance of WorkerConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is WorkerConfiguration;
    /**
     * the Amazon Resource Name (ARN) of the worker configuration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A summary description of the worker configuration.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * an ID of the latest successfully created revision of the worker configuration.
     */
    readonly latestRevision: pulumi.Output<number>;
    /**
     * The name of the worker configuration.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Contents of connect-distributed.properties file. The value can be either base64 encoded or in raw format.
     *
     * The following arguments are optional:
     */
    readonly propertiesFileContent: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a WorkerConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: WorkerConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering WorkerConfiguration resources.
 */
export interface WorkerConfigurationState {
    /**
     * the Amazon Resource Name (ARN) of the worker configuration.
     */
    arn?: pulumi.Input<string>;
    /**
     * A summary description of the worker configuration.
     */
    description?: pulumi.Input<string>;
    /**
     * an ID of the latest successfully created revision of the worker configuration.
     */
    latestRevision?: pulumi.Input<number>;
    /**
     * The name of the worker configuration.
     */
    name?: pulumi.Input<string>;
    /**
     * Contents of connect-distributed.properties file. The value can be either base64 encoded or in raw format.
     *
     * The following arguments are optional:
     */
    propertiesFileContent?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a WorkerConfiguration resource.
 */
export interface WorkerConfigurationArgs {
    /**
     * A summary description of the worker configuration.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the worker configuration.
     */
    name?: pulumi.Input<string>;
    /**
     * Contents of connect-distributed.properties file. The value can be either base64 encoded or in raw format.
     *
     * The following arguments are optional:
     */
    propertiesFileContent: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
