import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS CodeCatalyst Project.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.codecatalyst.Project("test", {
 *     spaceName: "myproject",
 *     displayName: "MyProject",
 *     description: "My CodeCatalyst Project created using Pulumi",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CodeCatalyst Project using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:codecatalyst/project:Project example project-id-12345678
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
     * The description of the project. This description will be displayed to all users of the project. We recommend providing a brief description of the project and its intended purpose.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The friendly name of the project that will be displayed to users.
     *
     * The following arguments are optional:
     */
    readonly displayName: pulumi.Output<string>;
    /**
     * The name of the project in the space.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name of the space.
     */
    readonly spaceName: pulumi.Output<string>;
    /**
     * Create a Project resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProjectArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Project resources.
 */
export interface ProjectState {
    /**
     * The description of the project. This description will be displayed to all users of the project. We recommend providing a brief description of the project and its intended purpose.
     */
    description?: pulumi.Input<string>;
    /**
     * The friendly name of the project that will be displayed to users.
     *
     * The following arguments are optional:
     */
    displayName?: pulumi.Input<string>;
    /**
     * The name of the project in the space.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the space.
     */
    spaceName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Project resource.
 */
export interface ProjectArgs {
    /**
     * The description of the project. This description will be displayed to all users of the project. We recommend providing a brief description of the project and its intended purpose.
     */
    description?: pulumi.Input<string>;
    /**
     * The friendly name of the project that will be displayed to users.
     *
     * The following arguments are optional:
     */
    displayName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the space.
     */
    spaceName: pulumi.Input<string>;
}
