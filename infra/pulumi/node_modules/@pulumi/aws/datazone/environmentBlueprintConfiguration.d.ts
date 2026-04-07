import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS DataZone Environment Blueprint Configuration.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.datazone.Domain("example", {
 *     name: "example_domain",
 *     domainExecutionRole: domainExecutionRole.arn,
 * });
 * const defaultDataLake = aws.datazone.getEnvironmentBlueprintOutput({
 *     domainId: example.id,
 *     name: "DefaultDataLake",
 *     managed: true,
 * });
 * const exampleEnvironmentBlueprintConfiguration = new aws.datazone.EnvironmentBlueprintConfiguration("example", {
 *     domainId: example.id,
 *     environmentBlueprintId: defaultDataLake.apply(defaultDataLake => defaultDataLake.id),
 *     enabledRegions: ["us-east-1"],
 *     regionalParameters: {
 *         "us-east-1": {
 *             s3Location: "s3://my-amazon-datazone-bucket",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DataZone Environment Blueprint Configuration using the `domain_id` and `environment_blueprint_id`, separated by a `/`. For example:
 *
 * ```sh
 * $ pulumi import aws:datazone/environmentBlueprintConfiguration:EnvironmentBlueprintConfiguration example domain-id-12345/environment-blueprint-id-54321
 * ```
 */
export declare class EnvironmentBlueprintConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing EnvironmentBlueprintConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EnvironmentBlueprintConfigurationState, opts?: pulumi.CustomResourceOptions): EnvironmentBlueprintConfiguration;
    /**
     * Returns true if the given object is an instance of EnvironmentBlueprintConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EnvironmentBlueprintConfiguration;
    /**
     * ID of the Domain.
     */
    readonly domainId: pulumi.Output<string>;
    /**
     * Regions in which the blueprint is enabled
     *
     * The following arguments are optional:
     */
    readonly enabledRegions: pulumi.Output<string[]>;
    /**
     * ID of the Environment Blueprint
     */
    readonly environmentBlueprintId: pulumi.Output<string>;
    /**
     * ARN of the manage access role with which this blueprint is created.
     */
    readonly manageAccessRoleArn: pulumi.Output<string | undefined>;
    /**
     * ARN of the provisioning role with which this blueprint is created.
     */
    readonly provisioningRoleArn: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Parameters for each region in which the blueprint is enabled
     */
    readonly regionalParameters: pulumi.Output<{
        [key: string]: {
            [key: string]: string;
        };
    } | undefined>;
    /**
     * Create a EnvironmentBlueprintConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EnvironmentBlueprintConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EnvironmentBlueprintConfiguration resources.
 */
export interface EnvironmentBlueprintConfigurationState {
    /**
     * ID of the Domain.
     */
    domainId?: pulumi.Input<string>;
    /**
     * Regions in which the blueprint is enabled
     *
     * The following arguments are optional:
     */
    enabledRegions?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ID of the Environment Blueprint
     */
    environmentBlueprintId?: pulumi.Input<string>;
    /**
     * ARN of the manage access role with which this blueprint is created.
     */
    manageAccessRoleArn?: pulumi.Input<string>;
    /**
     * ARN of the provisioning role with which this blueprint is created.
     */
    provisioningRoleArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Parameters for each region in which the blueprint is enabled
     */
    regionalParameters?: pulumi.Input<{
        [key: string]: pulumi.Input<{
            [key: string]: pulumi.Input<string>;
        }>;
    }>;
}
/**
 * The set of arguments for constructing a EnvironmentBlueprintConfiguration resource.
 */
export interface EnvironmentBlueprintConfigurationArgs {
    /**
     * ID of the Domain.
     */
    domainId: pulumi.Input<string>;
    /**
     * Regions in which the blueprint is enabled
     *
     * The following arguments are optional:
     */
    enabledRegions: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ID of the Environment Blueprint
     */
    environmentBlueprintId: pulumi.Input<string>;
    /**
     * ARN of the manage access role with which this blueprint is created.
     */
    manageAccessRoleArn?: pulumi.Input<string>;
    /**
     * ARN of the provisioning role with which this blueprint is created.
     */
    provisioningRoleArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Parameters for each region in which the blueprint is enabled
     */
    regionalParameters?: pulumi.Input<{
        [key: string]: pulumi.Input<{
            [key: string]: pulumi.Input<string>;
        }>;
    }>;
}
