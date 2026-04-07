import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS SESv2 (Simple Email V2) Configuration Set.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.sesv2.getConfigurationSet({
 *     configurationSetName: "example",
 * });
 * ```
 */
export declare function getConfigurationSet(args: GetConfigurationSetArgs, opts?: pulumi.InvokeOptions): Promise<GetConfigurationSetResult>;
/**
 * A collection of arguments for invoking getConfigurationSet.
 */
export interface GetConfigurationSetArgs {
    /**
     * The name of the configuration set.
     */
    configurationSetName: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Key-value map of resource tags for the container recipe.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getConfigurationSet.
 */
export interface GetConfigurationSetResult {
    readonly arn: string;
    readonly configurationSetName: string;
    /**
     * An object that defines the dedicated IP pool that is used to send emails that you send using the configuration set.
     */
    readonly deliveryOptions: outputs.sesv2.GetConfigurationSetDeliveryOption[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    /**
     * An object that defines whether or not Amazon SES collects reputation metrics for the emails that you send that use the configuration set.
     */
    readonly reputationOptions: outputs.sesv2.GetConfigurationSetReputationOption[];
    /**
     * An object that defines whether or not Amazon SES can send email that you send using the configuration set.
     */
    readonly sendingOptions: outputs.sesv2.GetConfigurationSetSendingOption[];
    /**
     * An object that contains information about the suppression list preferences for your account.
     */
    readonly suppressionOptions: outputs.sesv2.GetConfigurationSetSuppressionOption[];
    /**
     * Key-value map of resource tags for the container recipe.
     */
    readonly tags: {
        [key: string]: string;
    };
    /**
     * An object that defines the open and click tracking options for emails that you send using the configuration set.
     */
    readonly trackingOptions: outputs.sesv2.GetConfigurationSetTrackingOption[];
    /**
     * An object that contains information about the VDM preferences for your configuration set.
     */
    readonly vdmOptions: outputs.sesv2.GetConfigurationSetVdmOption[];
}
/**
 * Data source for managing an AWS SESv2 (Simple Email V2) Configuration Set.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.sesv2.getConfigurationSet({
 *     configurationSetName: "example",
 * });
 * ```
 */
export declare function getConfigurationSetOutput(args: GetConfigurationSetOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetConfigurationSetResult>;
/**
 * A collection of arguments for invoking getConfigurationSet.
 */
export interface GetConfigurationSetOutputArgs {
    /**
     * The name of the configuration set.
     */
    configurationSetName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the container recipe.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
