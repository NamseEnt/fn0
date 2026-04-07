import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Retrieve information about a GuardDuty detector.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.guardduty.getDetector({});
 * ```
 */
export declare function getDetector(args?: GetDetectorArgs, opts?: pulumi.InvokeOptions): Promise<GetDetectorResult>;
/**
 * A collection of arguments for invoking getDetector.
 */
export interface GetDetectorArgs {
    /**
     * ID of the detector.
     */
    id?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags for the resource.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getDetector.
 */
export interface GetDetectorResult {
    /**
     * ARN of the detector.
     */
    readonly arn: string;
    /**
     * Current configuration of the detector features.
     */
    readonly features: outputs.guardduty.GetDetectorFeature[];
    /**
     * The frequency of notifications sent about subsequent finding occurrences.
     */
    readonly findingPublishingFrequency: string;
    readonly id: string;
    readonly region: string;
    /**
     * Service-linked role that grants GuardDuty access to the resources in the AWS account.
     */
    readonly serviceRoleArn: string;
    /**
     * Current status of the detector.
     */
    readonly status: string;
    /**
     * Map of tags for the resource.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Retrieve information about a GuardDuty detector.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.guardduty.getDetector({});
 * ```
 */
export declare function getDetectorOutput(args?: GetDetectorOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDetectorResult>;
/**
 * A collection of arguments for invoking getDetector.
 */
export interface GetDetectorOutputArgs {
    /**
     * ID of the detector.
     */
    id?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags for the resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
