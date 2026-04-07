import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Elastic Transcoder pipeline resource.
 *
 * > **Warning:** This resource is deprecated. Use [AWS Elemental MediaConvert](https://aws.amazon.com/blogs/media/migrating-workflows-from-amazon-elastic-transcoder-to-aws-elemental-mediaconvert/) instead. AWS will [discontinue support for Amazon Elastic Transcoder](https://aws.amazon.com/blogs/media/support-for-amazon-elastic-transcoder-ending-soon/), effective November 13, 2025.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const bar = new aws.elastictranscoder.Pipeline("bar", {
 *     inputBucket: inputBucket.id,
 *     name: "aws_elastictranscoder_pipeline_my_test_",
 *     role: testRole.arn,
 *     contentConfig: {
 *         bucket: contentBucket.id,
 *         storageClass: "Standard",
 *     },
 *     thumbnailConfig: {
 *         bucket: thumbBucket.id,
 *         storageClass: "Standard",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Elastic Transcoder pipelines using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:elastictranscoder/pipeline:Pipeline basic_pipeline 1407981661351-cttk8b
 * ```
 */
export declare class Pipeline extends pulumi.CustomResource {
    /**
     * Get an existing Pipeline resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PipelineState, opts?: pulumi.CustomResourceOptions): Pipeline;
    /**
     * Returns true if the given object is an instance of Pipeline.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Pipeline;
    /**
     * The ARN of the Elastictranscoder pipeline.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The AWS Key Management Service (AWS KMS) key that you want to use with this pipeline.
     */
    readonly awsKmsKeyArn: pulumi.Output<string | undefined>;
    /**
     * The ContentConfig object specifies information about the Amazon S3 bucket in which you want Elastic Transcoder to save transcoded files and playlists. (documented below)
     */
    readonly contentConfig: pulumi.Output<outputs.elastictranscoder.PipelineContentConfig>;
    /**
     * The permissions for the `contentConfig` object. (documented below)
     */
    readonly contentConfigPermissions: pulumi.Output<outputs.elastictranscoder.PipelineContentConfigPermission[] | undefined>;
    /**
     * The Amazon S3 bucket in which you saved the media files that you want to transcode and the graphics that you want to use as watermarks.
     */
    readonly inputBucket: pulumi.Output<string>;
    /**
     * The name of the pipeline. Maximum 40 characters
     */
    readonly name: pulumi.Output<string>;
    /**
     * The Amazon Simple Notification Service (Amazon SNS) topic that you want to notify to report job status. (documented below)
     */
    readonly notifications: pulumi.Output<outputs.elastictranscoder.PipelineNotifications | undefined>;
    /**
     * The Amazon S3 bucket in which you want Elastic Transcoder to save the transcoded files.
     */
    readonly outputBucket: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The IAM Amazon Resource Name (ARN) for the role that you want Elastic Transcoder to use to transcode jobs for this pipeline.
     */
    readonly role: pulumi.Output<string>;
    /**
     * The ThumbnailConfig object specifies information about the Amazon S3 bucket in which you want Elastic Transcoder to save thumbnail files. (documented below)
     */
    readonly thumbnailConfig: pulumi.Output<outputs.elastictranscoder.PipelineThumbnailConfig>;
    /**
     * The permissions for the `thumbnailConfig` object. (documented below)
     *
     * See ["Create Pipeline"](http://docs.aws.amazon.com/elastictranscoder/latest/developerguide/create-pipeline.html) in the AWS docs for reference.
     *
     * The `contentConfig` object specifies information about the Amazon S3 bucket in
     * which you want Elastic Transcoder to save transcoded files and playlists: which
     * bucket to use, and the storage class that you want to assign to the files. If
     * you specify values for `contentConfig`, you must also specify values for
     * `thumbnailConfig`. If you specify values for `contentConfig` and
     * `thumbnailConfig`, omit the `outputBucket` object.
     */
    readonly thumbnailConfigPermissions: pulumi.Output<outputs.elastictranscoder.PipelineThumbnailConfigPermission[] | undefined>;
    /**
     * Create a Pipeline resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PipelineArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Pipeline resources.
 */
export interface PipelineState {
    /**
     * The ARN of the Elastictranscoder pipeline.
     */
    arn?: pulumi.Input<string>;
    /**
     * The AWS Key Management Service (AWS KMS) key that you want to use with this pipeline.
     */
    awsKmsKeyArn?: pulumi.Input<string>;
    /**
     * The ContentConfig object specifies information about the Amazon S3 bucket in which you want Elastic Transcoder to save transcoded files and playlists. (documented below)
     */
    contentConfig?: pulumi.Input<inputs.elastictranscoder.PipelineContentConfig>;
    /**
     * The permissions for the `contentConfig` object. (documented below)
     */
    contentConfigPermissions?: pulumi.Input<pulumi.Input<inputs.elastictranscoder.PipelineContentConfigPermission>[]>;
    /**
     * The Amazon S3 bucket in which you saved the media files that you want to transcode and the graphics that you want to use as watermarks.
     */
    inputBucket?: pulumi.Input<string>;
    /**
     * The name of the pipeline. Maximum 40 characters
     */
    name?: pulumi.Input<string>;
    /**
     * The Amazon Simple Notification Service (Amazon SNS) topic that you want to notify to report job status. (documented below)
     */
    notifications?: pulumi.Input<inputs.elastictranscoder.PipelineNotifications>;
    /**
     * The Amazon S3 bucket in which you want Elastic Transcoder to save the transcoded files.
     */
    outputBucket?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM Amazon Resource Name (ARN) for the role that you want Elastic Transcoder to use to transcode jobs for this pipeline.
     */
    role?: pulumi.Input<string>;
    /**
     * The ThumbnailConfig object specifies information about the Amazon S3 bucket in which you want Elastic Transcoder to save thumbnail files. (documented below)
     */
    thumbnailConfig?: pulumi.Input<inputs.elastictranscoder.PipelineThumbnailConfig>;
    /**
     * The permissions for the `thumbnailConfig` object. (documented below)
     *
     * See ["Create Pipeline"](http://docs.aws.amazon.com/elastictranscoder/latest/developerguide/create-pipeline.html) in the AWS docs for reference.
     *
     * The `contentConfig` object specifies information about the Amazon S3 bucket in
     * which you want Elastic Transcoder to save transcoded files and playlists: which
     * bucket to use, and the storage class that you want to assign to the files. If
     * you specify values for `contentConfig`, you must also specify values for
     * `thumbnailConfig`. If you specify values for `contentConfig` and
     * `thumbnailConfig`, omit the `outputBucket` object.
     */
    thumbnailConfigPermissions?: pulumi.Input<pulumi.Input<inputs.elastictranscoder.PipelineThumbnailConfigPermission>[]>;
}
/**
 * The set of arguments for constructing a Pipeline resource.
 */
export interface PipelineArgs {
    /**
     * The AWS Key Management Service (AWS KMS) key that you want to use with this pipeline.
     */
    awsKmsKeyArn?: pulumi.Input<string>;
    /**
     * The ContentConfig object specifies information about the Amazon S3 bucket in which you want Elastic Transcoder to save transcoded files and playlists. (documented below)
     */
    contentConfig?: pulumi.Input<inputs.elastictranscoder.PipelineContentConfig>;
    /**
     * The permissions for the `contentConfig` object. (documented below)
     */
    contentConfigPermissions?: pulumi.Input<pulumi.Input<inputs.elastictranscoder.PipelineContentConfigPermission>[]>;
    /**
     * The Amazon S3 bucket in which you saved the media files that you want to transcode and the graphics that you want to use as watermarks.
     */
    inputBucket: pulumi.Input<string>;
    /**
     * The name of the pipeline. Maximum 40 characters
     */
    name?: pulumi.Input<string>;
    /**
     * The Amazon Simple Notification Service (Amazon SNS) topic that you want to notify to report job status. (documented below)
     */
    notifications?: pulumi.Input<inputs.elastictranscoder.PipelineNotifications>;
    /**
     * The Amazon S3 bucket in which you want Elastic Transcoder to save the transcoded files.
     */
    outputBucket?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM Amazon Resource Name (ARN) for the role that you want Elastic Transcoder to use to transcode jobs for this pipeline.
     */
    role: pulumi.Input<string>;
    /**
     * The ThumbnailConfig object specifies information about the Amazon S3 bucket in which you want Elastic Transcoder to save thumbnail files. (documented below)
     */
    thumbnailConfig?: pulumi.Input<inputs.elastictranscoder.PipelineThumbnailConfig>;
    /**
     * The permissions for the `thumbnailConfig` object. (documented below)
     *
     * See ["Create Pipeline"](http://docs.aws.amazon.com/elastictranscoder/latest/developerguide/create-pipeline.html) in the AWS docs for reference.
     *
     * The `contentConfig` object specifies information about the Amazon S3 bucket in
     * which you want Elastic Transcoder to save transcoded files and playlists: which
     * bucket to use, and the storage class that you want to assign to the files. If
     * you specify values for `contentConfig`, you must also specify values for
     * `thumbnailConfig`. If you specify values for `contentConfig` and
     * `thumbnailConfig`, omit the `outputBucket` object.
     */
    thumbnailConfigPermissions?: pulumi.Input<pulumi.Input<inputs.elastictranscoder.PipelineThumbnailConfigPermission>[]>;
}
