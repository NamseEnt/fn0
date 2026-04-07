import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Amplify Backend Environment resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.amplify.App("example", {name: "example"});
 * const exampleBackendEnvironment = new aws.amplify.BackendEnvironment("example", {
 *     appId: example.id,
 *     environmentName: "example",
 *     deploymentArtifacts: "app-example-deployment",
 *     stackName: "amplify-app-example",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Amplify backend environment using `app_id` and `environment_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:amplify/backendEnvironment:BackendEnvironment example d2ypk4k47z8u6/example
 * ```
 */
export declare class BackendEnvironment extends pulumi.CustomResource {
    /**
     * Get an existing BackendEnvironment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BackendEnvironmentState, opts?: pulumi.CustomResourceOptions): BackendEnvironment;
    /**
     * Returns true if the given object is an instance of BackendEnvironment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BackendEnvironment;
    /**
     * Unique ID for an Amplify app.
     */
    readonly appId: pulumi.Output<string>;
    /**
     * ARN for a backend environment that is part of an Amplify app.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Name of deployment artifacts.
     */
    readonly deploymentArtifacts: pulumi.Output<string>;
    /**
     * Name for the backend environment.
     */
    readonly environmentName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * AWS CloudFormation stack name of a backend environment.
     */
    readonly stackName: pulumi.Output<string>;
    /**
     * Create a BackendEnvironment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BackendEnvironmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BackendEnvironment resources.
 */
export interface BackendEnvironmentState {
    /**
     * Unique ID for an Amplify app.
     */
    appId?: pulumi.Input<string>;
    /**
     * ARN for a backend environment that is part of an Amplify app.
     */
    arn?: pulumi.Input<string>;
    /**
     * Name of deployment artifacts.
     */
    deploymentArtifacts?: pulumi.Input<string>;
    /**
     * Name for the backend environment.
     */
    environmentName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * AWS CloudFormation stack name of a backend environment.
     */
    stackName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a BackendEnvironment resource.
 */
export interface BackendEnvironmentArgs {
    /**
     * Unique ID for an Amplify app.
     */
    appId: pulumi.Input<string>;
    /**
     * Name of deployment artifacts.
     */
    deploymentArtifacts?: pulumi.Input<string>;
    /**
     * Name for the backend environment.
     */
    environmentName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * AWS CloudFormation stack name of a backend environment.
     */
    stackName?: pulumi.Input<string>;
}
