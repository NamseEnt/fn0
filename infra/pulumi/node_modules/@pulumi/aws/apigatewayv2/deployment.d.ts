import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an Amazon API Gateway Version 2 deployment.
 * More information can be found in the [Amazon API Gateway Developer Guide](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api.html).
 *
 * > **Note:** Creating a deployment for an API requires at least one `aws.apigatewayv2.Route` resource associated with that API. To avoid race conditions when all resources are being created together, you need to add implicit resource references via the `triggers` argument or explicit resource references using the [resource `dependsOn` meta-argument](https://www.pulumi.com/docs/intro/concepts/programming-model/#dependson).
 *
 * ## Example Usage
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_apigatewayv2_deployment` using the API identifier and deployment identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:apigatewayv2/deployment:Deployment example aabbccddee/1122334
 * ```
 * The `triggers` argument cannot be imported.
 */
export declare class Deployment extends pulumi.CustomResource {
    /**
     * Get an existing Deployment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DeploymentState, opts?: pulumi.CustomResourceOptions): Deployment;
    /**
     * Returns true if the given object is an instance of Deployment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Deployment;
    /**
     * API identifier.
     */
    readonly apiId: pulumi.Output<string>;
    /**
     * Whether the deployment was automatically released.
     */
    readonly autoDeployed: pulumi.Output<boolean>;
    /**
     * Description for the deployment resource. Must be less than or equal to 1024 characters in length.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of arbitrary keys and values that, when changed, will trigger a redeployment.
     */
    readonly triggers: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Create a Deployment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DeploymentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Deployment resources.
 */
export interface DeploymentState {
    /**
     * API identifier.
     */
    apiId?: pulumi.Input<string>;
    /**
     * Whether the deployment was automatically released.
     */
    autoDeployed?: pulumi.Input<boolean>;
    /**
     * Description for the deployment resource. Must be less than or equal to 1024 characters in length.
     */
    description?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of arbitrary keys and values that, when changed, will trigger a redeployment.
     */
    triggers?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Deployment resource.
 */
export interface DeploymentArgs {
    /**
     * API identifier.
     */
    apiId: pulumi.Input<string>;
    /**
     * Description for the deployment resource. Must be less than or equal to 1024 characters in length.
     */
    description?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of arbitrary keys and values that, when changed, will trigger a redeployment.
     */
    triggers?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
