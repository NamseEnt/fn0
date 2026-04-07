import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Security Lake Custom Log Source.
 *
 * > **NOTE:** The underlying `aws.securitylake.DataLake` must be configured before creating the `aws.securitylake.CustomLogSource`. Use a `dependsOn` statement.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.securitylake.CustomLogSource("example", {
 *     sourceName: "example-name",
 *     sourceVersion: "1.0",
 *     eventClasses: ["FILE_ACTIVITY"],
 *     configuration: {
 *         crawlerConfiguration: {
 *             roleArn: customLog.arn,
 *         },
 *         providerIdentity: {
 *             externalId: "example-id",
 *             principal: "123456789012",
 *         },
 *     },
 * }, {
 *     dependsOn: [exampleAwsSecuritylakeDataLake],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Custom log sources using the source name. For example:
 *
 * ```sh
 * $ pulumi import aws:securitylake/customLogSource:CustomLogSource example example-name
 * ```
 */
export declare class CustomLogSource extends pulumi.CustomResource {
    /**
     * Get an existing CustomLogSource resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CustomLogSourceState, opts?: pulumi.CustomResourceOptions): CustomLogSource;
    /**
     * Returns true if the given object is an instance of CustomLogSource.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CustomLogSource;
    /**
     * The attributes of a third-party custom source.
     */
    readonly attributes: pulumi.Output<outputs.securitylake.CustomLogSourceAttribute[]>;
    /**
     * The configuration for the third-party custom source.
     */
    readonly configuration: pulumi.Output<outputs.securitylake.CustomLogSourceConfiguration | undefined>;
    /**
     * The Open Cybersecurity Schema Framework (OCSF) event classes which describes the type of data that the custom source will send to Security Lake.
     */
    readonly eventClasses: pulumi.Output<string[] | undefined>;
    /**
     * The details of the log provider for a third-party custom source.
     */
    readonly providerDetails: pulumi.Output<outputs.securitylake.CustomLogSourceProviderDetail[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specify the name for a third-party custom source.
     * This must be a Regionally unique value.
     * Has a maximum length of 20.
     */
    readonly sourceName: pulumi.Output<string>;
    /**
     * Specify the source version for the third-party custom source, to limit log collection to a specific version of custom data source.
     */
    readonly sourceVersion: pulumi.Output<string>;
    /**
     * Create a CustomLogSource resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CustomLogSourceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CustomLogSource resources.
 */
export interface CustomLogSourceState {
    /**
     * The attributes of a third-party custom source.
     */
    attributes?: pulumi.Input<pulumi.Input<inputs.securitylake.CustomLogSourceAttribute>[]>;
    /**
     * The configuration for the third-party custom source.
     */
    configuration?: pulumi.Input<inputs.securitylake.CustomLogSourceConfiguration>;
    /**
     * The Open Cybersecurity Schema Framework (OCSF) event classes which describes the type of data that the custom source will send to Security Lake.
     */
    eventClasses?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The details of the log provider for a third-party custom source.
     */
    providerDetails?: pulumi.Input<pulumi.Input<inputs.securitylake.CustomLogSourceProviderDetail>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specify the name for a third-party custom source.
     * This must be a Regionally unique value.
     * Has a maximum length of 20.
     */
    sourceName?: pulumi.Input<string>;
    /**
     * Specify the source version for the third-party custom source, to limit log collection to a specific version of custom data source.
     */
    sourceVersion?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a CustomLogSource resource.
 */
export interface CustomLogSourceArgs {
    /**
     * The configuration for the third-party custom source.
     */
    configuration?: pulumi.Input<inputs.securitylake.CustomLogSourceConfiguration>;
    /**
     * The Open Cybersecurity Schema Framework (OCSF) event classes which describes the type of data that the custom source will send to Security Lake.
     */
    eventClasses?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specify the name for a third-party custom source.
     * This must be a Regionally unique value.
     * Has a maximum length of 20.
     */
    sourceName: pulumi.Input<string>;
    /**
     * Specify the source version for the third-party custom source, to limit log collection to a specific version of custom data source.
     */
    sourceVersion?: pulumi.Input<string>;
}
