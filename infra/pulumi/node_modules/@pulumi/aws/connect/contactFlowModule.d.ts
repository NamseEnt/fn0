import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Amazon Connect Contact Flow Module resource. For more information see
 * [Amazon Connect: Getting Started](https://docs.aws.amazon.com/connect/latest/adminguide/amazon-connect-get-started.html)
 *
 * This resource embeds or references Contact Flows Modules specified in Amazon Connect Contact Flow Language. For more information see
 * [Amazon Connect Flow language](https://docs.aws.amazon.com/connect/latest/adminguide/flow-language.html)
 *
 * !> **WARN:** Contact Flow Modules exported from the Console [See Contact Flow import/export which is the same for Contact Flow Modules](https://docs.aws.amazon.com/connect/latest/adminguide/contact-flow-import-export.html) are not in the Amazon Connect Contact Flow Language and can not be used with this resource. Instead, the recommendation is to use the AWS CLI [`describe-contact-flow-module`](https://docs.aws.amazon.com/cli/latest/reference/connect/describe-contact-flow-module.html).
 * See example below which uses `jq` to extract the `Content` attribute and saves it to a local file.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.connect.ContactFlowModule("example", {
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     name: "Example",
 *     description: "Example Contact Flow Module Description",
 *     content: JSON.stringify({
 *         Version: "2019-10-30",
 *         StartAction: "12345678-1234-1234-1234-123456789012",
 *         Actions: [
 *             {
 *                 Identifier: "12345678-1234-1234-1234-123456789012",
 *                 Parameters: {
 *                     Text: "Hello contact flow module",
 *                 },
 *                 Transitions: {
 *                     NextAction: "abcdef-abcd-abcd-abcd-abcdefghijkl",
 *                     Errors: [],
 *                     Conditions: [],
 *                 },
 *                 Type: "MessageParticipant",
 *             },
 *             {
 *                 Identifier: "abcdef-abcd-abcd-abcd-abcdefghijkl",
 *                 Type: "DisconnectParticipant",
 *                 Parameters: {},
 *                 Transitions: {},
 *             },
 *         ],
 *         Settings: {
 *             InputParameters: [],
 *             OutputParameters: [],
 *             Transitions: [
 *                 {
 *                     DisplayName: "Success",
 *                     ReferenceName: "Success",
 *                     Description: "",
 *                 },
 *                 {
 *                     DisplayName: "Error",
 *                     ReferenceName: "Error",
 *                     Description: "",
 *                 },
 *             ],
 *         },
 *     }),
 *     tags: {
 *         Name: "Example Contact Flow Module",
 *         Application: "Example",
 *         Method: "Create",
 *     },
 * });
 * ```
 *
 * ### With External Content
 *
 * Use the AWS CLI to extract Contact Flow Content:
 *
 * Use the generated file as input:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.connect.ContactFlowModule("example", {
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     name: "Example",
 *     description: "Example Contact Flow Module Description",
 *     filename: "contact_flow_module.json",
 *     contentHash: std.filebase64sha256({
 *         input: "contact_flow_module.json",
 *     }).then(invoke => invoke.result),
 *     tags: {
 *         Name: "Example Contact Flow Module",
 *         Application: "Example",
 *         Method: "Create",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Amazon Connect Contact Flow Modules using the `instance_id` and `contact_flow_module_id` separated by a colon (`:`). For example:
 *
 * ```sh
 * $ pulumi import aws:connect/contactFlowModule:ContactFlowModule example f1288a1f-6193-445a-b47e-af739b2:c1d4e5f6-1b3c-1b3c-1b3c-c1d4e5f6c1d4e5
 * ```
 */
export declare class ContactFlowModule extends pulumi.CustomResource {
    /**
     * Get an existing ContactFlowModule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ContactFlowModuleState, opts?: pulumi.CustomResourceOptions): ContactFlowModule;
    /**
     * Returns true if the given object is an instance of ContactFlowModule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ContactFlowModule;
    /**
     * The Amazon Resource Name (ARN) of the Contact Flow Module.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The identifier of the Contact Flow Module.
     */
    readonly contactFlowModuleId: pulumi.Output<string>;
    /**
     * Specifies the content of the Contact Flow Module, provided as a JSON string, written in Amazon Connect Contact Flow Language. If defined, the `filename` argument cannot be used.
     */
    readonly content: pulumi.Output<string>;
    /**
     * Used to trigger updates. Must be set to a base64-encoded SHA256 hash of the Contact Flow Module source specified with `filename`.
     */
    readonly contentHash: pulumi.Output<string | undefined>;
    /**
     * Specifies the description of the Contact Flow Module.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The path to the Contact Flow Module source within the local filesystem. Conflicts with `content`.
     */
    readonly filename: pulumi.Output<string | undefined>;
    /**
     * Specifies the identifier of the hosting Amazon Connect Instance.
     */
    readonly instanceId: pulumi.Output<string>;
    /**
     * Specifies the name of the Contact Flow Module.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Tags to apply to the Contact Flow Module. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a ContactFlowModule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ContactFlowModuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ContactFlowModule resources.
 */
export interface ContactFlowModuleState {
    /**
     * The Amazon Resource Name (ARN) of the Contact Flow Module.
     */
    arn?: pulumi.Input<string>;
    /**
     * The identifier of the Contact Flow Module.
     */
    contactFlowModuleId?: pulumi.Input<string>;
    /**
     * Specifies the content of the Contact Flow Module, provided as a JSON string, written in Amazon Connect Contact Flow Language. If defined, the `filename` argument cannot be used.
     */
    content?: pulumi.Input<string>;
    /**
     * Used to trigger updates. Must be set to a base64-encoded SHA256 hash of the Contact Flow Module source specified with `filename`.
     */
    contentHash?: pulumi.Input<string>;
    /**
     * Specifies the description of the Contact Flow Module.
     */
    description?: pulumi.Input<string>;
    /**
     * The path to the Contact Flow Module source within the local filesystem. Conflicts with `content`.
     */
    filename?: pulumi.Input<string>;
    /**
     * Specifies the identifier of the hosting Amazon Connect Instance.
     */
    instanceId?: pulumi.Input<string>;
    /**
     * Specifies the name of the Contact Flow Module.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the Contact Flow Module. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a ContactFlowModule resource.
 */
export interface ContactFlowModuleArgs {
    /**
     * Specifies the content of the Contact Flow Module, provided as a JSON string, written in Amazon Connect Contact Flow Language. If defined, the `filename` argument cannot be used.
     */
    content?: pulumi.Input<string>;
    /**
     * Used to trigger updates. Must be set to a base64-encoded SHA256 hash of the Contact Flow Module source specified with `filename`.
     */
    contentHash?: pulumi.Input<string>;
    /**
     * Specifies the description of the Contact Flow Module.
     */
    description?: pulumi.Input<string>;
    /**
     * The path to the Contact Flow Module source within the local filesystem. Conflicts with `content`.
     */
    filename?: pulumi.Input<string>;
    /**
     * Specifies the identifier of the hosting Amazon Connect Instance.
     */
    instanceId: pulumi.Input<string>;
    /**
     * Specifies the name of the Contact Flow Module.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the Contact Flow Module. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
