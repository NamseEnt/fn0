import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Athena Workgroup.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.athena.Workgroup("example", {
 *     name: "example",
 *     configuration: {
 *         enforceWorkgroupConfiguration: true,
 *         publishCloudwatchMetricsEnabled: true,
 *         resultConfiguration: {
 *             outputLocation: `s3://${exampleAwsS3Bucket.bucket}/output/`,
 *             encryptionConfiguration: {
 *                 encryptionOption: "SSE_KMS",
 *                 kmsKeyArn: exampleAwsKmsKey.arn,
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Athena Workgroups using their name. For example:
 *
 * ```sh
 * $ pulumi import aws:athena/workgroup:Workgroup example example
 * ```
 */
export declare class Workgroup extends pulumi.CustomResource {
    /**
     * Get an existing Workgroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WorkgroupState, opts?: pulumi.CustomResourceOptions): Workgroup;
    /**
     * Returns true if the given object is an instance of Workgroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Workgroup;
    /**
     * ARN of the workgroup
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Configuration block with various settings for the workgroup. Documented below.
     */
    readonly configuration: pulumi.Output<outputs.athena.WorkgroupConfiguration | undefined>;
    /**
     * Description of the workgroup.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Option to delete the workgroup and its contents even if the workgroup contains any named queries.
     */
    readonly forceDestroy: pulumi.Output<boolean | undefined>;
    /**
     * Name of the workgroup.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * State of the workgroup. Valid values are `DISABLED` or `ENABLED`. Defaults to `ENABLED`.
     */
    readonly state: pulumi.Output<string | undefined>;
    /**
     * Key-value map of resource tags for the workgroup. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Workgroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: WorkgroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Workgroup resources.
 */
export interface WorkgroupState {
    /**
     * ARN of the workgroup
     */
    arn?: pulumi.Input<string>;
    /**
     * Configuration block with various settings for the workgroup. Documented below.
     */
    configuration?: pulumi.Input<inputs.athena.WorkgroupConfiguration>;
    /**
     * Description of the workgroup.
     */
    description?: pulumi.Input<string>;
    /**
     * Option to delete the workgroup and its contents even if the workgroup contains any named queries.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * Name of the workgroup.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * State of the workgroup. Valid values are `DISABLED` or `ENABLED`. Defaults to `ENABLED`.
     */
    state?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the workgroup. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Workgroup resource.
 */
export interface WorkgroupArgs {
    /**
     * Configuration block with various settings for the workgroup. Documented below.
     */
    configuration?: pulumi.Input<inputs.athena.WorkgroupConfiguration>;
    /**
     * Description of the workgroup.
     */
    description?: pulumi.Input<string>;
    /**
     * Option to delete the workgroup and its contents even if the workgroup contains any named queries.
     */
    forceDestroy?: pulumi.Input<boolean>;
    /**
     * Name of the workgroup.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * State of the workgroup. Valid values are `DISABLED` or `ENABLED`. Defaults to `ENABLED`.
     */
    state?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the workgroup. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
