import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a SageMaker AI App resource.
 *
 * ## Example Usage
 *
 * ### Basic usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sagemaker.App("example", {
 *     domainId: exampleAwsSagemakerDomain.id,
 *     userProfileName: exampleAwsSagemakerUserProfile.userProfileName,
 *     appName: "example",
 *     appType: "JupyterServer",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SageMaker AI Apps using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/app:App example arn:aws:sagemaker:us-west-2:012345678912:app/domain-id/user-profile-name/app-type/app-name
 * ```
 */
export declare class App extends pulumi.CustomResource {
    /**
     * Get an existing App resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AppState, opts?: pulumi.CustomResourceOptions): App;
    /**
     * Returns true if the given object is an instance of App.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is App;
    /**
     * The name of the app.
     */
    readonly appName: pulumi.Output<string>;
    /**
     * The type of app. Valid values are `JupyterServer`, `KernelGateway`, `RStudioServerPro`, `RSessionGateway`, `TensorBoard`, `CodeEditor`, `JupyterLab`, `DetailedProfiler`, and `Canvas`.
     */
    readonly appType: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the app.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The domain ID.
     */
    readonly domainId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The instance type and the Amazon Resource Name (ARN) of the SageMaker AI image created on the instance.See Resource Spec below.
     */
    readonly resourceSpec: pulumi.Output<outputs.sagemaker.AppResourceSpec>;
    /**
     * The name of the space. At least one of `userProfileName` or `spaceName` required.
     */
    readonly spaceName: pulumi.Output<string | undefined>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The user profile name. At least one of `userProfileName` or `spaceName` required.
     */
    readonly userProfileName: pulumi.Output<string | undefined>;
    /**
     * Create a App resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AppArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering App resources.
 */
export interface AppState {
    /**
     * The name of the app.
     */
    appName?: pulumi.Input<string>;
    /**
     * The type of app. Valid values are `JupyterServer`, `KernelGateway`, `RStudioServerPro`, `RSessionGateway`, `TensorBoard`, `CodeEditor`, `JupyterLab`, `DetailedProfiler`, and `Canvas`.
     */
    appType?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the app.
     */
    arn?: pulumi.Input<string>;
    /**
     * The domain ID.
     */
    domainId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The instance type and the Amazon Resource Name (ARN) of the SageMaker AI image created on the instance.See Resource Spec below.
     */
    resourceSpec?: pulumi.Input<inputs.sagemaker.AppResourceSpec>;
    /**
     * The name of the space. At least one of `userProfileName` or `spaceName` required.
     */
    spaceName?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * The user profile name. At least one of `userProfileName` or `spaceName` required.
     */
    userProfileName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a App resource.
 */
export interface AppArgs {
    /**
     * The name of the app.
     */
    appName: pulumi.Input<string>;
    /**
     * The type of app. Valid values are `JupyterServer`, `KernelGateway`, `RStudioServerPro`, `RSessionGateway`, `TensorBoard`, `CodeEditor`, `JupyterLab`, `DetailedProfiler`, and `Canvas`.
     */
    appType: pulumi.Input<string>;
    /**
     * The domain ID.
     */
    domainId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The instance type and the Amazon Resource Name (ARN) of the SageMaker AI image created on the instance.See Resource Spec below.
     */
    resourceSpec?: pulumi.Input<inputs.sagemaker.AppResourceSpec>;
    /**
     * The name of the space. At least one of `userProfileName` or `spaceName` required.
     */
    spaceName?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The user profile name. At least one of `userProfileName` or `spaceName` required.
     */
    userProfileName?: pulumi.Input<string>;
}
