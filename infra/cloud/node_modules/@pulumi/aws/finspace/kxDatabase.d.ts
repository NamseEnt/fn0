import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS FinSpace Kx Database.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.Key("example", {
 *     description: "Example KMS Key",
 *     deletionWindowInDays: 7,
 * });
 * const exampleKxEnvironment = new aws.finspace.KxEnvironment("example", {
 *     name: "my-tf-kx-environment",
 *     kmsKeyId: example.arn,
 * });
 * const exampleKxDatabase = new aws.finspace.KxDatabase("example", {
 *     environmentId: exampleKxEnvironment.id,
 *     name: "my-tf-kx-database",
 *     description: "Example database description",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import an AWS FinSpace Kx Database using the `id` (environment ID and database name, comma-delimited). For example:
 *
 * ```sh
 * $ pulumi import aws:finspace/kxDatabase:KxDatabase example n3ceo7wqxoxcti5tujqwzs,my-tf-kx-database
 * ```
 */
export declare class KxDatabase extends pulumi.CustomResource {
    /**
     * Get an existing KxDatabase resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: KxDatabaseState, opts?: pulumi.CustomResourceOptions): KxDatabase;
    /**
     * Returns true if the given object is an instance of KxDatabase.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is KxDatabase;
    /**
     * Amazon Resource Name (ARN) identifier of the KX database.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Timestamp at which the databse is created in FinSpace. Value determined as epoch time in seconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000.
     */
    readonly createdTimestamp: pulumi.Output<string>;
    /**
     * Description of the KX database.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Unique identifier for the KX environment.
     */
    readonly environmentId: pulumi.Output<string>;
    /**
     * Last timestamp at which the database was updated in FinSpace. Value determined as epoch time in seconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000.
     */
    readonly lastModifiedTimestamp: pulumi.Output<string>;
    /**
     * Name of the KX database.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * Create a KxDatabase resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: KxDatabaseArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering KxDatabase resources.
 */
export interface KxDatabaseState {
    /**
     * Amazon Resource Name (ARN) identifier of the KX database.
     */
    arn?: pulumi.Input<string>;
    /**
     * Timestamp at which the databse is created in FinSpace. Value determined as epoch time in seconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000.
     */
    createdTimestamp?: pulumi.Input<string>;
    /**
     * Description of the KX database.
     */
    description?: pulumi.Input<string>;
    /**
     * Unique identifier for the KX environment.
     */
    environmentId?: pulumi.Input<string>;
    /**
     * Last timestamp at which the database was updated in FinSpace. Value determined as epoch time in seconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000.
     */
    lastModifiedTimestamp?: pulumi.Input<string>;
    /**
     * Name of the KX database.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
}
/**
 * The set of arguments for constructing a KxDatabase resource.
 */
export interface KxDatabaseArgs {
    /**
     * Description of the KX database.
     */
    description?: pulumi.Input<string>;
    /**
     * Unique identifier for the KX environment.
     */
    environmentId: pulumi.Input<string>;
    /**
     * Name of the KX database.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
