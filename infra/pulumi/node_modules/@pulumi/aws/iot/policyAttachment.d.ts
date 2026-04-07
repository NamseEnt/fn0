import * as pulumi from "@pulumi/pulumi";
import { Policy } from "./index";
/**
 * Provides an IoT policy attachment.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const pubsub = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         actions: ["iot:*"],
 *         resources: ["*"],
 *     }],
 * });
 * const pubsubPolicy = new aws.iot.Policy("pubsub", {
 *     name: "PubSubToAnyTopic",
 *     policy: pubsub.then(pubsub => pubsub.json),
 * });
 * const cert = new aws.iot.Certificate("cert", {
 *     csr: std.file({
 *         input: "csr.pem",
 *     }).then(invoke => invoke.result),
 *     active: true,
 * });
 * const att = new aws.iot.PolicyAttachment("att", {
 *     policy: pubsubPolicy.name,
 *     target: cert.arn,
 * });
 * ```
 */
export declare class PolicyAttachment extends pulumi.CustomResource {
    /**
     * Get an existing PolicyAttachment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PolicyAttachmentState, opts?: pulumi.CustomResourceOptions): PolicyAttachment;
    /**
     * Returns true if the given object is an instance of PolicyAttachment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PolicyAttachment;
    /**
     * The name of the policy to attach.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The identity to which the policy is attached.
     */
    readonly target: pulumi.Output<string>;
    /**
     * Create a PolicyAttachment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PolicyAttachmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PolicyAttachment resources.
 */
export interface PolicyAttachmentState {
    /**
     * The name of the policy to attach.
     */
    policy?: pulumi.Input<string | Policy>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The identity to which the policy is attached.
     */
    target?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PolicyAttachment resource.
 */
export interface PolicyAttachmentArgs {
    /**
     * The name of the policy to attach.
     */
    policy: pulumi.Input<string | Policy>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The identity to which the policy is attached.
     */
    target: pulumi.Input<string>;
}
