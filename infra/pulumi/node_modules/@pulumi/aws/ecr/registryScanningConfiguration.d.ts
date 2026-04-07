import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Elastic Container Registry Scanning Configuration. Can't be completely deleted, instead reverts to the default `BASIC` scanning configuration without rules.
 *
 * ## Example Usage
 *
 * ### Basic example
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const configuration = new aws.ecr.RegistryScanningConfiguration("configuration", {
 *     scanType: "ENHANCED",
 *     rules: [{
 *         scanFrequency: "CONTINUOUS_SCAN",
 *         repositoryFilters: [{
 *             filter: "example",
 *             filterType: "WILDCARD",
 *         }],
 *     }],
 * });
 * ```
 *
 * ### Multiple rules
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.ecr.RegistryScanningConfiguration("test", {
 *     scanType: "ENHANCED",
 *     rules: [
 *         {
 *             scanFrequency: "SCAN_ON_PUSH",
 *             repositoryFilters: [{
 *                 filter: "*",
 *                 filterType: "WILDCARD",
 *             }],
 *         },
 *         {
 *             scanFrequency: "CONTINUOUS_SCAN",
 *             repositoryFilters: [{
 *                 filter: "example",
 *                 filterType: "WILDCARD",
 *             }],
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import ECR Scanning Configurations using the `registry_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ecr/registryScanningConfiguration:RegistryScanningConfiguration example 123456789012
 * ```
 */
export declare class RegistryScanningConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing RegistryScanningConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RegistryScanningConfigurationState, opts?: pulumi.CustomResourceOptions): RegistryScanningConfiguration;
    /**
     * Returns true if the given object is an instance of RegistryScanningConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RegistryScanningConfiguration;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The registry ID the scanning configuration applies to.
     */
    readonly registryId: pulumi.Output<string>;
    /**
     * One or multiple blocks specifying scanning rules to determine which repository filters are used and at what frequency scanning will occur. See below for schema.
     */
    readonly rules: pulumi.Output<outputs.ecr.RegistryScanningConfigurationRule[] | undefined>;
    /**
     * the scanning type to set for the registry. Can be either `ENHANCED` or `BASIC`.
     */
    readonly scanType: pulumi.Output<string>;
    /**
     * Create a RegistryScanningConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RegistryScanningConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RegistryScanningConfiguration resources.
 */
export interface RegistryScanningConfigurationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The registry ID the scanning configuration applies to.
     */
    registryId?: pulumi.Input<string>;
    /**
     * One or multiple blocks specifying scanning rules to determine which repository filters are used and at what frequency scanning will occur. See below for schema.
     */
    rules?: pulumi.Input<pulumi.Input<inputs.ecr.RegistryScanningConfigurationRule>[]>;
    /**
     * the scanning type to set for the registry. Can be either `ENHANCED` or `BASIC`.
     */
    scanType?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RegistryScanningConfiguration resource.
 */
export interface RegistryScanningConfigurationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * One or multiple blocks specifying scanning rules to determine which repository filters are used and at what frequency scanning will occur. See below for schema.
     */
    rules?: pulumi.Input<pulumi.Input<inputs.ecr.RegistryScanningConfigurationRule>[]>;
    /**
     * the scanning type to set for the registry. Can be either `ENHANCED` or `BASIC`.
     */
    scanType: pulumi.Input<string>;
}
