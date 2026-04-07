import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an Amazon API Gateway Version 2 stage.
 * More information can be found in the [Amazon API Gateway Developer Guide](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api.html).
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apigatewayv2.Stage("example", {
 *     apiId: exampleAwsApigatewayv2Api.id,
 *     name: "example-stage",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_apigatewayv2_stage` using the API identifier and stage name. For example:
 *
 * ```sh
 * $ pulumi import aws:apigatewayv2/stage:Stage example aabbccddee/example-stage
 * ```
 * -> __Note:__ The API Gateway managed stage created as part of [_quick_create_](https://docs.aws.amazon.com/apigateway/latest/developerguide/api-gateway-basic-concept.html#apigateway-definition-quick-create) cannot be imported.
 */
export declare class Stage extends pulumi.CustomResource {
    /**
     * Get an existing Stage resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: StageState, opts?: pulumi.CustomResourceOptions): Stage;
    /**
     * Returns true if the given object is an instance of Stage.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Stage;
    /**
     * Settings for logging access in this stage.
     * Use the `aws.apigateway.Account` resource to configure [permissions for CloudWatch Logging](https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-logging.html#set-up-access-logging-permissions).
     */
    readonly accessLogSettings: pulumi.Output<outputs.apigatewayv2.StageAccessLogSettings | undefined>;
    /**
     * API identifier.
     */
    readonly apiId: pulumi.Output<string>;
    /**
     * ARN of the stage.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Whether updates to an API automatically trigger a new deployment. Defaults to `false`. Applicable for HTTP APIs.
     */
    readonly autoDeploy: pulumi.Output<boolean | undefined>;
    /**
     * Identifier of a client certificate for the stage. Use the `aws.apigateway.ClientCertificate` resource to configure a client certificate.
     * Supported only for WebSocket APIs.
     */
    readonly clientCertificateId: pulumi.Output<string | undefined>;
    /**
     * Default route settings for the stage.
     */
    readonly defaultRouteSettings: pulumi.Output<outputs.apigatewayv2.StageDefaultRouteSettings | undefined>;
    /**
     * Deployment identifier of the stage. Use the `aws.apigatewayv2.Deployment` resource to configure a deployment.
     */
    readonly deploymentId: pulumi.Output<string>;
    /**
     * Description for the stage. Must be less than or equal to 1024 characters in length.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * ARN prefix to be used in an `aws.lambda.Permission`'s `sourceArn` attribute.
     * For WebSocket APIs this attribute can additionally be used in an `aws.iam.Policy` to authorize access to the [`@connections` API](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-how-to-call-websocket-api-connections.html).
     * See the [Amazon API Gateway Developer Guide](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-control-access-iam.html) for details.
     */
    readonly executionArn: pulumi.Output<string>;
    /**
     * URL to invoke the API pointing to the stage,
     * e.g., `wss://z4675bid1j.execute-api.eu-west-2.amazonaws.com/example-stage`, or `https://z4675bid1j.execute-api.eu-west-2.amazonaws.com/`
     */
    readonly invokeUrl: pulumi.Output<string>;
    /**
     * Name of the stage. Must be between 1 and 128 characters in length.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Route settings for the stage.
     */
    readonly routeSettings: pulumi.Output<outputs.apigatewayv2.StageRouteSetting[] | undefined>;
    /**
     * Map that defines the stage variables for the stage.
     */
    readonly stageVariables: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags to assign to the stage. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Stage resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: StageArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Stage resources.
 */
export interface StageState {
    /**
     * Settings for logging access in this stage.
     * Use the `aws.apigateway.Account` resource to configure [permissions for CloudWatch Logging](https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-logging.html#set-up-access-logging-permissions).
     */
    accessLogSettings?: pulumi.Input<inputs.apigatewayv2.StageAccessLogSettings>;
    /**
     * API identifier.
     */
    apiId?: pulumi.Input<string>;
    /**
     * ARN of the stage.
     */
    arn?: pulumi.Input<string>;
    /**
     * Whether updates to an API automatically trigger a new deployment. Defaults to `false`. Applicable for HTTP APIs.
     */
    autoDeploy?: pulumi.Input<boolean>;
    /**
     * Identifier of a client certificate for the stage. Use the `aws.apigateway.ClientCertificate` resource to configure a client certificate.
     * Supported only for WebSocket APIs.
     */
    clientCertificateId?: pulumi.Input<string>;
    /**
     * Default route settings for the stage.
     */
    defaultRouteSettings?: pulumi.Input<inputs.apigatewayv2.StageDefaultRouteSettings>;
    /**
     * Deployment identifier of the stage. Use the `aws.apigatewayv2.Deployment` resource to configure a deployment.
     */
    deploymentId?: pulumi.Input<string>;
    /**
     * Description for the stage. Must be less than or equal to 1024 characters in length.
     */
    description?: pulumi.Input<string>;
    /**
     * ARN prefix to be used in an `aws.lambda.Permission`'s `sourceArn` attribute.
     * For WebSocket APIs this attribute can additionally be used in an `aws.iam.Policy` to authorize access to the [`@connections` API](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-how-to-call-websocket-api-connections.html).
     * See the [Amazon API Gateway Developer Guide](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-control-access-iam.html) for details.
     */
    executionArn?: pulumi.Input<string>;
    /**
     * URL to invoke the API pointing to the stage,
     * e.g., `wss://z4675bid1j.execute-api.eu-west-2.amazonaws.com/example-stage`, or `https://z4675bid1j.execute-api.eu-west-2.amazonaws.com/`
     */
    invokeUrl?: pulumi.Input<string>;
    /**
     * Name of the stage. Must be between 1 and 128 characters in length.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Route settings for the stage.
     */
    routeSettings?: pulumi.Input<pulumi.Input<inputs.apigatewayv2.StageRouteSetting>[]>;
    /**
     * Map that defines the stage variables for the stage.
     */
    stageVariables?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags to assign to the stage. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Stage resource.
 */
export interface StageArgs {
    /**
     * Settings for logging access in this stage.
     * Use the `aws.apigateway.Account` resource to configure [permissions for CloudWatch Logging](https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-logging.html#set-up-access-logging-permissions).
     */
    accessLogSettings?: pulumi.Input<inputs.apigatewayv2.StageAccessLogSettings>;
    /**
     * API identifier.
     */
    apiId: pulumi.Input<string>;
    /**
     * Whether updates to an API automatically trigger a new deployment. Defaults to `false`. Applicable for HTTP APIs.
     */
    autoDeploy?: pulumi.Input<boolean>;
    /**
     * Identifier of a client certificate for the stage. Use the `aws.apigateway.ClientCertificate` resource to configure a client certificate.
     * Supported only for WebSocket APIs.
     */
    clientCertificateId?: pulumi.Input<string>;
    /**
     * Default route settings for the stage.
     */
    defaultRouteSettings?: pulumi.Input<inputs.apigatewayv2.StageDefaultRouteSettings>;
    /**
     * Deployment identifier of the stage. Use the `aws.apigatewayv2.Deployment` resource to configure a deployment.
     */
    deploymentId?: pulumi.Input<string>;
    /**
     * Description for the stage. Must be less than or equal to 1024 characters in length.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the stage. Must be between 1 and 128 characters in length.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Route settings for the stage.
     */
    routeSettings?: pulumi.Input<pulumi.Input<inputs.apigatewayv2.StageRouteSetting>[]>;
    /**
     * Map that defines the stage variables for the stage.
     */
    stageVariables?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags to assign to the stage. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
