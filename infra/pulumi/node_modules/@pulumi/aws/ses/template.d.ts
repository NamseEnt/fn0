import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to create a SES template.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const myTemplate = new aws.ses.Template("MyTemplate", {
 *     name: "MyTemplate",
 *     subject: "Greetings, {{name}}!",
 *     html: "<h1>Hello {{name}},</h1><p>Your favorite animal is {{favoriteanimal}}.</p>",
 *     text: `Hello {{name}},\x0d
 * Your favorite animal is {{favoriteanimal}}.`,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SES templates using the template name. For example:
 *
 * ```sh
 * $ pulumi import aws:ses/template:Template MyTemplate MyTemplate
 * ```
 */
export declare class Template extends pulumi.CustomResource {
    /**
     * Get an existing Template resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TemplateState, opts?: pulumi.CustomResourceOptions): Template;
    /**
     * Returns true if the given object is an instance of Template.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Template;
    /**
     * The ARN of the SES template
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The HTML body of the email. Must be less than 500KB in size, including both the text and HTML parts.
     */
    readonly html: pulumi.Output<string | undefined>;
    /**
     * The name of the template. Cannot exceed 64 characters. You will refer to this name when you send email.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The subject line of the email.
     */
    readonly subject: pulumi.Output<string | undefined>;
    /**
     * The email body that will be visible to recipients whose email clients do not display HTML. Must be less than 500KB in size, including both the text and HTML parts.
     */
    readonly text: pulumi.Output<string | undefined>;
    /**
     * Create a Template resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: TemplateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Template resources.
 */
export interface TemplateState {
    /**
     * The ARN of the SES template
     */
    arn?: pulumi.Input<string>;
    /**
     * The HTML body of the email. Must be less than 500KB in size, including both the text and HTML parts.
     */
    html?: pulumi.Input<string>;
    /**
     * The name of the template. Cannot exceed 64 characters. You will refer to this name when you send email.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The subject line of the email.
     */
    subject?: pulumi.Input<string>;
    /**
     * The email body that will be visible to recipients whose email clients do not display HTML. Must be less than 500KB in size, including both the text and HTML parts.
     */
    text?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Template resource.
 */
export interface TemplateArgs {
    /**
     * The HTML body of the email. Must be less than 500KB in size, including both the text and HTML parts.
     */
    html?: pulumi.Input<string>;
    /**
     * The name of the template. Cannot exceed 64 characters. You will refer to this name when you send email.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The subject line of the email.
     */
    subject?: pulumi.Input<string>;
    /**
     * The email body that will be visible to recipients whose email clients do not display HTML. Must be less than 500KB in size, including both the text and HTML parts.
     */
    text?: pulumi.Input<string>;
}
