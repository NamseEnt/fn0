import * as pulumi from "@pulumi/pulumi";
/**
 * Creates a new Amazon Redshift Serverless Usage Limit.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshiftserverless.Workgroup("example", {
 *     namespaceName: exampleAwsRedshiftserverlessNamespace.namespaceName,
 *     workgroupName: "example",
 * });
 * const exampleUsageLimit = new aws.redshiftserverless.UsageLimit("example", {
 *     resourceArn: example.arn,
 *     usageType: "serverless-compute",
 *     amount: 60,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Serverless Usage Limits using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshiftserverless/usageLimit:UsageLimit example example-id
 * ```
 */
export declare class UsageLimit extends pulumi.CustomResource {
    /**
     * Get an existing UsageLimit resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: UsageLimitState, opts?: pulumi.CustomResourceOptions): UsageLimit;
    /**
     * Returns true if the given object is an instance of UsageLimit.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is UsageLimit;
    /**
     * The limit amount. If time-based, this amount is in Redshift Processing Units (RPU) consumed per hour. If data-based, this amount is in terabytes (TB) of data transferred between Regions in cross-account sharing. The value must be a positive number.
     */
    readonly amount: pulumi.Output<number>;
    /**
     * Amazon Resource Name (ARN) of the Redshift Serverless Usage Limit.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The action that Amazon Redshift Serverless takes when the limit is reached. Valid values are `log`, `emit-metric`, and `deactivate`. The default is `log`.
     */
    readonly breachAction: pulumi.Output<string | undefined>;
    /**
     * The time period that the amount applies to. A weekly period begins on Sunday. Valid values are `daily`, `weekly`, and `monthly`. The default is `monthly`.
     */
    readonly period: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the Amazon Redshift Serverless resource to create the usage limit for.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * The type of Amazon Redshift Serverless usage to create a usage limit for. Valid values are `serverless-compute` or `cross-region-datasharing`.
     */
    readonly usageType: pulumi.Output<string>;
    /**
     * Create a UsageLimit resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: UsageLimitArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering UsageLimit resources.
 */
export interface UsageLimitState {
    /**
     * The limit amount. If time-based, this amount is in Redshift Processing Units (RPU) consumed per hour. If data-based, this amount is in terabytes (TB) of data transferred between Regions in cross-account sharing. The value must be a positive number.
     */
    amount?: pulumi.Input<number>;
    /**
     * Amazon Resource Name (ARN) of the Redshift Serverless Usage Limit.
     */
    arn?: pulumi.Input<string>;
    /**
     * The action that Amazon Redshift Serverless takes when the limit is reached. Valid values are `log`, `emit-metric`, and `deactivate`. The default is `log`.
     */
    breachAction?: pulumi.Input<string>;
    /**
     * The time period that the amount applies to. A weekly period begins on Sunday. Valid values are `daily`, `weekly`, and `monthly`. The default is `monthly`.
     */
    period?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Amazon Redshift Serverless resource to create the usage limit for.
     */
    resourceArn?: pulumi.Input<string>;
    /**
     * The type of Amazon Redshift Serverless usage to create a usage limit for. Valid values are `serverless-compute` or `cross-region-datasharing`.
     */
    usageType?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a UsageLimit resource.
 */
export interface UsageLimitArgs {
    /**
     * The limit amount. If time-based, this amount is in Redshift Processing Units (RPU) consumed per hour. If data-based, this amount is in terabytes (TB) of data transferred between Regions in cross-account sharing. The value must be a positive number.
     */
    amount: pulumi.Input<number>;
    /**
     * The action that Amazon Redshift Serverless takes when the limit is reached. Valid values are `log`, `emit-metric`, and `deactivate`. The default is `log`.
     */
    breachAction?: pulumi.Input<string>;
    /**
     * The time period that the amount applies to. A weekly period begins on Sunday. Valid values are `daily`, `weekly`, and `monthly`. The default is `monthly`.
     */
    period?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Amazon Redshift Serverless resource to create the usage limit for.
     */
    resourceArn: pulumi.Input<string>;
    /**
     * The type of Amazon Redshift Serverless usage to create a usage limit for. Valid values are `serverless-compute` or `cross-region-datasharing`.
     */
    usageType: pulumi.Input<string>;
}
