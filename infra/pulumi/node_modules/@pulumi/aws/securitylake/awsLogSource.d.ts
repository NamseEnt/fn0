import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an Amazon Security Lake AWS Log Source.
 *
 * > **NOTE:** A single `aws.securitylake.AwsLogSource` should be used to configure a log source across all regions and accounts.
 *
 * > **NOTE:** The underlying `aws.securitylake.DataLake` must be configured before creating the `aws.securitylake.AwsLogSource`. Use a `dependsOn` statement.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.securitylake.AwsLogSource("example", {source: {
 *     accounts: ["123456789012"],
 *     regions: ["eu-west-1"],
 *     sourceName: "ROUTE53",
 * }}, {
 *     dependsOn: [exampleAwsSecuritylakeDataLake],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AWS log sources using the source name. For example:
 *
 * ```sh
 * $ pulumi import aws:securitylake/awsLogSource:AwsLogSource example ROUTE53
 * ```
 */
export declare class AwsLogSource extends pulumi.CustomResource {
    /**
     * Get an existing AwsLogSource resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AwsLogSourceState, opts?: pulumi.CustomResourceOptions): AwsLogSource;
    /**
     * Returns true if the given object is an instance of AwsLogSource.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AwsLogSource;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specify the natively-supported AWS service to add as a source in Security Lake.
     */
    readonly source: pulumi.Output<outputs.securitylake.AwsLogSourceSource | undefined>;
    /**
     * Create a AwsLogSource resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: AwsLogSourceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AwsLogSource resources.
 */
export interface AwsLogSourceState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specify the natively-supported AWS service to add as a source in Security Lake.
     */
    source?: pulumi.Input<inputs.securitylake.AwsLogSourceSource>;
}
/**
 * The set of arguments for constructing a AwsLogSource resource.
 */
export interface AwsLogSourceArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specify the natively-supported AWS service to add as a source in Security Lake.
     */
    source?: pulumi.Input<inputs.securitylake.AwsLogSourceSource>;
}
