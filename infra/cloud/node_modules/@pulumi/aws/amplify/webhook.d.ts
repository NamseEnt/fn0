import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Amplify Webhook resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.amplify.App("example", {name: "app"});
 * const master = new aws.amplify.Branch("master", {
 *     appId: example.id,
 *     branchName: "master",
 * });
 * const masterWebhook = new aws.amplify.Webhook("master", {
 *     appId: example.id,
 *     branchName: master.branchName,
 *     description: "triggermaster",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Amplify webhook using a webhook ID. For example:
 *
 * ```sh
 * $ pulumi import aws:amplify/webhook:Webhook master a26b22a0-748b-4b57-b9a0-ae7e601fe4b1
 * ```
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
     * Unique ID for an Amplify app.
     */
    readonly appId: pulumi.Output<string>;
    /**
     * ARN for the webhook.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Name for a branch that is part of the Amplify app.
     */
    readonly branchName: pulumi.Output<string>;
    /**
     * Description for a webhook.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * URL of the webhook.
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
     * Unique ID for an Amplify app.
     */
    appId?: pulumi.Input<string>;
    /**
     * ARN for the webhook.
     */
    arn?: pulumi.Input<string>;
    /**
     * Name for a branch that is part of the Amplify app.
     */
    branchName?: pulumi.Input<string>;
    /**
     * Description for a webhook.
     */
    description?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * URL of the webhook.
     */
    url?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Webhook resource.
 */
export interface WebhookArgs {
    /**
     * Unique ID for an Amplify app.
     */
    appId: pulumi.Input<string>;
    /**
     * Name for a branch that is part of the Amplify app.
     */
    branchName: pulumi.Input<string>;
    /**
     * Description for a webhook.
     */
    description?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
