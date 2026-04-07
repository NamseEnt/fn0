import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Step Function Activity resource
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const sfnActivity = new aws.sfn.Activity("sfn_activity", {name: "my-activity"});
 * ```
 *
 * ### Encryption
 *
 * > *NOTE:* See the section [Data at rest encyption](https://docs.aws.amazon.com/step-functions/latest/dg/encryption-at-rest.html) in the [AWS Step Functions Developer Guide](https://docs.aws.amazon.com/step-functions/latest/dg/welcome.html) for more information about enabling encryption of data using a customer-managed key for Step Functions State Machines data.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const sfnActivity = new aws.sfn.Activity("sfn_activity", {
 *     name: "my-activity",
 *     encryptionConfiguration: {
 *         kmsKeyId: kmsKeyForSfn.arn,
 *         type: "CUSTOMER_MANAGED_KMS_KEY",
 *         kmsDataKeyReusePeriodSeconds: 900,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * In Terraform v1.12.0 and later, the `import` block can be used with the `identity` attribute. For example:
 *
 * Using `pulumi import`, import activities using the `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:sfn/activity:Activity example arn:aws:states:eu-west-1:123456789098:activity:bar
 * ```
 */
export declare class Activity extends pulumi.CustomResource {
    /**
     * Get an existing Activity resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ActivityState, opts?: pulumi.CustomResourceOptions): Activity;
    /**
     * Returns true if the given object is an instance of Activity.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Activity;
    /**
     * Amazon Resource Name (ARN) of the activity.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Date the activity was created.
     */
    readonly creationDate: pulumi.Output<string>;
    /**
     * Defines what encryption configuration is used to encrypt data in the Activity. For more information see the section [Data at rest encyption](https://docs.aws.amazon.com/step-functions/latest/dg/encryption-at-rest.html) in the AWS Step Functions User Guide.
     */
    readonly encryptionConfiguration: pulumi.Output<outputs.sfn.ActivityEncryptionConfiguration>;
    /**
     * The name of the activity to create.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Activity resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ActivityArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Activity resources.
 */
export interface ActivityState {
    /**
     * Amazon Resource Name (ARN) of the activity.
     */
    arn?: pulumi.Input<string>;
    /**
     * Date the activity was created.
     */
    creationDate?: pulumi.Input<string>;
    /**
     * Defines what encryption configuration is used to encrypt data in the Activity. For more information see the section [Data at rest encyption](https://docs.aws.amazon.com/step-functions/latest/dg/encryption-at-rest.html) in the AWS Step Functions User Guide.
     */
    encryptionConfiguration?: pulumi.Input<inputs.sfn.ActivityEncryptionConfiguration>;
    /**
     * The name of the activity to create.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a Activity resource.
 */
export interface ActivityArgs {
    /**
     * Defines what encryption configuration is used to encrypt data in the Activity. For more information see the section [Data at rest encyption](https://docs.aws.amazon.com/step-functions/latest/dg/encryption-at-rest.html) in the AWS Step Functions User Guide.
     */
    encryptionConfiguration?: pulumi.Input<inputs.sfn.ActivityEncryptionConfiguration>;
    /**
     * The name of the activity to create.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
