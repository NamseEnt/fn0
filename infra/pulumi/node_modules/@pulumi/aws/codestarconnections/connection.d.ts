import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CodeStar Connection.
 *
 * > **NOTE:** The `aws.codestarconnections.Connection` resource is created in the state `PENDING`. Authentication with the connection provider must be completed in the AWS Console. See the [AWS documentation](https://docs.aws.amazon.com/dtconsole/latest/userguide/connections-update.html) for details.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.codestarconnections.Connection("example", {
 *     name: "example-connection",
 *     providerType: "Bitbucket",
 * });
 * const examplePipeline = new aws.codepipeline.Pipeline("example", {
 *     artifactStores: [{}],
 *     stages: [
 *         {
 *             name: "Source",
 *             actions: [{
 *                 name: "Source",
 *                 category: "Source",
 *                 owner: "AWS",
 *                 provider: "CodeStarSourceConnection",
 *                 version: "1",
 *                 outputArtifacts: ["source_output"],
 *                 configuration: {
 *                     ConnectionArn: example.arn,
 *                     FullRepositoryId: "my-organization/test",
 *                     BranchName: "main",
 *                 },
 *             }],
 *         },
 *         {
 *             actions: [{}],
 *             name: "Build",
 *         },
 *         {
 *             actions: [{}],
 *             name: "Deploy",
 *         },
 *     ],
 *     name: "tf-test-pipeline",
 *     roleArn: codepipelineRole.arn,
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the CodeStar connection.
 *
 * Using `pulumi import`, import CodeStar connections using the ARN. For example:
 *
 * console
 *
 * % pulumi import aws_codestarconnections_connection.test-connection arn:aws:codestar-connections:us-west-1:0123456789:connection/79d4d357-a2ee-41e4-b350-2fe39ae59448
 */
export declare class Connection extends pulumi.CustomResource {
    /**
     * Get an existing Connection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConnectionState, opts?: pulumi.CustomResourceOptions): Connection;
    /**
     * Returns true if the given object is an instance of Connection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Connection;
    /**
     * The codestar connection ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The codestar connection status. Possible values are `PENDING`, `AVAILABLE` and `ERROR`.
     */
    readonly connectionStatus: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the host associated with the connection. Conflicts with `providerType`
     */
    readonly hostArn: pulumi.Output<string | undefined>;
    /**
     * The name of the connection to be created. The name must be unique in the calling AWS account. Changing `name` will create a new resource.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The name of the external provider where your third-party code repository is configured. Valid values are `Bitbucket`, `GitHub`, `GitHubEnterpriseServer`, `GitLab` or `GitLabSelfManaged`. Changing `providerType` will create a new resource. Conflicts with `hostArn`
     */
    readonly providerType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of key-value resource tags to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Connection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ConnectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Connection resources.
 */
export interface ConnectionState {
    /**
     * The codestar connection ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * The codestar connection status. Possible values are `PENDING`, `AVAILABLE` and `ERROR`.
     */
    connectionStatus?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the host associated with the connection. Conflicts with `providerType`
     */
    hostArn?: pulumi.Input<string>;
    /**
     * The name of the connection to be created. The name must be unique in the calling AWS account. Changing `name` will create a new resource.
     */
    name?: pulumi.Input<string>;
    /**
     * The name of the external provider where your third-party code repository is configured. Valid values are `Bitbucket`, `GitHub`, `GitHubEnterpriseServer`, `GitLab` or `GitLabSelfManaged`. Changing `providerType` will create a new resource. Conflicts with `hostArn`
     */
    providerType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of key-value resource tags to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Connection resource.
 */
export interface ConnectionArgs {
    /**
     * The Amazon Resource Name (ARN) of the host associated with the connection. Conflicts with `providerType`
     */
    hostArn?: pulumi.Input<string>;
    /**
     * The name of the connection to be created. The name must be unique in the calling AWS account. Changing `name` will create a new resource.
     */
    name?: pulumi.Input<string>;
    /**
     * The name of the external provider where your third-party code repository is configured. Valid values are `Bitbucket`, `GitHub`, `GitHubEnterpriseServer`, `GitLab` or `GitLabSelfManaged`. Changing `providerType` will create a new resource. Conflicts with `hostArn`
     */
    providerType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of key-value resource tags to associate with the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
