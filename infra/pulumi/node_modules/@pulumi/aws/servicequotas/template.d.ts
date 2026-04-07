import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Service Quotas Template.
 *
 * > Only the management account of an organization can alter Service Quota templates, and this must be done from the `us-east-1` region.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicequotas.Template("example", {
 *     awsRegion: "us-east-1",
 *     quotaCode: "L-2ACBD22F",
 *     serviceCode: "lambda",
 *     value: 80,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Service Quotas Template using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:servicequotas/template:Template example us-east-1,L-2ACBD22F,lambda
 * ```
 */
export declare class Template extends pulumi.CustomResource {
    /**
     * Get an existing Template resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TemplateState, opts?: pulumi.CustomResourceOptions): Template;
    /**
     * Returns true if the given object is an instance of Template.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Template;
    /**
     * AWS Region to which the template applies.
     */
    readonly awsRegion: pulumi.Output<string>;
    /**
     * Indicates whether the quota is global.
     */
    readonly globalQuota: pulumi.Output<boolean>;
    /**
     * Quota identifier. To find the quota code for a specific quota, use the aws.servicequotas.ServiceQuota data source.
     */
    readonly quotaCode: pulumi.Output<string>;
    /**
     * Quota name.
     */
    readonly quotaName: pulumi.Output<string>;
    /**
     * AWS Region to which the template applies. Use `aws.getRegion` instead.
     *
     * @deprecated region is deprecated. Use aws.getRegion instead.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Service identifier. To find the service code value for an AWS service, use the aws.servicequotas.getService data source.
     */
    readonly serviceCode: pulumi.Output<string>;
    /**
     * Service name.
     */
    readonly serviceName: pulumi.Output<string>;
    /**
     * Unit of measurement.
     */
    readonly unit: pulumi.Output<string>;
    /**
     * The new, increased value for the quota.
     */
    readonly value: pulumi.Output<number>;
    /**
     * Create a Template resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TemplateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Template resources.
 */
export interface TemplateState {
    /**
     * AWS Region to which the template applies.
     */
    awsRegion?: pulumi.Input<string>;
    /**
     * Indicates whether the quota is global.
     */
    globalQuota?: pulumi.Input<boolean>;
    /**
     * Quota identifier. To find the quota code for a specific quota, use the aws.servicequotas.ServiceQuota data source.
     */
    quotaCode?: pulumi.Input<string>;
    /**
     * Quota name.
     */
    quotaName?: pulumi.Input<string>;
    /**
     * AWS Region to which the template applies. Use `aws.getRegion` instead.
     *
     * @deprecated region is deprecated. Use aws.getRegion instead.
     */
    region?: pulumi.Input<string>;
    /**
     * Service identifier. To find the service code value for an AWS service, use the aws.servicequotas.getService data source.
     */
    serviceCode?: pulumi.Input<string>;
    /**
     * Service name.
     */
    serviceName?: pulumi.Input<string>;
    /**
     * Unit of measurement.
     */
    unit?: pulumi.Input<string>;
    /**
     * The new, increased value for the quota.
     */
    value?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a Template resource.
 */
export interface TemplateArgs {
    /**
     * AWS Region to which the template applies.
     */
    awsRegion?: pulumi.Input<string>;
    /**
     * Quota identifier. To find the quota code for a specific quota, use the aws.servicequotas.ServiceQuota data source.
     */
    quotaCode: pulumi.Input<string>;
    /**
     * AWS Region to which the template applies. Use `aws.getRegion` instead.
     *
     * @deprecated region is deprecated. Use aws.getRegion instead.
     */
    region?: pulumi.Input<string>;
    /**
     * Service identifier. To find the service code value for an AWS service, use the aws.servicequotas.getService data source.
     */
    serviceCode: pulumi.Input<string>;
    /**
     * The new, increased value for the quota.
     */
    value: pulumi.Input<number>;
}
