import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides details about an Image Builder Distribution Configuration.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.imagebuilder.getDistributionConfiguration({
 *     arn: "arn:aws:imagebuilder:us-west-2:aws:distribution-configuration/example",
 * });
 * ```
 */
export declare function getDistributionConfiguration(args: GetDistributionConfigurationArgs, opts?: pulumi.InvokeOptions): Promise<GetDistributionConfigurationResult>;
/**
 * A collection of arguments for invoking getDistributionConfiguration.
 */
export interface GetDistributionConfigurationArgs {
    /**
     * ARN of the distribution configuration.
     */
    arn: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Key-value map of resource tags for the distribution configuration.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getDistributionConfiguration.
 */
export interface GetDistributionConfigurationResult {
    readonly arn: string;
    /**
     * Date the distribution configuration was created.
     */
    readonly dateCreated: string;
    /**
     * Date the distribution configuration was updated.
     */
    readonly dateUpdated: string;
    /**
     * Description of the container distribution configuration.
     */
    readonly description: string;
    /**
     * Set of distributions.
     */
    readonly distributions: outputs.imagebuilder.GetDistributionConfigurationDistribution[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Name of the distribution configuration.
     */
    readonly name: string;
    /**
     * AWS Region of distribution.
     */
    readonly region: string;
    /**
     * Key-value map of resource tags for the distribution configuration.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Provides details about an Image Builder Distribution Configuration.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.imagebuilder.getDistributionConfiguration({
 *     arn: "arn:aws:imagebuilder:us-west-2:aws:distribution-configuration/example",
 * });
 * ```
 */
export declare function getDistributionConfigurationOutput(args: GetDistributionConfigurationOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDistributionConfigurationResult>;
/**
 * A collection of arguments for invoking getDistributionConfiguration.
 */
export interface GetDistributionConfigurationOutputArgs {
    /**
     * ARN of the distribution configuration.
     */
    arn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the distribution configuration.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
