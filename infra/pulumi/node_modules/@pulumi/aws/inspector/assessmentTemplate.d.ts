import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Inspector Classic Assessment Template
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.inspector.AssessmentTemplate("example", {
 *     name: "example",
 *     targetArn: exampleAwsInspectorAssessmentTarget.arn,
 *     duration: 3600,
 *     rulesPackageArns: [
 *         "arn:aws:inspector:us-west-2:758058086616:rulespackage/0-9hgA516p",
 *         "arn:aws:inspector:us-west-2:758058086616:rulespackage/0-H5hpSawc",
 *         "arn:aws:inspector:us-west-2:758058086616:rulespackage/0-JJOtZiqQ",
 *         "arn:aws:inspector:us-west-2:758058086616:rulespackage/0-vg5GGHSD",
 *     ],
 *     eventSubscriptions: [{
 *         event: "ASSESSMENT_RUN_COMPLETED",
 *         topicArn: exampleAwsSnsTopic.arn,
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Inspector assessment template.
 *
 * Using `pulumi import`, import `aws_inspector_assessment_template` using the template assessment ARN. For example:
 *
 * console
 *
 * % pulumi import aws_inspector_assessment_template.example arn:aws:inspector:us-west-2:123456789012:target/0-9IaAzhGR/template/0-WEcjR8CH
 */
export declare class AssessmentTemplate extends pulumi.CustomResource {
    /**
     * Get an existing AssessmentTemplate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AssessmentTemplateState, opts?: pulumi.CustomResourceOptions): AssessmentTemplate;
    /**
     * Returns true if the given object is an instance of AssessmentTemplate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AssessmentTemplate;
    /**
     * The template assessment ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The duration of the inspector run.
     */
    readonly duration: pulumi.Output<number>;
    /**
     * A block that enables sending notifications about a specified assessment template event to a designated SNS topic. See Event Subscriptions for details.
     */
    readonly eventSubscriptions: pulumi.Output<outputs.inspector.AssessmentTemplateEventSubscription[] | undefined>;
    /**
     * The name of the assessment template.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The rules to be used during the run.
     */
    readonly rulesPackageArns: pulumi.Output<string[]>;
    /**
     * Key-value map of tags for the Inspector assessment template. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The assessment target ARN to attach the template to.
     */
    readonly targetArn: pulumi.Output<string>;
    /**
     * Create a AssessmentTemplate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AssessmentTemplateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AssessmentTemplate resources.
 */
export interface AssessmentTemplateState {
    /**
     * The template assessment ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * The duration of the inspector run.
     */
    duration?: pulumi.Input<number>;
    /**
     * A block that enables sending notifications about a specified assessment template event to a designated SNS topic. See Event Subscriptions for details.
     */
    eventSubscriptions?: pulumi.Input<pulumi.Input<inputs.inspector.AssessmentTemplateEventSubscription>[]>;
    /**
     * The name of the assessment template.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The rules to be used during the run.
     */
    rulesPackageArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value map of tags for the Inspector assessment template. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * The assessment target ARN to attach the template to.
     */
    targetArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AssessmentTemplate resource.
 */
export interface AssessmentTemplateArgs {
    /**
     * The duration of the inspector run.
     */
    duration: pulumi.Input<number>;
    /**
     * A block that enables sending notifications about a specified assessment template event to a designated SNS topic. See Event Subscriptions for details.
     */
    eventSubscriptions?: pulumi.Input<pulumi.Input<inputs.inspector.AssessmentTemplateEventSubscription>[]>;
    /**
     * The name of the assessment template.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The rules to be used during the run.
     */
    rulesPackageArns: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value map of tags for the Inspector assessment template. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The assessment target ARN to attach the template to.
     */
    targetArn: pulumi.Input<string>;
}
