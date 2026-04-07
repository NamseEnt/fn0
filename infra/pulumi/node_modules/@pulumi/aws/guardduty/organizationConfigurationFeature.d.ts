import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage a single Amazon GuardDuty [organization configuration feature](https://docs.aws.amazon.com/guardduty/latest/ug/guardduty-features-activation-model.html#guardduty-features).
 *
 * > **NOTE:** Deleting this resource does not disable the organization configuration feature, the resource is simply removed from state instead.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.guardduty.Detector("example", {enable: true});
 * const eksRuntimeMonitoring = new aws.guardduty.OrganizationConfigurationFeature("eks_runtime_monitoring", {
 *     detectorId: example.id,
 *     name: "EKS_RUNTIME_MONITORING",
 *     autoEnable: "ALL",
 *     additionalConfigurations: [{
 *         name: "EKS_ADDON_MANAGEMENT",
 *         autoEnable: "NEW",
 *     }],
 * });
 * ```
 */
export declare class OrganizationConfigurationFeature extends pulumi.CustomResource {
    /**
     * Get an existing OrganizationConfigurationFeature resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OrganizationConfigurationFeatureState, opts?: pulumi.CustomResourceOptions): OrganizationConfigurationFeature;
    /**
     * Returns true if the given object is an instance of OrganizationConfigurationFeature.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OrganizationConfigurationFeature;
    /**
     * Additional feature configuration block for features `EKS_RUNTIME_MONITORING` or `RUNTIME_MONITORING`. See below.
     */
    readonly additionalConfigurations: pulumi.Output<outputs.guardduty.OrganizationConfigurationFeatureAdditionalConfiguration[] | undefined>;
    /**
     * The status of the feature that is configured for the member accounts within the organization. Valid values: `NEW`, `ALL`, `NONE`.
     */
    readonly autoEnable: pulumi.Output<string>;
    /**
     * The ID of the detector that configures the delegated administrator.
     */
    readonly detectorId: pulumi.Output<string>;
    /**
     * The name of the feature that will be configured for the organization. Valid values: `S3_DATA_EVENTS`, `EKS_AUDIT_LOGS`, `EBS_MALWARE_PROTECTION`, `RDS_LOGIN_EVENTS`, `EKS_RUNTIME_MONITORING`, `LAMBDA_NETWORK_LOGS`, `RUNTIME_MONITORING`. Only one of two features `EKS_RUNTIME_MONITORING` or `RUNTIME_MONITORING` can be added, adding both features will cause an error. Refer to the [AWS Documentation](https://docs.aws.amazon.com/guardduty/latest/APIReference/API_DetectorFeatureConfiguration.html) for the current list of supported values.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a OrganizationConfigurationFeature resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OrganizationConfigurationFeatureArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OrganizationConfigurationFeature resources.
 */
export interface OrganizationConfigurationFeatureState {
    /**
     * Additional feature configuration block for features `EKS_RUNTIME_MONITORING` or `RUNTIME_MONITORING`. See below.
     */
    additionalConfigurations?: pulumi.Input<pulumi.Input<inputs.guardduty.OrganizationConfigurationFeatureAdditionalConfiguration>[]>;
    /**
     * The status of the feature that is configured for the member accounts within the organization. Valid values: `NEW`, `ALL`, `NONE`.
     */
    autoEnable?: pulumi.Input<string>;
    /**
     * The ID of the detector that configures the delegated administrator.
     */
    detectorId?: pulumi.Input<string>;
    /**
     * The name of the feature that will be configured for the organization. Valid values: `S3_DATA_EVENTS`, `EKS_AUDIT_LOGS`, `EBS_MALWARE_PROTECTION`, `RDS_LOGIN_EVENTS`, `EKS_RUNTIME_MONITORING`, `LAMBDA_NETWORK_LOGS`, `RUNTIME_MONITORING`. Only one of two features `EKS_RUNTIME_MONITORING` or `RUNTIME_MONITORING` can be added, adding both features will cause an error. Refer to the [AWS Documentation](https://docs.aws.amazon.com/guardduty/latest/APIReference/API_DetectorFeatureConfiguration.html) for the current list of supported values.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a OrganizationConfigurationFeature resource.
 */
export interface OrganizationConfigurationFeatureArgs {
    /**
     * Additional feature configuration block for features `EKS_RUNTIME_MONITORING` or `RUNTIME_MONITORING`. See below.
     */
    additionalConfigurations?: pulumi.Input<pulumi.Input<inputs.guardduty.OrganizationConfigurationFeatureAdditionalConfiguration>[]>;
    /**
     * The status of the feature that is configured for the member accounts within the organization. Valid values: `NEW`, `ALL`, `NONE`.
     */
    autoEnable: pulumi.Input<string>;
    /**
     * The ID of the detector that configures the delegated administrator.
     */
    detectorId: pulumi.Input<string>;
    /**
     * The name of the feature that will be configured for the organization. Valid values: `S3_DATA_EVENTS`, `EKS_AUDIT_LOGS`, `EBS_MALWARE_PROTECTION`, `RDS_LOGIN_EVENTS`, `EKS_RUNTIME_MONITORING`, `LAMBDA_NETWORK_LOGS`, `RUNTIME_MONITORING`. Only one of two features `EKS_RUNTIME_MONITORING` or `RUNTIME_MONITORING` can be added, adding both features will cause an error. Refer to the [AWS Documentation](https://docs.aws.amazon.com/guardduty/latest/APIReference/API_DetectorFeatureConfiguration.html) for the current list of supported values.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
