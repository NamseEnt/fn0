import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
import { RestApi } from "./index";
/**
 * Manages API Gateway Stage Method Settings. For example, CloudWatch logging and metrics.
 *
 * > **NOTE:** We recommend using this resource in conjunction with the `aws.apigateway.Stage` resource instead of a stage managed by the `aws.apigateway.Deployment` resource optional `stageName` argument. Stages managed by the `aws.apigateway.Deployment` resource are recreated on redeployment and this resource will require a second apply to recreate the method settings.
 *
 * ## Example Usage
 *
 * ### End-to-end
 *
 * ### Off
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const pathSpecific = new aws.apigateway.MethodSettings("path_specific", {
 *     restApi: example.id,
 *     stageName: exampleAwsApiGatewayStage.stageName,
 *     methodPath: "path1/GET",
 *     settings: {
 *         loggingLevel: "OFF",
 *     },
 * });
 * ```
 *
 * ### Errors Only
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const pathSpecific = new aws.apigateway.MethodSettings("path_specific", {
 *     restApi: example.id,
 *     stageName: exampleAwsApiGatewayStage.stageName,
 *     methodPath: "path1/GET",
 *     settings: {
 *         loggingLevel: "ERROR",
 *         metricsEnabled: true,
 *         dataTraceEnabled: false,
 *     },
 * });
 * ```
 *
 * ### Errors and Info Logs
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const pathSpecific = new aws.apigateway.MethodSettings("path_specific", {
 *     restApi: example.id,
 *     stageName: exampleAwsApiGatewayStage.stageName,
 *     methodPath: "path1/GET",
 *     settings: {
 *         loggingLevel: "INFO",
 *         metricsEnabled: true,
 *         dataTraceEnabled: false,
 *     },
 * });
 * ```
 *
 * ### Full Request and Response Logs
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const pathSpecific = new aws.apigateway.MethodSettings("path_specific", {
 *     restApi: example.id,
 *     stageName: exampleAwsApiGatewayStage.stageName,
 *     methodPath: "path1/GET",
 *     settings: {
 *         loggingLevel: "INFO",
 *         metricsEnabled: true,
 *         dataTraceEnabled: true,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_api_gateway_method_settings` using `REST-API-ID/STAGE-NAME/METHOD-PATH`. For example:
 *
 * ```sh
 * $ pulumi import aws:apigateway/methodSettings:MethodSettings example 12345abcde/example/test/GET
 * ```
 */
export declare class MethodSettings extends pulumi.CustomResource {
    /**
     * Get an existing MethodSettings resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MethodSettingsState, opts?: pulumi.CustomResourceOptions): MethodSettings;
    /**
     * Returns true if the given object is an instance of MethodSettings.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is MethodSettings;
    /**
     * Method path defined as `{resource_path}/{http_method}` for an individual method override, or `*&#47;*` for overriding all methods in the stage. Ensure to trim any leading forward slashes in the path (e.g., `trimprefix(aws_api_gateway_resource.example.path, "/")`).
     */
    readonly methodPath: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID of the REST API
     */
    readonly restApi: pulumi.Output<string>;
    /**
     * Settings block, see below.
     */
    readonly settings: pulumi.Output<outputs.apigateway.MethodSettingsSettings>;
    /**
     * Name of the stage
     */
    readonly stageName: pulumi.Output<string>;
    /**
     * Create a MethodSettings resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MethodSettingsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering MethodSettings resources.
 */
export interface MethodSettingsState {
    /**
     * Method path defined as `{resource_path}/{http_method}` for an individual method override, or `*&#47;*` for overriding all methods in the stage. Ensure to trim any leading forward slashes in the path (e.g., `trimprefix(aws_api_gateway_resource.example.path, "/")`).
     */
    methodPath?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the REST API
     */
    restApi?: pulumi.Input<string | RestApi>;
    /**
     * Settings block, see below.
     */
    settings?: pulumi.Input<inputs.apigateway.MethodSettingsSettings>;
    /**
     * Name of the stage
     */
    stageName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a MethodSettings resource.
 */
export interface MethodSettingsArgs {
    /**
     * Method path defined as `{resource_path}/{http_method}` for an individual method override, or `*&#47;*` for overriding all methods in the stage. Ensure to trim any leading forward slashes in the path (e.g., `trimprefix(aws_api_gateway_resource.example.path, "/")`).
     */
    methodPath: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the REST API
     */
    restApi: pulumi.Input<string | RestApi>;
    /**
     * Settings block, see below.
     */
    settings: pulumi.Input<inputs.apigateway.MethodSettingsSettings>;
    /**
     * Name of the stage
     */
    stageName: pulumi.Input<string>;
}
