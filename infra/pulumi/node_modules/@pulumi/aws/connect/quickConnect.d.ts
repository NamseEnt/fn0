import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Amazon Connect Quick Connect resource. For more information see
 * [Amazon Connect: Getting Started](https://docs.aws.amazon.com/connect/latest/adminguide/amazon-connect-get-started.html)
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.connect.QuickConnect("test", {
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 *     name: "Example Name",
 *     description: "quick connect phone number",
 *     quickConnectConfig: {
 *         quickConnectType: "PHONE_NUMBER",
 *         phoneConfigs: [{
 *             phoneNumber: "+12345678912",
 *         }],
 *     },
 *     tags: {
 *         Name: "Example Quick Connect",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Amazon Connect Quick Connects using the `instance_id` and `quick_connect_id` separated by a colon (`:`). For example:
 *
 * ```sh
 * $ pulumi import aws:connect/quickConnect:QuickConnect example f1288a1f-6193-445a-b47e-af739b2:c1d4e5f6-1b3c-1b3c-1b3c-c1d4e5f6c1d4e5
 * ```
 */
export declare class QuickConnect extends pulumi.CustomResource {
    /**
     * Get an existing QuickConnect resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: QuickConnectState, opts?: pulumi.CustomResourceOptions): QuickConnect;
    /**
     * Returns true if the given object is an instance of QuickConnect.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is QuickConnect;
    /**
     * The Amazon Resource Name (ARN) of the Quick Connect.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Specifies the description of the Quick Connect.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Specifies the identifier of the hosting Amazon Connect Instance.
     */
    readonly instanceId: pulumi.Output<string>;
    /**
     * Specifies the name of the Quick Connect.
     */
    readonly name: pulumi.Output<string>;
    /**
     * A block that defines the configuration information for the Quick Connect: `quickConnectType` and one of `phoneConfig`, `queueConfig`, `userConfig` . The Quick Connect Config block is documented below.
     */
    readonly quickConnectConfig: pulumi.Output<outputs.connect.QuickConnectQuickConnectConfig>;
    /**
     * The identifier for the Quick Connect.
     */
    readonly quickConnectId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Tags to apply to the Quick Connect. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a QuickConnect resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: QuickConnectArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering QuickConnect resources.
 */
export interface QuickConnectState {
    /**
     * The Amazon Resource Name (ARN) of the Quick Connect.
     */
    arn?: pulumi.Input<string>;
    /**
     * Specifies the description of the Quick Connect.
     */
    description?: pulumi.Input<string>;
    /**
     * Specifies the identifier of the hosting Amazon Connect Instance.
     */
    instanceId?: pulumi.Input<string>;
    /**
     * Specifies the name of the Quick Connect.
     */
    name?: pulumi.Input<string>;
    /**
     * A block that defines the configuration information for the Quick Connect: `quickConnectType` and one of `phoneConfig`, `queueConfig`, `userConfig` . The Quick Connect Config block is documented below.
     */
    quickConnectConfig?: pulumi.Input<inputs.connect.QuickConnectQuickConnectConfig>;
    /**
     * The identifier for the Quick Connect.
     */
    quickConnectId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the Quick Connect. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a QuickConnect resource.
 */
export interface QuickConnectArgs {
    /**
     * Specifies the description of the Quick Connect.
     */
    description?: pulumi.Input<string>;
    /**
     * Specifies the identifier of the hosting Amazon Connect Instance.
     */
    instanceId: pulumi.Input<string>;
    /**
     * Specifies the name of the Quick Connect.
     */
    name?: pulumi.Input<string>;
    /**
     * A block that defines the configuration information for the Quick Connect: `quickConnectType` and one of `phoneConfig`, `queueConfig`, `userConfig` . The Quick Connect Config block is documented below.
     */
    quickConnectConfig: pulumi.Input<inputs.connect.QuickConnectQuickConnectConfig>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the Quick Connect. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
