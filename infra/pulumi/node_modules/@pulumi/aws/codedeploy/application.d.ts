import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CodeDeploy application to be used as a basis for deployments
 *
 * ## Example Usage
 *
 * ### ECS Application
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.codedeploy.Application("example", {
 *     computePlatform: "ECS",
 *     name: "example",
 * });
 * ```
 *
 * ### Lambda Application
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.codedeploy.Application("example", {
 *     computePlatform: "Lambda",
 *     name: "example",
 * });
 * ```
 *
 * ### Server Application
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.codedeploy.Application("example", {
 *     computePlatform: "Server",
 *     name: "example",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CodeDeploy Applications using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:codedeploy/application:Application example my-application
 * ```
 */
export declare class Application extends pulumi.CustomResource {
    /**
     * Get an existing Application resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ApplicationState, opts?: pulumi.CustomResourceOptions): Application;
    /**
     * Returns true if the given object is an instance of Application.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Application;
    /**
     * The application ID.
     */
    readonly applicationId: pulumi.Output<string>;
    /**
     * The ARN of the CodeDeploy application.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The compute platform can either be `ECS`, `Lambda`, or `Server`. Default is `Server`.
     */
    readonly computePlatform: pulumi.Output<string | undefined>;
    /**
     * The name for a connection to a GitHub account.
     */
    readonly githubAccountName: pulumi.Output<string>;
    /**
     * Whether the user has authenticated with GitHub for the specified application.
     */
    readonly linkedToGithub: pulumi.Output<boolean>;
    /**
     * The name of the application.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
    /**
     * Create a Application resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ApplicationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Application resources.
 */
export interface ApplicationState {
    /**
     * The application ID.
     */
    applicationId?: pulumi.Input<string>;
    /**
     * The ARN of the CodeDeploy application.
     */
    arn?: pulumi.Input<string>;
    /**
     * The compute platform can either be `ECS`, `Lambda`, or `Server`. Default is `Server`.
     */
    computePlatform?: pulumi.Input<string>;
    /**
     * The name for a connection to a GitHub account.
     */
    githubAccountName?: pulumi.Input<string>;
    /**
     * Whether the user has authenticated with GitHub for the specified application.
     */
    linkedToGithub?: pulumi.Input<boolean>;
    /**
     * The name of the application.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
}
/**
 * The set of arguments for constructing a Application resource.
 */
export interface ApplicationArgs {
    /**
     * The compute platform can either be `ECS`, `Lambda`, or `Server`. Default is `Server`.
     */
    computePlatform?: pulumi.Input<string>;
    /**
     * The name of the application.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
