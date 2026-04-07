import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a CodePipeline Webhook.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as github from "@pulumi/github";
 *
 * const bar = new aws.codepipeline.Pipeline("bar", {
 *     name: "tf-test-pipeline",
 *     roleArn: barAwsIamRole.arn,
 *     artifactStores: [{
 *         location: barAwsS3Bucket.bucket,
 *         type: "S3",
 *         encryptionKey: {
 *             id: s3kmskey.arn,
 *             type: "KMS",
 *         },
 *     }],
 *     stages: [
 *         {
 *             name: "Source",
 *             actions: [{
 *                 name: "Source",
 *                 category: "Source",
 *                 owner: "ThirdParty",
 *                 provider: "GitHub",
 *                 version: "1",
 *                 outputArtifacts: ["test"],
 *                 configuration: {
 *                     Owner: "my-organization",
 *                     Repo: "test",
 *                     Branch: "master",
 *                 },
 *             }],
 *         },
 *         {
 *             name: "Build",
 *             actions: [{
 *                 name: "Build",
 *                 category: "Build",
 *                 owner: "AWS",
 *                 provider: "CodeBuild",
 *                 inputArtifacts: ["test"],
 *                 version: "1",
 *                 configuration: {
 *                     ProjectName: "test",
 *                 },
 *             }],
 *         },
 *     ],
 * });
 * const webhookSecret = "super-secret";
 * const barWebhook = new aws.codepipeline.Webhook("bar", {
 *     name: "test-webhook-github-bar",
 *     authentication: "GITHUB_HMAC",
 *     targetAction: "Source",
 *     targetPipeline: bar.name,
 *     authenticationConfiguration: {
 *         secretToken: webhookSecret,
 *     },
 *     filters: [{
 *         jsonPath: "$.ref",
 *         matchEquals: "refs/heads/{Branch}",
 *     }],
 * });
 * // Wire the CodePipeline webhook into a GitHub repository.
 * const barRepositoryWebhook = new github.RepositoryWebhook("bar", {
 *     repository: repo.name,
 *     name: "web",
 *     configuration: [{
 *         url: barWebhook.url,
 *         contentType: "json",
 *         insecureSsl: true,
 *         secret: webhookSecret,
 *     }],
 *     events: ["push"],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the CodePipeline webhook.
 *
 * Using `pulumi import`, import CodePipeline Webhooks using their ARN. For example:
 *
 * console
 *
 * % pulumi import aws_codepipeline_webhook.example arn:aws:codepipeline:us-west-2:123456789012:webhook:example
 */
export declare class Webhook extends pulumi.CustomResource {
    /**
     * Get an existing Webhook resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WebhookState, opts?: pulumi.CustomResourceOptions): Webhook;
    /**
     * Returns true if the given object is an instance of Webhook.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Webhook;
    /**
     * The CodePipeline webhook's ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The type of authentication  to use. One of `IP`, `GITHUB_HMAC`, or `UNAUTHENTICATED`.
     */
    readonly authentication: pulumi.Output<string>;
    /**
     * An `auth` block. Required for `IP` and `GITHUB_HMAC`. Auth blocks are documented below.
     */
    readonly authenticationConfiguration: pulumi.Output<outputs.codepipeline.WebhookAuthenticationConfiguration | undefined>;
    /**
     * One or more `filter` blocks. Filter blocks are documented below.
     */
    readonly filters: pulumi.Output<outputs.codepipeline.WebhookFilter[]>;
    /**
     * The name of the webhook.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The name of the action in a pipeline you want to connect to the webhook. The action must be from the source (first) stage of the pipeline.
     */
    readonly targetAction: pulumi.Output<string>;
    /**
     * The name of the pipeline.
     */
    readonly targetPipeline: pulumi.Output<string>;
    /**
     * The CodePipeline webhook's URL. POST events to this endpoint to trigger the target.
     */
    readonly url: pulumi.Output<string>;
    /**
     * Create a Webhook resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: WebhookArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Webhook resources.
 */
export interface WebhookState {
    /**
     * The CodePipeline webhook's ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * The type of authentication  to use. One of `IP`, `GITHUB_HMAC`, or `UNAUTHENTICATED`.
     */
    authentication?: pulumi.Input<string>;
    /**
     * An `auth` block. Required for `IP` and `GITHUB_HMAC`. Auth blocks are documented below.
     */
    authenticationConfiguration?: pulumi.Input<inputs.codepipeline.WebhookAuthenticationConfiguration>;
    /**
     * One or more `filter` blocks. Filter blocks are documented below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.codepipeline.WebhookFilter>[]>;
    /**
     * The name of the webhook.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The name of the action in a pipeline you want to connect to the webhook. The action must be from the source (first) stage of the pipeline.
     */
    targetAction?: pulumi.Input<string>;
    /**
     * The name of the pipeline.
     */
    targetPipeline?: pulumi.Input<string>;
    /**
     * The CodePipeline webhook's URL. POST events to this endpoint to trigger the target.
     */
    url?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Webhook resource.
 */
export interface WebhookArgs {
    /**
     * The type of authentication  to use. One of `IP`, `GITHUB_HMAC`, or `UNAUTHENTICATED`.
     */
    authentication: pulumi.Input<string>;
    /**
     * An `auth` block. Required for `IP` and `GITHUB_HMAC`. Auth blocks are documented below.
     */
    authenticationConfiguration?: pulumi.Input<inputs.codepipeline.WebhookAuthenticationConfiguration>;
    /**
     * One or more `filter` blocks. Filter blocks are documented below.
     */
    filters: pulumi.Input<pulumi.Input<inputs.codepipeline.WebhookFilter>[]>;
    /**
     * The name of the webhook.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The name of the action in a pipeline you want to connect to the webhook. The action must be from the source (first) stage of the pipeline.
     */
    targetAction: pulumi.Input<string>;
    /**
     * The name of the pipeline.
     */
    targetPipeline: pulumi.Input<string>;
}
