import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Q Business Application resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.qbusiness.Application("example", {
 *     displayName: "example-app",
 *     iamServiceRoleArn: exampleAwsIamRole.arn,
 *     identityCenterInstanceArn: exampleAwsSsoadminInstances.arns[0],
 *     attachmentsConfiguration: {
 *         attachmentsControlMode: "ENABLED",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a Q Business Application using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:qbusiness/application:Application example id-12345678
 * ```
 */
export declare class Application extends pulumi.CustomResource {
    /**
     * Get an existing Application resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ApplicationState, opts?: pulumi.CustomResourceOptions): Application;
    /**
     * Returns true if the given object is an instance of Application.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Application;
    /**
     * ARN of the Q Business application.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Information about whether file upload functionality is activated or deactivated for your end user. See `attachmentsConfiguration` below.
     */
    readonly attachmentsConfiguration: pulumi.Output<outputs.qbusiness.ApplicationAttachmentsConfiguration | undefined>;
    /**
     * Description of the Amazon Q application.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Name of the Amazon Q application.
     */
    readonly displayName: pulumi.Output<string>;
    /**
     * Information about encryption configuration. See `encryptionConfiguration` below.
     */
    readonly encryptionConfiguration: pulumi.Output<outputs.qbusiness.ApplicationEncryptionConfiguration | undefined>;
    /**
     * ARN of an IAM role with permissions to access your Amazon CloudWatch logs and metrics.
     */
    readonly iamServiceRoleArn: pulumi.Output<string>;
    /**
     * ARN of the AWS IAM Identity Center application attached to your Amazon Q Business application.
     */
    readonly identityCenterApplicationArn: pulumi.Output<string>;
    /**
     * ARN of the IAM Identity Center instance you are either creating for — or connecting to — your Amazon Q Business application.
     *
     * The following arguments are optional:
     */
    readonly identityCenterInstanceArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.qbusiness.ApplicationTimeouts | undefined>;
    /**
     * Create a Application resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ApplicationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Application resources.
 */
export interface ApplicationState {
    /**
     * ARN of the Q Business application.
     */
    arn?: pulumi.Input<string>;
    /**
     * Information about whether file upload functionality is activated or deactivated for your end user. See `attachmentsConfiguration` below.
     */
    attachmentsConfiguration?: pulumi.Input<inputs.qbusiness.ApplicationAttachmentsConfiguration>;
    /**
     * Description of the Amazon Q application.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the Amazon Q application.
     */
    displayName?: pulumi.Input<string>;
    /**
     * Information about encryption configuration. See `encryptionConfiguration` below.
     */
    encryptionConfiguration?: pulumi.Input<inputs.qbusiness.ApplicationEncryptionConfiguration>;
    /**
     * ARN of an IAM role with permissions to access your Amazon CloudWatch logs and metrics.
     */
    iamServiceRoleArn?: pulumi.Input<string>;
    /**
     * ARN of the AWS IAM Identity Center application attached to your Amazon Q Business application.
     */
    identityCenterApplicationArn?: pulumi.Input<string>;
    /**
     * ARN of the IAM Identity Center instance you are either creating for — or connecting to — your Amazon Q Business application.
     *
     * The following arguments are optional:
     */
    identityCenterInstanceArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.qbusiness.ApplicationTimeouts>;
}
/**
 * The set of arguments for constructing a Application resource.
 */
export interface ApplicationArgs {
    /**
     * Information about whether file upload functionality is activated or deactivated for your end user. See `attachmentsConfiguration` below.
     */
    attachmentsConfiguration?: pulumi.Input<inputs.qbusiness.ApplicationAttachmentsConfiguration>;
    /**
     * Description of the Amazon Q application.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the Amazon Q application.
     */
    displayName: pulumi.Input<string>;
    /**
     * Information about encryption configuration. See `encryptionConfiguration` below.
     */
    encryptionConfiguration?: pulumi.Input<inputs.qbusiness.ApplicationEncryptionConfiguration>;
    /**
     * ARN of an IAM role with permissions to access your Amazon CloudWatch logs and metrics.
     */
    iamServiceRoleArn: pulumi.Input<string>;
    /**
     * ARN of the IAM Identity Center instance you are either creating for — or connecting to — your Amazon Q Business application.
     *
     * The following arguments are optional:
     */
    identityCenterInstanceArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.qbusiness.ApplicationTimeouts>;
}
