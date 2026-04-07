import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an individual Service Quota.
 *
 * > **NOTE:** Global quotas apply to all AWS regions, but can only be accessed in `us-east-1` in the Commercial partition or `us-gov-west-1` in the GovCloud partition. In other regions, the AWS API will return the error `The request failed because the specified service does not exist.`
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicequotas.ServiceQuota("example", {
 *     quotaCode: "L-F678F1CE",
 *     serviceCode: "vpc",
 *     value: 75,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_servicequotas_service_quota` using the service code and quota code, separated by a front slash (`/`). For example:
 *
 * ~> __NOTE:__ This resource does not require explicit import and will assume management of an existing service quota on Pulumi resource creation.
 *
 * ```sh
 * $ pulumi import aws:servicequotas/serviceQuota:ServiceQuota example vpc/L-F678F1CE
 * ```
 */
export declare class ServiceQuota extends pulumi.CustomResource {
    /**
     * Get an existing ServiceQuota resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServiceQuotaState, opts?: pulumi.CustomResourceOptions): ServiceQuota;
    /**
     * Returns true if the given object is an instance of ServiceQuota.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ServiceQuota;
    /**
     * Whether the service quota can be increased.
     */
    readonly adjustable: pulumi.Output<boolean>;
    /**
     * Amazon Resource Name (ARN) of the service quota.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Default value of the service quota.
     */
    readonly defaultValue: pulumi.Output<number>;
    /**
     * Code of the service quota to track. For example: `L-F678F1CE`. Available values can be found with the [AWS CLI service-quotas list-service-quotas command](https://docs.aws.amazon.com/cli/latest/reference/service-quotas/list-service-quotas.html).
     */
    readonly quotaCode: pulumi.Output<string>;
    /**
     * Name of the quota.
     */
    readonly quotaName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly requestId: pulumi.Output<string>;
    readonly requestStatus: pulumi.Output<string>;
    /**
     * Code of the service to track. For example: `vpc`. Available values can be found with the [AWS CLI service-quotas list-services command](https://docs.aws.amazon.com/cli/latest/reference/service-quotas/list-services.html).
     */
    readonly serviceCode: pulumi.Output<string>;
    /**
     * Name of the service.
     */
    readonly serviceName: pulumi.Output<string>;
    /**
     * Information about the measurement.
     */
    readonly usageMetrics: pulumi.Output<outputs.servicequotas.ServiceQuotaUsageMetric[]>;
    /**
     * Float specifying the desired value for the service quota. If the desired value is higher than the current value, a quota increase request is submitted. When a known request is submitted and pending, the value reflects the desired value of the pending request.
     */
    readonly value: pulumi.Output<number>;
    /**
     * Create a ServiceQuota resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ServiceQuotaArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ServiceQuota resources.
 */
export interface ServiceQuotaState {
    /**
     * Whether the service quota can be increased.
     */
    adjustable?: pulumi.Input<boolean>;
    /**
     * Amazon Resource Name (ARN) of the service quota.
     */
    arn?: pulumi.Input<string>;
    /**
     * Default value of the service quota.
     */
    defaultValue?: pulumi.Input<number>;
    /**
     * Code of the service quota to track. For example: `L-F678F1CE`. Available values can be found with the [AWS CLI service-quotas list-service-quotas command](https://docs.aws.amazon.com/cli/latest/reference/service-quotas/list-service-quotas.html).
     */
    quotaCode?: pulumi.Input<string>;
    /**
     * Name of the quota.
     */
    quotaName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    requestId?: pulumi.Input<string>;
    requestStatus?: pulumi.Input<string>;
    /**
     * Code of the service to track. For example: `vpc`. Available values can be found with the [AWS CLI service-quotas list-services command](https://docs.aws.amazon.com/cli/latest/reference/service-quotas/list-services.html).
     */
    serviceCode?: pulumi.Input<string>;
    /**
     * Name of the service.
     */
    serviceName?: pulumi.Input<string>;
    /**
     * Information about the measurement.
     */
    usageMetrics?: pulumi.Input<pulumi.Input<inputs.servicequotas.ServiceQuotaUsageMetric>[]>;
    /**
     * Float specifying the desired value for the service quota. If the desired value is higher than the current value, a quota increase request is submitted. When a known request is submitted and pending, the value reflects the desired value of the pending request.
     */
    value?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a ServiceQuota resource.
 */
export interface ServiceQuotaArgs {
    /**
     * Code of the service quota to track. For example: `L-F678F1CE`. Available values can be found with the [AWS CLI service-quotas list-service-quotas command](https://docs.aws.amazon.com/cli/latest/reference/service-quotas/list-service-quotas.html).
     */
    quotaCode: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Code of the service to track. For example: `vpc`. Available values can be found with the [AWS CLI service-quotas list-services command](https://docs.aws.amazon.com/cli/latest/reference/service-quotas/list-services.html).
     */
    serviceCode: pulumi.Input<string>;
    /**
     * Float specifying the desired value for the service quota. If the desired value is higher than the current value, a quota increase request is submitted. When a known request is submitted and pending, the value reflects the desired value of the pending request.
     */
    value: pulumi.Input<number>;
}
