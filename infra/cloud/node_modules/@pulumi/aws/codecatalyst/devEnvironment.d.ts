import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS CodeCatalyst Dev Environment.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.codecatalyst.DevEnvironment("test", {
 *     alias: "devenv",
 *     spaceName: "myspace",
 *     projectName: "myproject",
 *     instanceType: "dev.standard1.small",
 *     persistentStorage: {
 *         size: 16,
 *     },
 *     ides: {
 *         name: "PyCharm",
 *         runtime: "public.ecr.aws/jetbrains/py",
 *     },
 *     inactivityTimeoutMinutes: 40,
 *     repositories: [{
 *         repositoryName: "pulumi-provider-aws",
 *         branchName: "main",
 *     }],
 * });
 * ```
 */
export declare class DevEnvironment extends pulumi.CustomResource {
    /**
     * Get an existing DevEnvironment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DevEnvironmentState, opts?: pulumi.CustomResourceOptions): DevEnvironment;
    /**
     * Returns true if the given object is an instance of DevEnvironment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DevEnvironment;
    readonly alias: pulumi.Output<string | undefined>;
    /**
     * Information about the integrated development environment (IDE) configured for a Dev Environment.
     */
    readonly ides: pulumi.Output<outputs.codecatalyst.DevEnvironmentIdes>;
    /**
     * The amount of time the Dev Environment will run without any activity detected before stopping, in minutes. Only whole integers are allowed. Dev Environments consume compute minutes when running.
     */
    readonly inactivityTimeoutMinutes: pulumi.Output<number | undefined>;
    /**
     * The Amazon EC2 instace type to use for the Dev Environment. Valid values include dev.standard1.small,dev.standard1.medium,dev.standard1.large,dev.standard1.xlarge
     *
     * The following arguments are optional:
     */
    readonly instanceType: pulumi.Output<string>;
    /**
     * Information about the amount of storage allocated to the Dev Environment.
     */
    readonly persistentStorage: pulumi.Output<outputs.codecatalyst.DevEnvironmentPersistentStorage>;
    /**
     * The name of the project in the space.
     */
    readonly projectName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The source repository that contains the branch to clone into the Dev Environment.
     */
    readonly repositories: pulumi.Output<outputs.codecatalyst.DevEnvironmentRepository[] | undefined>;
    /**
     * The name of the space.
     */
    readonly spaceName: pulumi.Output<string>;
    /**
     * Create a DevEnvironment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DevEnvironmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DevEnvironment resources.
 */
export interface DevEnvironmentState {
    alias?: pulumi.Input<string>;
    /**
     * Information about the integrated development environment (IDE) configured for a Dev Environment.
     */
    ides?: pulumi.Input<inputs.codecatalyst.DevEnvironmentIdes>;
    /**
     * The amount of time the Dev Environment will run without any activity detected before stopping, in minutes. Only whole integers are allowed. Dev Environments consume compute minutes when running.
     */
    inactivityTimeoutMinutes?: pulumi.Input<number>;
    /**
     * The Amazon EC2 instace type to use for the Dev Environment. Valid values include dev.standard1.small,dev.standard1.medium,dev.standard1.large,dev.standard1.xlarge
     *
     * The following arguments are optional:
     */
    instanceType?: pulumi.Input<string>;
    /**
     * Information about the amount of storage allocated to the Dev Environment.
     */
    persistentStorage?: pulumi.Input<inputs.codecatalyst.DevEnvironmentPersistentStorage>;
    /**
     * The name of the project in the space.
     */
    projectName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The source repository that contains the branch to clone into the Dev Environment.
     */
    repositories?: pulumi.Input<pulumi.Input<inputs.codecatalyst.DevEnvironmentRepository>[]>;
    /**
     * The name of the space.
     */
    spaceName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DevEnvironment resource.
 */
export interface DevEnvironmentArgs {
    alias?: pulumi.Input<string>;
    /**
     * Information about the integrated development environment (IDE) configured for a Dev Environment.
     */
    ides: pulumi.Input<inputs.codecatalyst.DevEnvironmentIdes>;
    /**
     * The amount of time the Dev Environment will run without any activity detected before stopping, in minutes. Only whole integers are allowed. Dev Environments consume compute minutes when running.
     */
    inactivityTimeoutMinutes?: pulumi.Input<number>;
    /**
     * The Amazon EC2 instace type to use for the Dev Environment. Valid values include dev.standard1.small,dev.standard1.medium,dev.standard1.large,dev.standard1.xlarge
     *
     * The following arguments are optional:
     */
    instanceType: pulumi.Input<string>;
    /**
     * Information about the amount of storage allocated to the Dev Environment.
     */
    persistentStorage: pulumi.Input<inputs.codecatalyst.DevEnvironmentPersistentStorage>;
    /**
     * The name of the project in the space.
     */
    projectName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The source repository that contains the branch to clone into the Dev Environment.
     */
    repositories?: pulumi.Input<pulumi.Input<inputs.codecatalyst.DevEnvironmentRepository>[]>;
    /**
     * The name of the space.
     */
    spaceName: pulumi.Input<string>;
}
