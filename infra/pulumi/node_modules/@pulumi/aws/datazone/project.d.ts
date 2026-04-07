import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS DataZone Project.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.datazone.Project("test", {
 *     domainId: testAwsDatazoneDomain.id,
 *     glossaryTerms: ["2N8w6XJCwZf"],
 *     name: "name",
 *     description: "desc",
 *     skipDeletionCheck: true,
 * });
 * ```
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.datazone.Project("test", {
 *     domainIdentifier: testAwsDatazoneDomain.id,
 *     name: "name",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DataZone Project using a colon-delimited string combining `domain_id` and `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:datazone/project:Project example domain-1234:project-1234
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
     * Timestamp of when the project was made.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Creator of the project.
     */
    readonly createdBy: pulumi.Output<string>;
    /**
     * Description of project.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Identifier of domain which the project is part of. Must follow the regex of `^dzd[-_][a-zA-Z0-9_-]{1,36}$`.
     */
    readonly domainIdentifier: pulumi.Output<string>;
    /**
     * List of error messages if operation cannot be completed.
     */
    readonly failureReasons: pulumi.Output<outputs.datazone.ProjectFailureReason[]>;
    /**
     * List of glossary terms that can be used in the project. The list cannot be empty or include over 20 values. Each value must follow the regex of `[a-zA-Z0-9_-]{1,36}$`.
     */
    readonly glossaryTerms: pulumi.Output<string[] | undefined>;
    /**
     * Timestamp of when the project was last updated.
     */
    readonly lastUpdatedAt: pulumi.Output<string>;
    /**
     * Name of the project. Must follow the regex of `^[\w -]+$`. and have a length of at most 64.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Enum that conveys state of project. Can be `ACTIVE`, `DELETING`, or `DELETE_FAILED`.
     */
    readonly projectStatus: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Optional flag to delete all child entities within the project.
     */
    readonly skipDeletionCheck: pulumi.Output<boolean | undefined>;
    readonly timeouts: pulumi.Output<outputs.datazone.ProjectTimeouts | undefined>;
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
     * Timestamp of when the project was made.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Creator of the project.
     */
    createdBy?: pulumi.Input<string>;
    /**
     * Description of project.
     */
    description?: pulumi.Input<string>;
    /**
     * Identifier of domain which the project is part of. Must follow the regex of `^dzd[-_][a-zA-Z0-9_-]{1,36}$`.
     */
    domainIdentifier?: pulumi.Input<string>;
    /**
     * List of error messages if operation cannot be completed.
     */
    failureReasons?: pulumi.Input<pulumi.Input<inputs.datazone.ProjectFailureReason>[]>;
    /**
     * List of glossary terms that can be used in the project. The list cannot be empty or include over 20 values. Each value must follow the regex of `[a-zA-Z0-9_-]{1,36}$`.
     */
    glossaryTerms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Timestamp of when the project was last updated.
     */
    lastUpdatedAt?: pulumi.Input<string>;
    /**
     * Name of the project. Must follow the regex of `^[\w -]+$`. and have a length of at most 64.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Enum that conveys state of project. Can be `ACTIVE`, `DELETING`, or `DELETE_FAILED`.
     */
    projectStatus?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Optional flag to delete all child entities within the project.
     */
    skipDeletionCheck?: pulumi.Input<boolean>;
    timeouts?: pulumi.Input<inputs.datazone.ProjectTimeouts>;
}
/**
 * The set of arguments for constructing a Project resource.
 */
export interface ProjectArgs {
    /**
     * Description of project.
     */
    description?: pulumi.Input<string>;
    /**
     * Identifier of domain which the project is part of. Must follow the regex of `^dzd[-_][a-zA-Z0-9_-]{1,36}$`.
     */
    domainIdentifier: pulumi.Input<string>;
    /**
     * List of glossary terms that can be used in the project. The list cannot be empty or include over 20 values. Each value must follow the regex of `[a-zA-Z0-9_-]{1,36}$`.
     */
    glossaryTerms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Name of the project. Must follow the regex of `^[\w -]+$`. and have a length of at most 64.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Optional flag to delete all child entities within the project.
     */
    skipDeletionCheck?: pulumi.Input<boolean>;
    timeouts?: pulumi.Input<inputs.datazone.ProjectTimeouts>;
}
