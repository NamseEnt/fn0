import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an EMR Containers (EMR on EKS) Job Template.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.emrcontainers.JobTemplate("example", {
 *     jobTemplateData: {
 *         executionRoleArn: exampleAwsIamRole.arn,
 *         releaseLabel: "emr-6.10.0-latest",
 *         jobDriver: {
 *             sparkSqlJobDriver: {
 *                 entryPoint: "default",
 *             },
 *         },
 *     },
 *     name: "example",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EKS job templates using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:emrcontainers/jobTemplate:JobTemplate example a1b2c3d4e5f6g7h8i9j10k11l
 * ```
 */
export declare class JobTemplate extends pulumi.CustomResource {
    /**
     * Get an existing JobTemplate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: JobTemplateState, opts?: pulumi.CustomResourceOptions): JobTemplate;
    /**
     * Returns true if the given object is an instance of JobTemplate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is JobTemplate;
    /**
     * ARN of the job template.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The job template data which holds values of StartJobRun API request.
     */
    readonly jobTemplateData: pulumi.Output<outputs.emrcontainers.JobTemplateJobTemplateData>;
    /**
     * The KMS key ARN used to encrypt the job template.
     */
    readonly kmsKeyArn: pulumi.Output<string | undefined>;
    /**
     * The specified name of the job template.
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
     * Create a JobTemplate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: JobTemplateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering JobTemplate resources.
 */
export interface JobTemplateState {
    /**
     * ARN of the job template.
     */
    arn?: pulumi.Input<string>;
    /**
     * The job template data which holds values of StartJobRun API request.
     */
    jobTemplateData?: pulumi.Input<inputs.emrcontainers.JobTemplateJobTemplateData>;
    /**
     * The KMS key ARN used to encrypt the job template.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * The specified name of the job template.
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
 * The set of arguments for constructing a JobTemplate resource.
 */
export interface JobTemplateArgs {
    /**
     * The job template data which holds values of StartJobRun API request.
     */
    jobTemplateData: pulumi.Input<inputs.emrcontainers.JobTemplateJobTemplateData>;
    /**
     * The KMS key ARN used to encrypt the job template.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * The specified name of the job template.
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
