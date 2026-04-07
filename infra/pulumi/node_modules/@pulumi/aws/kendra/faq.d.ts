import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Kendra FAQ.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kendra.Faq("example", {
 *     indexId: exampleAwsKendraIndex.id,
 *     name: "Example",
 *     roleArn: exampleAwsIamRole.arn,
 *     s3Path: {
 *         bucket: exampleAwsS3Bucket.id,
 *         key: exampleAwsS3Object.key,
 *     },
 *     tags: {
 *         Name: "Example Kendra Faq",
 *     },
 * });
 * ```
 *
 * ### With File Format
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kendra.Faq("example", {
 *     indexId: exampleAwsKendraIndex.id,
 *     name: "Example",
 *     fileFormat: "CSV",
 *     roleArn: exampleAwsIamRole.arn,
 *     s3Path: {
 *         bucket: exampleAwsS3Bucket.id,
 *         key: exampleAwsS3Object.key,
 *     },
 * });
 * ```
 *
 * ### With Language Code
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kendra.Faq("example", {
 *     indexId: exampleAwsKendraIndex.id,
 *     name: "Example",
 *     languageCode: "en",
 *     roleArn: exampleAwsIamRole.arn,
 *     s3Path: {
 *         bucket: exampleAwsS3Bucket.id,
 *         key: exampleAwsS3Object.key,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_kendra_faq` using the unique identifiers of the FAQ and index separated by a slash (`/`). For example:
 *
 * ```sh
 * $ pulumi import aws:kendra/faq:Faq example faq-123456780/idx-8012925589
 * ```
 */
export declare class Faq extends pulumi.CustomResource {
    /**
     * Get an existing Faq resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FaqState, opts?: pulumi.CustomResourceOptions): Faq;
    /**
     * Returns true if the given object is an instance of Faq.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Faq;
    /**
     * ARN of the FAQ.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The Unix datetime that the FAQ was created.
     */
    readonly createdAt: pulumi.Output<string>;
    readonly description: pulumi.Output<string | undefined>;
    /**
     * When the Status field value is `FAILED`, this contains a message that explains why.
     */
    readonly errorMessage: pulumi.Output<string>;
    /**
     * The identifier of the FAQ.
     */
    readonly faqId: pulumi.Output<string>;
    readonly fileFormat: pulumi.Output<string | undefined>;
    /**
     * The identifier of the index for a FAQ.
     */
    readonly indexId: pulumi.Output<string>;
    readonly languageCode: pulumi.Output<string>;
    /**
     * The name that should be associated with the FAQ.
     */
    readonly name: pulumi.Output<string>;
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of a role with permission to access the S3 bucket that contains the FAQs. For more information, see [IAM Roles for Amazon Kendra](https://docs.aws.amazon.com/kendra/latest/dg/iam-roles.html).
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * The S3 location of the FAQ input data. Detailed below.
     */
    readonly s3Path: pulumi.Output<outputs.kendra.FaqS3Path>;
    /**
     * The status of the FAQ. It is ready to use when the status is ACTIVE.
     */
    readonly status: pulumi.Output<string>;
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
     * The date and time that the FAQ was last updated.
     */
    readonly updatedAt: pulumi.Output<string>;
    /**
     * Create a Faq resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FaqArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Faq resources.
 */
export interface FaqState {
    /**
     * ARN of the FAQ.
     */
    arn?: pulumi.Input<string>;
    /**
     * The Unix datetime that the FAQ was created.
     */
    createdAt?: pulumi.Input<string>;
    description?: pulumi.Input<string>;
    /**
     * When the Status field value is `FAILED`, this contains a message that explains why.
     */
    errorMessage?: pulumi.Input<string>;
    /**
     * The identifier of the FAQ.
     */
    faqId?: pulumi.Input<string>;
    fileFormat?: pulumi.Input<string>;
    /**
     * The identifier of the index for a FAQ.
     */
    indexId?: pulumi.Input<string>;
    languageCode?: pulumi.Input<string>;
    /**
     * The name that should be associated with the FAQ.
     */
    name?: pulumi.Input<string>;
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of a role with permission to access the S3 bucket that contains the FAQs. For more information, see [IAM Roles for Amazon Kendra](https://docs.aws.amazon.com/kendra/latest/dg/iam-roles.html).
     */
    roleArn?: pulumi.Input<string>;
    /**
     * The S3 location of the FAQ input data. Detailed below.
     */
    s3Path?: pulumi.Input<inputs.kendra.FaqS3Path>;
    /**
     * The status of the FAQ. It is ready to use when the status is ACTIVE.
     */
    status?: pulumi.Input<string>;
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
     * The date and time that the FAQ was last updated.
     */
    updatedAt?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Faq resource.
 */
export interface FaqArgs {
    description?: pulumi.Input<string>;
    fileFormat?: pulumi.Input<string>;
    /**
     * The identifier of the index for a FAQ.
     */
    indexId: pulumi.Input<string>;
    languageCode?: pulumi.Input<string>;
    /**
     * The name that should be associated with the FAQ.
     */
    name?: pulumi.Input<string>;
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of a role with permission to access the S3 bucket that contains the FAQs. For more information, see [IAM Roles for Amazon Kendra](https://docs.aws.amazon.com/kendra/latest/dg/iam-roles.html).
     */
    roleArn: pulumi.Input<string>;
    /**
     * The S3 location of the FAQ input data. Detailed below.
     */
    s3Path: pulumi.Input<inputs.kendra.FaqS3Path>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
