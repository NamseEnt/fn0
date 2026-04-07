import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS CloudFront Continuous Deployment Policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const staging = new aws.cloudfront.Distribution("staging", {
 *     enabled: true,
 *     staging: true,
 * });
 * const example = new aws.cloudfront.ContinuousDeploymentPolicy("example", {
 *     enabled: true,
 *     stagingDistributionDnsNames: {
 *         items: [staging.domainName],
 *         quantity: 1,
 *     },
 *     trafficConfig: {
 *         type: "SingleWeight",
 *         singleWeightConfig: {
 *             weight: 0.01,
 *         },
 *     },
 * });
 * const production = new aws.cloudfront.Distribution("production", {
 *     enabled: true,
 *     continuousDeploymentPolicyId: example.id,
 * });
 * ```
 *
 * ### Single Weight Config with Session Stickiness
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudfront.ContinuousDeploymentPolicy("example", {
 *     enabled: true,
 *     stagingDistributionDnsNames: {
 *         items: [staging.domainName],
 *         quantity: 1,
 *     },
 *     trafficConfig: {
 *         type: "SingleWeight",
 *         singleWeightConfig: {
 *             weight: 0.01,
 *             sessionStickinessConfig: {
 *                 idleTtl: 300,
 *                 maximumTtl: 600,
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ### Single Header Config
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudfront.ContinuousDeploymentPolicy("example", {
 *     enabled: true,
 *     stagingDistributionDnsNames: {
 *         items: [staging.domainName],
 *         quantity: 1,
 *     },
 *     trafficConfig: {
 *         type: "SingleHeader",
 *         singleHeaderConfig: {
 *             header: "aws-cf-cd-example",
 *             value: "example",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudFront Continuous Deployment Policy using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudfront/continuousDeploymentPolicy:ContinuousDeploymentPolicy example abcd-1234
 * ```
 */
export declare class ContinuousDeploymentPolicy extends pulumi.CustomResource {
    /**
     * Get an existing ContinuousDeploymentPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ContinuousDeploymentPolicyState, opts?: pulumi.CustomResourceOptions): ContinuousDeploymentPolicy;
    /**
     * Returns true if the given object is an instance of ContinuousDeploymentPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ContinuousDeploymentPolicy;
    /**
     * The continuous deployment policy ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Whether this continuous deployment policy is enabled.
     */
    readonly enabled: pulumi.Output<boolean>;
    /**
     * Current version of the continuous distribution policy.
     */
    readonly etag: pulumi.Output<string>;
    /**
     * Date and time the continuous deployment policy was last modified.
     */
    readonly lastModifiedTime: pulumi.Output<string>;
    /**
     * CloudFront domain name of the staging distribution. See `stagingDistributionDnsNames`.
     */
    readonly stagingDistributionDnsNames: pulumi.Output<outputs.cloudfront.ContinuousDeploymentPolicyStagingDistributionDnsNames | undefined>;
    /**
     * Parameters for routing production traffic from primary to staging distributions. See `trafficConfig`.
     */
    readonly trafficConfig: pulumi.Output<outputs.cloudfront.ContinuousDeploymentPolicyTrafficConfig | undefined>;
    /**
     * Create a ContinuousDeploymentPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ContinuousDeploymentPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ContinuousDeploymentPolicy resources.
 */
export interface ContinuousDeploymentPolicyState {
    /**
     * The continuous deployment policy ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * Whether this continuous deployment policy is enabled.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Current version of the continuous distribution policy.
     */
    etag?: pulumi.Input<string>;
    /**
     * Date and time the continuous deployment policy was last modified.
     */
    lastModifiedTime?: pulumi.Input<string>;
    /**
     * CloudFront domain name of the staging distribution. See `stagingDistributionDnsNames`.
     */
    stagingDistributionDnsNames?: pulumi.Input<inputs.cloudfront.ContinuousDeploymentPolicyStagingDistributionDnsNames>;
    /**
     * Parameters for routing production traffic from primary to staging distributions. See `trafficConfig`.
     */
    trafficConfig?: pulumi.Input<inputs.cloudfront.ContinuousDeploymentPolicyTrafficConfig>;
}
/**
 * The set of arguments for constructing a ContinuousDeploymentPolicy resource.
 */
export interface ContinuousDeploymentPolicyArgs {
    /**
     * Whether this continuous deployment policy is enabled.
     */
    enabled: pulumi.Input<boolean>;
    /**
     * CloudFront domain name of the staging distribution. See `stagingDistributionDnsNames`.
     */
    stagingDistributionDnsNames?: pulumi.Input<inputs.cloudfront.ContinuousDeploymentPolicyStagingDistributionDnsNames>;
    /**
     * Parameters for routing production traffic from primary to staging distributions. See `trafficConfig`.
     */
    trafficConfig?: pulumi.Input<inputs.cloudfront.ContinuousDeploymentPolicyTrafficConfig>;
}
