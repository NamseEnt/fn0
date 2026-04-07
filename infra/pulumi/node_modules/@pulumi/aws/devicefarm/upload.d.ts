import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage AWS Device Farm Uploads.
 *
 * > **NOTE:** AWS currently has limited regional support for Device Farm (e.g., `us-west-2`). See [AWS Device Farm endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/devicefarm.html) for information on supported regions.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.devicefarm.Project("example", {name: "example"});
 * const exampleUpload = new aws.devicefarm.Upload("example", {
 *     name: "example",
 *     projectArn: example.arn,
 *     type: "APPIUM_JAVA_TESTNG_TEST_SPEC",
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Device Farm upload.
 *
 * Using `pulumi import`, import DeviceFarm Uploads using their ARN. For example:
 *
 * console
 *
 * % pulumi import aws_devicefarm_upload.example arn:aws:devicefarm:us-west-2:123456789012:upload:4fa784c7-ccb4-4dbf-ba4f-02198320daa1
 */
export declare class Upload extends pulumi.CustomResource {
    /**
     * Get an existing Upload resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UploadState, opts?: pulumi.CustomResourceOptions): Upload;
    /**
     * Returns true if the given object is an instance of Upload.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Upload;
    /**
     * The Amazon Resource Name of this upload.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The upload's category.
     */
    readonly category: pulumi.Output<string>;
    /**
     * The upload's content type (for example, application/octet-stream).
     */
    readonly contentType: pulumi.Output<string | undefined>;
    /**
     * The upload's metadata. For example, for Android, this contains information that is parsed from the manifest and is displayed in the AWS Device Farm console after the associated app is uploaded.
     */
    readonly metadata: pulumi.Output<string>;
    /**
     * The upload's file name. The name should not contain any forward slashes (/). If you are uploading an iOS app, the file name must end with the .ipa extension. If you are uploading an Android app, the file name must end with the .apk extension. For all others, the file name must end with the .zip file extension.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The ARN of the project for the upload.
     */
    readonly projectArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The upload's upload type. See [AWS Docs](https://docs.aws.amazon.com/devicefarm/latest/APIReference/API_CreateUpload.html#API_CreateUpload_RequestSyntax) for valid list of values.
     */
    readonly type: pulumi.Output<string>;
    /**
     * The presigned Amazon S3 URL that was used to store a file using a PUT request.
     */
    readonly url: pulumi.Output<string>;
    /**
     * Create a Upload resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UploadArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Upload resources.
 */
export interface UploadState {
    /**
     * The Amazon Resource Name of this upload.
     */
    arn?: pulumi.Input<string>;
    /**
     * The upload's category.
     */
    category?: pulumi.Input<string>;
    /**
     * The upload's content type (for example, application/octet-stream).
     */
    contentType?: pulumi.Input<string>;
    /**
     * The upload's metadata. For example, for Android, this contains information that is parsed from the manifest and is displayed in the AWS Device Farm console after the associated app is uploaded.
     */
    metadata?: pulumi.Input<string>;
    /**
     * The upload's file name. The name should not contain any forward slashes (/). If you are uploading an iOS app, the file name must end with the .ipa extension. If you are uploading an Android app, the file name must end with the .apk extension. For all others, the file name must end with the .zip file extension.
     */
    name?: pulumi.Input<string>;
    /**
     * The ARN of the project for the upload.
     */
    projectArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The upload's upload type. See [AWS Docs](https://docs.aws.amazon.com/devicefarm/latest/APIReference/API_CreateUpload.html#API_CreateUpload_RequestSyntax) for valid list of values.
     */
    type?: pulumi.Input<string>;
    /**
     * The presigned Amazon S3 URL that was used to store a file using a PUT request.
     */
    url?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Upload resource.
 */
export interface UploadArgs {
    /**
     * The upload's content type (for example, application/octet-stream).
     */
    contentType?: pulumi.Input<string>;
    /**
     * The upload's file name. The name should not contain any forward slashes (/). If you are uploading an iOS app, the file name must end with the .ipa extension. If you are uploading an Android app, the file name must end with the .apk extension. For all others, the file name must end with the .zip file extension.
     */
    name?: pulumi.Input<string>;
    /**
     * The ARN of the project for the upload.
     */
    projectArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The upload's upload type. See [AWS Docs](https://docs.aws.amazon.com/devicefarm/latest/APIReference/API_CreateUpload.html#API_CreateUpload_RequestSyntax) for valid list of values.
     */
    type: pulumi.Input<string>;
}
