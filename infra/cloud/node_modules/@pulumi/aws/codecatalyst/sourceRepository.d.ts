import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS CodeCatalyst Source Repository.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.codecatalyst.SourceRepository("example", {
 *     name: "example-repo",
 *     projectName: "example-project",
 *     spaceName: "example-space",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CodeCatalyst Source Repository using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:codecatalyst/sourceRepository:SourceRepository example example-repo
 * ```
 */
export declare class SourceRepository extends pulumi.CustomResource {
    /**
     * Get an existing SourceRepository resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SourceRepositoryState, opts?: pulumi.CustomResourceOptions): SourceRepository;
    /**
     * Returns true if the given object is an instance of SourceRepository.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SourceRepository;
    /**
     * The description of the project. This description will be displayed to all users of the project. We recommend providing a brief description of the project and its intended purpose.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The name of the source repository. For more information about name requirements, see [Quotas for source repositories](https://docs.aws.amazon.com/codecatalyst/latest/userguide/source-quotas.html).
     */
    readonly name: pulumi.Output<string>;
    /**
     * The name of the project in the CodeCatalyst space.
     *
     * The following arguments are optional:
     */
    readonly projectName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name of the CodeCatalyst space.
     */
    readonly spaceName: pulumi.Output<string>;
    /**
     * Create a SourceRepository resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SourceRepositoryArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SourceRepository resources.
 */
export interface SourceRepositoryState {
    /**
     * The description of the project. This description will be displayed to all users of the project. We recommend providing a brief description of the project and its intended purpose.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the source repository. For more information about name requirements, see [Quotas for source repositories](https://docs.aws.amazon.com/codecatalyst/latest/userguide/source-quotas.html).
     */
    name?: pulumi.Input<string>;
    /**
     * The name of the project in the CodeCatalyst space.
     *
     * The following arguments are optional:
     */
    projectName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the CodeCatalyst space.
     */
    spaceName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SourceRepository resource.
 */
export interface SourceRepositoryArgs {
    /**
     * The description of the project. This description will be displayed to all users of the project. We recommend providing a brief description of the project and its intended purpose.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the source repository. For more information about name requirements, see [Quotas for source repositories](https://docs.aws.amazon.com/codecatalyst/latest/userguide/source-quotas.html).
     */
    name?: pulumi.Input<string>;
    /**
     * The name of the project in the CodeCatalyst space.
     *
     * The following arguments are optional:
     */
    projectName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the CodeCatalyst space.
     */
    spaceName: pulumi.Input<string>;
}
