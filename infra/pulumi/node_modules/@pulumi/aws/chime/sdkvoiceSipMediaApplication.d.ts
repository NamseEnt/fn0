import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * A ChimeSDKVoice SIP Media Application is a managed object that passes values from a SIP rule to a target AWS Lambda function.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.chime.SdkvoiceSipMediaApplication("example", {
 *     awsRegion: "us-east-1",
 *     name: "example-sip-media-application",
 *     endpoints: {
 *         lambdaArn: test.arn,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a ChimeSDKVoice SIP Media Application using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:chime/sdkvoiceSipMediaApplication:SdkvoiceSipMediaApplication example abcdef123456
 * ```
 */
export declare class SdkvoiceSipMediaApplication extends pulumi.CustomResource {
    /**
     * Get an existing SdkvoiceSipMediaApplication resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SdkvoiceSipMediaApplicationState, opts?: pulumi.CustomResourceOptions): SdkvoiceSipMediaApplication;
    /**
     * Returns true if the given object is an instance of SdkvoiceSipMediaApplication.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SdkvoiceSipMediaApplication;
    /**
     * ARN (Amazon Resource Name) of the AWS Chime SDK Voice Sip Media Application
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The AWS Region in which the AWS Chime SDK Voice Sip Media Application is created.
     */
    readonly awsRegion: pulumi.Output<string>;
    /**
     * List of endpoints (Lambda Amazon Resource Names) specified for the SIP media application. Currently, only one endpoint is supported. See `endpoints`.
     */
    readonly endpoints: pulumi.Output<outputs.chime.SdkvoiceSipMediaApplicationEndpoints>;
    /**
     * The name of the AWS Chime SDK Voice Sip Media Application.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a SdkvoiceSipMediaApplication resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SdkvoiceSipMediaApplicationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SdkvoiceSipMediaApplication resources.
 */
export interface SdkvoiceSipMediaApplicationState {
    /**
     * ARN (Amazon Resource Name) of the AWS Chime SDK Voice Sip Media Application
     */
    arn?: pulumi.Input<string>;
    /**
     * The AWS Region in which the AWS Chime SDK Voice Sip Media Application is created.
     */
    awsRegion?: pulumi.Input<string>;
    /**
     * List of endpoints (Lambda Amazon Resource Names) specified for the SIP media application. Currently, only one endpoint is supported. See `endpoints`.
     */
    endpoints?: pulumi.Input<inputs.chime.SdkvoiceSipMediaApplicationEndpoints>;
    /**
     * The name of the AWS Chime SDK Voice Sip Media Application.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a SdkvoiceSipMediaApplication resource.
 */
export interface SdkvoiceSipMediaApplicationArgs {
    /**
     * The AWS Region in which the AWS Chime SDK Voice Sip Media Application is created.
     */
    awsRegion: pulumi.Input<string>;
    /**
     * List of endpoints (Lambda Amazon Resource Names) specified for the SIP media application. Currently, only one endpoint is supported. See `endpoints`.
     */
    endpoints: pulumi.Input<inputs.chime.SdkvoiceSipMediaApplicationEndpoints>;
    /**
     * The name of the AWS Chime SDK Voice Sip Media Application.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
