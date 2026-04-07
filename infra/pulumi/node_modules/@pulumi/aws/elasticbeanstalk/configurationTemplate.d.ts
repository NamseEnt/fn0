import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Elastic Beanstalk Configuration Template, which are associated with
 * a specific application and are used to deploy different versions of the
 * application with the same configuration settings.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleApplication = new aws.elasticbeanstalk.Application("example", {
 *     name: "tf-test-name",
 *     description: "tf-test-desc",
 * });
 * const example = new aws.elasticbeanstalk.ConfigurationTemplate("example", {
 *     name: "tf-test-template-config",
 *     application: exampleApplication.name,
 *     solutionStackName: "64bit Amazon Linux 2015.09 v2.0.8 running Go 1.4",
 * });
 * ```
 *
 * ## Option Settings
 *
 * The `setting` field supports the following format:
 *
 * * `namespace` - (Required) Unique namespace identifying the option's associated AWS resource
 * * `name` - (Required) Name of the configuration option
 * * `value` - (Required) Value for the configuration option
 * * `resource` - (Optional) resource name for [scheduled action](https://docs.aws.amazon.com/elasticbeanstalk/latest/dg/command-options-general.html#command-options-general-autoscalingscheduledaction)
 */
export declare class ConfigurationTemplate extends pulumi.CustomResource {
    /**
     * Get an existing ConfigurationTemplate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConfigurationTemplateState, opts?: pulumi.CustomResourceOptions): ConfigurationTemplate;
    /**
     * Returns true if the given object is an instance of ConfigurationTemplate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ConfigurationTemplate;
    /**
     * name of the application to associate with this configuration template
     */
    readonly application: pulumi.Output<string>;
    /**
     * Short description of the Template
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The ID of the environment used with this configuration template
     */
    readonly environmentId: pulumi.Output<string | undefined>;
    /**
     * A unique name for this Template.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Option settings to configure the new Environment. These
     * override specific values that are set as defaults. The format is detailed
     * below in Option Settings
     */
    readonly settings: pulumi.Output<outputs.elasticbeanstalk.ConfigurationTemplateSetting[]>;
    /**
     * A solution stack to base your Template
     * off of. Example stacks can be found in the [Amazon API documentation][1]
     */
    readonly solutionStackName: pulumi.Output<string | undefined>;
    /**
     * Create a ConfigurationTemplate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConfigurationTemplateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ConfigurationTemplate resources.
 */
export interface ConfigurationTemplateState {
    /**
     * name of the application to associate with this configuration template
     */
    application?: pulumi.Input<string>;
    /**
     * Short description of the Template
     */
    description?: pulumi.Input<string>;
    /**
     * The ID of the environment used with this configuration template
     */
    environmentId?: pulumi.Input<string>;
    /**
     * A unique name for this Template.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Option settings to configure the new Environment. These
     * override specific values that are set as defaults. The format is detailed
     * below in Option Settings
     */
    settings?: pulumi.Input<pulumi.Input<inputs.elasticbeanstalk.ConfigurationTemplateSetting>[]>;
    /**
     * A solution stack to base your Template
     * off of. Example stacks can be found in the [Amazon API documentation][1]
     */
    solutionStackName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ConfigurationTemplate resource.
 */
export interface ConfigurationTemplateArgs {
    /**
     * name of the application to associate with this configuration template
     */
    application: pulumi.Input<string>;
    /**
     * Short description of the Template
     */
    description?: pulumi.Input<string>;
    /**
     * The ID of the environment used with this configuration template
     */
    environmentId?: pulumi.Input<string>;
    /**
     * A unique name for this Template.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Option settings to configure the new Environment. These
     * override specific values that are set as defaults. The format is detailed
     * below in Option Settings
     */
    settings?: pulumi.Input<pulumi.Input<inputs.elasticbeanstalk.ConfigurationTemplateSetting>[]>;
    /**
     * A solution stack to base your Template
     * off of. Example stacks can be found in the [Amazon API documentation][1]
     */
    solutionStackName?: pulumi.Input<string>;
}
