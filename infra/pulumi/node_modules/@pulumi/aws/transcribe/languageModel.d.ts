import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Transcribe LanguageModel.
 *
 * > This resource can take a significant amount of time to provision. See Language Model [FAQ](https://aws.amazon.com/transcribe/faqs/) for more details.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.iam.getPolicyDocument({
 *     statements: [{
 *         actions: ["sts:AssumeRole"],
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["transcribe.amazonaws.com"],
 *         }],
 *     }],
 * });
 * const exampleRole = new aws.iam.Role("example", {
 *     name: "example",
 *     assumeRolePolicy: example.then(example => example.json),
 * });
 * const testPolicy = new aws.iam.RolePolicy("test_policy", {
 *     name: "example",
 *     role: exampleRole.id,
 *     policy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: [
 *                 "s3:GetObject",
 *                 "s3:ListBucket",
 *             ],
 *             Effect: "Allow",
 *             Resource: ["*"],
 *         }],
 *     }),
 * });
 * const exampleBucket = new aws.s3.Bucket("example", {
 *     bucket: "example-transcribe",
 *     forceDestroy: true,
 * });
 * const object = new aws.s3.BucketObjectv2("object", {
 *     bucket: exampleBucket.id,
 *     key: "transcribe/test1.txt",
 *     source: new pulumi.asset.FileAsset("test1.txt"),
 * });
 * const exampleLanguageModel = new aws.transcribe.LanguageModel("example", {
 *     modelName: "example",
 *     baseModelName: "NarrowBand",
 *     inputDataConfig: {
 *         dataAccessRoleArn: exampleRole.arn,
 *         s3Uri: pulumi.interpolate`s3://${exampleBucket.id}/transcribe/`,
 *     },
 *     languageCode: "en-US",
 *     tags: {
 *         ENVIRONMENT: "development",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Transcribe LanguageModel using the `model_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:transcribe/languageModel:LanguageModel example example-name
 * ```
 */
export declare class LanguageModel extends pulumi.CustomResource {
    /**
     * Get an existing LanguageModel resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LanguageModelState, opts?: pulumi.CustomResourceOptions): LanguageModel;
    /**
     * Returns true if the given object is an instance of LanguageModel.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LanguageModel;
    /**
     * ARN of the LanguageModel.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Name of reference base model.
     */
    readonly baseModelName: pulumi.Output<string>;
    /**
     * The input data config for the LanguageModel. See Input Data Config for more details.
     */
    readonly inputDataConfig: pulumi.Output<outputs.transcribe.LanguageModelInputDataConfig>;
    /**
     * The language code you selected for your language model. Refer to the [supported languages](https://docs.aws.amazon.com/transcribe/latest/dg/supported-languages.html) page for accepted codes.
     */
    readonly languageCode: pulumi.Output<string>;
    /**
     * The model name.
     */
    readonly modelName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a LanguageModel resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LanguageModelArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LanguageModel resources.
 */
export interface LanguageModelState {
    /**
     * ARN of the LanguageModel.
     */
    arn?: pulumi.Input<string>;
    /**
     * Name of reference base model.
     */
    baseModelName?: pulumi.Input<string>;
    /**
     * The input data config for the LanguageModel. See Input Data Config for more details.
     */
    inputDataConfig?: pulumi.Input<inputs.transcribe.LanguageModelInputDataConfig>;
    /**
     * The language code you selected for your language model. Refer to the [supported languages](https://docs.aws.amazon.com/transcribe/latest/dg/supported-languages.html) page for accepted codes.
     */
    languageCode?: pulumi.Input<string>;
    /**
     * The model name.
     */
    modelName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a LanguageModel resource.
 */
export interface LanguageModelArgs {
    /**
     * Name of reference base model.
     */
    baseModelName: pulumi.Input<string>;
    /**
     * The input data config for the LanguageModel. See Input Data Config for more details.
     */
    inputDataConfig: pulumi.Input<inputs.transcribe.LanguageModelInputDataConfig>;
    /**
     * The language code you selected for your language model. Refer to the [supported languages](https://docs.aws.amazon.com/transcribe/latest/dg/supported-languages.html) page for accepted codes.
     */
    languageCode: pulumi.Input<string>;
    /**
     * The model name.
     */
    modelName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
