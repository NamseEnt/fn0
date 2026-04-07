import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Kendra Thesaurus.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kendra.Thesaurus("example", {
 *     indexId: exampleAwsKendraIndex.id,
 *     name: "Example",
 *     roleArn: exampleAwsIamRole.arn,
 *     sourceS3Path: {
 *         bucket: exampleAwsS3Bucket.id,
 *         key: exampleAwsS3Object.key,
 *     },
 *     tags: {
 *         Name: "Example Kendra Thesaurus",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_kendra_thesaurus` using the unique identifiers of the thesaurus and index separated by a slash (`/`). For example:
 *
 * ```sh
 * $ pulumi import aws:kendra/thesaurus:Thesaurus example thesaurus-123456780/idx-8012925589
 * ```
 */
export declare class Thesaurus extends pulumi.CustomResource {
    /**
     * Get an existing Thesaurus resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ThesaurusState, opts?: pulumi.CustomResourceOptions): Thesaurus;
    /**
     * Returns true if the given object is an instance of Thesaurus.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Thesaurus;
    /**
     * ARN of the thesaurus.
     */
    readonly arn: pulumi.Output<string>;
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The identifier of the index for a thesaurus.
     */
    readonly indexId: pulumi.Output<string>;
    /**
     * The name for the thesaurus.
     */
    readonly name: pulumi.Output<string>;
    readonly region: pulumi.Output<string>;
    /**
     * The IAM (Identity and Access Management) role used to access the thesaurus file in S3.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * The S3 path where your thesaurus file sits in S3. Detailed below.
     */
    readonly sourceS3Path: pulumi.Output<outputs.kendra.ThesaurusSourceS3Path>;
    /**
     * The current status of the thesaurus.
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
    readonly thesaurusId: pulumi.Output<string>;
    /**
     * Create a Thesaurus resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ThesaurusArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Thesaurus resources.
 */
export interface ThesaurusState {
    /**
     * ARN of the thesaurus.
     */
    arn?: pulumi.Input<string>;
    description?: pulumi.Input<string>;
    /**
     * The identifier of the index for a thesaurus.
     */
    indexId?: pulumi.Input<string>;
    /**
     * The name for the thesaurus.
     */
    name?: pulumi.Input<string>;
    region?: pulumi.Input<string>;
    /**
     * The IAM (Identity and Access Management) role used to access the thesaurus file in S3.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * The S3 path where your thesaurus file sits in S3. Detailed below.
     */
    sourceS3Path?: pulumi.Input<inputs.kendra.ThesaurusSourceS3Path>;
    /**
     * The current status of the thesaurus.
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
    thesaurusId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Thesaurus resource.
 */
export interface ThesaurusArgs {
    description?: pulumi.Input<string>;
    /**
     * The identifier of the index for a thesaurus.
     */
    indexId: pulumi.Input<string>;
    /**
     * The name for the thesaurus.
     */
    name?: pulumi.Input<string>;
    region?: pulumi.Input<string>;
    /**
     * The IAM (Identity and Access Management) role used to access the thesaurus file in S3.
     */
    roleArn: pulumi.Input<string>;
    /**
     * The S3 path where your thesaurus file sits in S3. Detailed below.
     */
    sourceS3Path: pulumi.Input<inputs.kendra.ThesaurusSourceS3Path>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
