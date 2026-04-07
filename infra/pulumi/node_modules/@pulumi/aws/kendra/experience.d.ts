import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Kendra Experience.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kendra.Experience("example", {
 *     indexId: exampleAwsKendraIndex.id,
 *     description: "My Kendra Experience",
 *     name: "example",
 *     roleArn: exampleAwsIamRole.arn,
 *     configuration: {
 *         contentSourceConfiguration: {
 *             directPutContent: true,
 *             faqIds: [exampleAwsKendraFaq.faqId],
 *         },
 *         userIdentityConfiguration: {
 *             identityAttributeName: "12345ec453-1546651e-79c4-4554-91fa-00b43ccfa245",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Kendra Experience using the unique identifiers of the experience and index separated by a slash (`/`). For example:
 *
 * ```sh
 * $ pulumi import aws:kendra/experience:Experience example 1045d08d-66ef-4882-b3ed-dfb7df183e90/b34dfdf7-1f2b-4704-9581-79e00296845f
 * ```
 */
export declare class Experience extends pulumi.CustomResource {
    /**
     * Get an existing Experience resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ExperienceState, opts?: pulumi.CustomResourceOptions): Experience;
    /**
     * Returns true if the given object is an instance of Experience.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Experience;
    /**
     * ARN of the Experience.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Configuration information for your Amazon Kendra experience. The provider will only perform drift detection of its value when present in a configuration. Detailed below.
     *
     * > **NOTE:** By default of the AWS Kendra API, updates to an existing `aws.kendra.Experience` resource (e.g. updating the `name`) will also update the `configuration.content_source_configuration.direct_put_content` parameter to `false` if not already provided.
     */
    readonly configuration: pulumi.Output<outputs.kendra.ExperienceConfiguration>;
    /**
     * A description for your Amazon Kendra experience.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Shows the endpoint URLs for your Amazon Kendra experiences. The URLs are unique and fully hosted by AWS.
     */
    readonly endpoints: pulumi.Output<outputs.kendra.ExperienceEndpoint[]>;
    /**
     * The unique identifier of the experience.
     */
    readonly experienceId: pulumi.Output<string>;
    /**
     * The identifier of the index for your Amazon Kendra experience.
     */
    readonly indexId: pulumi.Output<string>;
    /**
     * A name for your Amazon Kendra experience.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of a role with permission to access `Query API`, `QuerySuggestions API`, `SubmitFeedback API`, and `AWS SSO` that stores your user and group information. For more information, see [IAM roles for Amazon Kendra](https://docs.aws.amazon.com/kendra/latest/dg/iam-roles.html).
     *
     * The following arguments are optional:
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * The current processing status of your Amazon Kendra experience.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Create a Experience resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ExperienceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Experience resources.
 */
export interface ExperienceState {
    /**
     * ARN of the Experience.
     */
    arn?: pulumi.Input<string>;
    /**
     * Configuration information for your Amazon Kendra experience. The provider will only perform drift detection of its value when present in a configuration. Detailed below.
     *
     * > **NOTE:** By default of the AWS Kendra API, updates to an existing `aws.kendra.Experience` resource (e.g. updating the `name`) will also update the `configuration.content_source_configuration.direct_put_content` parameter to `false` if not already provided.
     */
    configuration?: pulumi.Input<inputs.kendra.ExperienceConfiguration>;
    /**
     * A description for your Amazon Kendra experience.
     */
    description?: pulumi.Input<string>;
    /**
     * Shows the endpoint URLs for your Amazon Kendra experiences. The URLs are unique and fully hosted by AWS.
     */
    endpoints?: pulumi.Input<pulumi.Input<inputs.kendra.ExperienceEndpoint>[]>;
    /**
     * The unique identifier of the experience.
     */
    experienceId?: pulumi.Input<string>;
    /**
     * The identifier of the index for your Amazon Kendra experience.
     */
    indexId?: pulumi.Input<string>;
    /**
     * A name for your Amazon Kendra experience.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of a role with permission to access `Query API`, `QuerySuggestions API`, `SubmitFeedback API`, and `AWS SSO` that stores your user and group information. For more information, see [IAM roles for Amazon Kendra](https://docs.aws.amazon.com/kendra/latest/dg/iam-roles.html).
     *
     * The following arguments are optional:
     */
    roleArn?: pulumi.Input<string>;
    /**
     * The current processing status of your Amazon Kendra experience.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Experience resource.
 */
export interface ExperienceArgs {
    /**
     * Configuration information for your Amazon Kendra experience. The provider will only perform drift detection of its value when present in a configuration. Detailed below.
     *
     * > **NOTE:** By default of the AWS Kendra API, updates to an existing `aws.kendra.Experience` resource (e.g. updating the `name`) will also update the `configuration.content_source_configuration.direct_put_content` parameter to `false` if not already provided.
     */
    configuration?: pulumi.Input<inputs.kendra.ExperienceConfiguration>;
    /**
     * A description for your Amazon Kendra experience.
     */
    description?: pulumi.Input<string>;
    /**
     * The identifier of the index for your Amazon Kendra experience.
     */
    indexId: pulumi.Input<string>;
    /**
     * A name for your Amazon Kendra experience.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of a role with permission to access `Query API`, `QuerySuggestions API`, `SubmitFeedback API`, and `AWS SSO` that stores your user and group information. For more information, see [IAM roles for Amazon Kendra](https://docs.aws.amazon.com/kendra/latest/dg/iam-roles.html).
     *
     * The following arguments are optional:
     */
    roleArn: pulumi.Input<string>;
}
