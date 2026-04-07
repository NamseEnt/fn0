import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Amazon AppIntegrations Data Integration resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appintegrations.DataIntegration("example", {
 *     name: "example",
 *     description: "example",
 *     kmsKey: test.arn,
 *     sourceUri: "Salesforce://AppFlow/example",
 *     scheduleConfig: {
 *         firstExecutionFrom: "1439788442681",
 *         object: "Account",
 *         scheduleExpression: "rate(1 hour)",
 *     },
 *     tags: {
 *         Key1: "Value1",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Amazon AppIntegrations Data Integrations using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:appintegrations/dataIntegration:DataIntegration example 12345678-1234-1234-1234-123456789123
 * ```
 */
export declare class DataIntegration extends pulumi.CustomResource {
    /**
     * Get an existing DataIntegration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DataIntegrationState, opts?: pulumi.CustomResourceOptions): DataIntegration;
    /**
     * Returns true if the given object is an instance of DataIntegration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DataIntegration;
    /**
     * The Amazon Resource Name (ARN) of the Data Integration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Specifies the description of the Data Integration.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Specifies the KMS key Amazon Resource Name (ARN) for the Data Integration.
     */
    readonly kmsKey: pulumi.Output<string>;
    /**
     * Specifies the name of the Data Integration.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A block that defines the name of the data and how often it should be pulled from the source. The Schedule Config block is documented below.
     */
    readonly scheduleConfig: pulumi.Output<outputs.appintegrations.DataIntegrationScheduleConfig>;
    /**
     * Specifies the URI of the data source. Create an AppFlow Connector Profile and reference the name of the profile in the URL. An example of this value for Salesforce is `Salesforce://AppFlow/example` where `example` is the name of the AppFlow Connector Profile.
     */
    readonly sourceUri: pulumi.Output<string>;
    /**
     * Tags to apply to the Data Integration. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a DataIntegration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DataIntegrationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DataIntegration resources.
 */
export interface DataIntegrationState {
    /**
     * The Amazon Resource Name (ARN) of the Data Integration.
     */
    arn?: pulumi.Input<string>;
    /**
     * Specifies the description of the Data Integration.
     */
    description?: pulumi.Input<string>;
    /**
     * Specifies the KMS key Amazon Resource Name (ARN) for the Data Integration.
     */
    kmsKey?: pulumi.Input<string>;
    /**
     * Specifies the name of the Data Integration.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A block that defines the name of the data and how often it should be pulled from the source. The Schedule Config block is documented below.
     */
    scheduleConfig?: pulumi.Input<inputs.appintegrations.DataIntegrationScheduleConfig>;
    /**
     * Specifies the URI of the data source. Create an AppFlow Connector Profile and reference the name of the profile in the URL. An example of this value for Salesforce is `Salesforce://AppFlow/example` where `example` is the name of the AppFlow Connector Profile.
     */
    sourceUri?: pulumi.Input<string>;
    /**
     * Tags to apply to the Data Integration. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a DataIntegration resource.
 */
export interface DataIntegrationArgs {
    /**
     * Specifies the description of the Data Integration.
     */
    description?: pulumi.Input<string>;
    /**
     * Specifies the KMS key Amazon Resource Name (ARN) for the Data Integration.
     */
    kmsKey: pulumi.Input<string>;
    /**
     * Specifies the name of the Data Integration.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A block that defines the name of the data and how often it should be pulled from the source. The Schedule Config block is documented below.
     */
    scheduleConfig: pulumi.Input<inputs.appintegrations.DataIntegrationScheduleConfig>;
    /**
     * Specifies the URI of the data source. Create an AppFlow Connector Profile and reference the name of the profile in the URL. An example of this value for Salesforce is `Salesforce://AppFlow/example` where `example` is the name of the AppFlow Connector Profile.
     */
    sourceUri: pulumi.Input<string>;
    /**
     * Tags to apply to the Data Integration. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
