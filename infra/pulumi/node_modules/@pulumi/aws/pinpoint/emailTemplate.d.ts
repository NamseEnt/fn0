import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Pinpoint Email Template resource
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.pinpoint.EmailTemplate("test", {
 *     templateName: "testing",
 *     emailTemplates: [{
 *         subject: "testing",
 *         textPart: "we are testing template text part",
 *         headers: [{
 *             name: "testingname",
 *             value: "testingvalue",
 *         }],
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Pinpoint Email Template using the `template_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:pinpoint/emailTemplate:EmailTemplate reset template_name
 * ```
 */
export declare class EmailTemplate extends pulumi.CustomResource {
    /**
     * Get an existing EmailTemplate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EmailTemplateState, opts?: pulumi.CustomResourceOptions): EmailTemplate;
    /**
     * Returns true if the given object is an instance of EmailTemplate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EmailTemplate;
    /**
     * Amazon Resource Name (ARN) of the message template.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Specifies the content and settings for a message template that can be used in messages that are sent through the email channel. See Email Template
     */
    readonly emailTemplates: pulumi.Output<outputs.pinpoint.EmailTemplateEmailTemplate[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * name of the message template. A template name must start with an alphanumeric character and can contain a maximum of 128 characters. The characters can be alphanumeric characters, underscores (_), or hyphens (-). Template names are case sensitive.
     */
    readonly templateName: pulumi.Output<string>;
    /**
     * Create a EmailTemplate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EmailTemplateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EmailTemplate resources.
 */
export interface EmailTemplateState {
    /**
     * Amazon Resource Name (ARN) of the message template.
     */
    arn?: pulumi.Input<string>;
    /**
     * Specifies the content and settings for a message template that can be used in messages that are sent through the email channel. See Email Template
     */
    emailTemplates?: pulumi.Input<pulumi.Input<inputs.pinpoint.EmailTemplateEmailTemplate>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * name of the message template. A template name must start with an alphanumeric character and can contain a maximum of 128 characters. The characters can be alphanumeric characters, underscores (_), or hyphens (-). Template names are case sensitive.
     */
    templateName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EmailTemplate resource.
 */
export interface EmailTemplateArgs {
    /**
     * Specifies the content and settings for a message template that can be used in messages that are sent through the email channel. See Email Template
     */
    emailTemplates?: pulumi.Input<pulumi.Input<inputs.pinpoint.EmailTemplateEmailTemplate>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * name of the message template. A template name must start with an alphanumeric character and can contain a maximum of 128 characters. The characters can be alphanumeric characters, underscores (_), or hyphens (-). Template names are case sensitive.
     */
    templateName: pulumi.Input<string>;
}
