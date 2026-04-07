import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage a single Amazon GuardDuty [detector feature](https://docs.aws.amazon.com/guardduty/latest/ug/guardduty-features-activation-model.html#guardduty-features).
 *
 * > **NOTE:** Deleting this resource does not disable the detector feature, the resource in simply removed from state instead.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.guardduty.Detector("example", {enable: true});
 * const s3Protection = new aws.guardduty.DetectorFeature("s3_protection", {
 *     detectorId: example.id,
 *     name: "S3_DATA_EVENTS",
 *     status: "ENABLED",
 * });
 * ```
 *
 * ## Extended Threat Detection for EKS
 *
 * To enable GuardDuty [Extended Threat Detection](https://docs.aws.amazon.com/guardduty/latest/ug/guardduty-extended-threat-detection.html) for EKS, you need at least one of these features enabled: [EKS Protection](https://docs.aws.amazon.com/guardduty/latest/ug/kubernetes-protection.html) or [Runtime Monitoring](https://docs.aws.amazon.com/guardduty/latest/ug/runtime-monitoring-configuration.html). For maximum detection coverage, enabling both is recommended to enhance detection capabilities.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.guardduty.Detector("example", {enable: true});
 * const eksProtection = new aws.guardduty.DetectorFeature("eks_protection", {
 *     detectorId: example.id,
 *     name: "EKS_AUDIT_LOGS",
 *     status: "ENABLED",
 * });
 * const eksRuntimeMonitoring = new aws.guardduty.DetectorFeature("eks_runtime_monitoring", {
 *     detectorId: example.id,
 *     name: "EKS_RUNTIME_MONITORING",
 *     status: "ENABLED",
 *     additionalConfigurations: [{
 *         name: "EKS_ADDON_MANAGEMENT",
 *         status: "ENABLED",
 *     }],
 * });
 * ```
 */
export declare class DetectorFeature extends pulumi.CustomResource {
    /**
     * Get an existing DetectorFeature resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DetectorFeatureState, opts?: pulumi.CustomResourceOptions): DetectorFeature;
    /**
     * Returns true if the given object is an instance of DetectorFeature.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DetectorFeature;
    /**
     * Additional feature configuration block for features`EKS_RUNTIME_MONITORING` or `RUNTIME_MONITORING`. See below.
     */
    readonly additionalConfigurations: pulumi.Output<outputs.guardduty.DetectorFeatureAdditionalConfiguration[] | undefined>;
    /**
     * Amazon GuardDuty detector ID.
     */
    readonly detectorId: pulumi.Output<string>;
    /**
     * The name of the detector feature. Valid values: `S3_DATA_EVENTS`, `EKS_AUDIT_LOGS`, `EBS_MALWARE_PROTECTION`, `RDS_LOGIN_EVENTS`, `EKS_RUNTIME_MONITORING`, `LAMBDA_NETWORK_LOGS`, `RUNTIME_MONITORING`. Only one of two features `EKS_RUNTIME_MONITORING` or `RUNTIME_MONITORING` can be added, adding both features will cause an error. Refer to the [AWS Documentation](https://docs.aws.amazon.com/guardduty/latest/APIReference/API_DetectorFeatureConfiguration.html) for the current list of supported values.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The status of the detector feature. Valid values: `ENABLED`, `DISABLED`.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Create a DetectorFeature resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DetectorFeatureArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DetectorFeature resources.
 */
export interface DetectorFeatureState {
    /**
     * Additional feature configuration block for features`EKS_RUNTIME_MONITORING` or `RUNTIME_MONITORING`. See below.
     */
    additionalConfigurations?: pulumi.Input<pulumi.Input<inputs.guardduty.DetectorFeatureAdditionalConfiguration>[]>;
    /**
     * Amazon GuardDuty detector ID.
     */
    detectorId?: pulumi.Input<string>;
    /**
     * The name of the detector feature. Valid values: `S3_DATA_EVENTS`, `EKS_AUDIT_LOGS`, `EBS_MALWARE_PROTECTION`, `RDS_LOGIN_EVENTS`, `EKS_RUNTIME_MONITORING`, `LAMBDA_NETWORK_LOGS`, `RUNTIME_MONITORING`. Only one of two features `EKS_RUNTIME_MONITORING` or `RUNTIME_MONITORING` can be added, adding both features will cause an error. Refer to the [AWS Documentation](https://docs.aws.amazon.com/guardduty/latest/APIReference/API_DetectorFeatureConfiguration.html) for the current list of supported values.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The status of the detector feature. Valid values: `ENABLED`, `DISABLED`.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DetectorFeature resource.
 */
export interface DetectorFeatureArgs {
    /**
     * Additional feature configuration block for features`EKS_RUNTIME_MONITORING` or `RUNTIME_MONITORING`. See below.
     */
    additionalConfigurations?: pulumi.Input<pulumi.Input<inputs.guardduty.DetectorFeatureAdditionalConfiguration>[]>;
    /**
     * Amazon GuardDuty detector ID.
     */
    detectorId: pulumi.Input<string>;
    /**
     * The name of the detector feature. Valid values: `S3_DATA_EVENTS`, `EKS_AUDIT_LOGS`, `EBS_MALWARE_PROTECTION`, `RDS_LOGIN_EVENTS`, `EKS_RUNTIME_MONITORING`, `LAMBDA_NETWORK_LOGS`, `RUNTIME_MONITORING`. Only one of two features `EKS_RUNTIME_MONITORING` or `RUNTIME_MONITORING` can be added, adding both features will cause an error. Refer to the [AWS Documentation](https://docs.aws.amazon.com/guardduty/latest/APIReference/API_DetectorFeatureConfiguration.html) for the current list of supported values.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The status of the detector feature. Valid values: `ENABLED`, `DISABLED`.
     */
    status: pulumi.Input<string>;
}
