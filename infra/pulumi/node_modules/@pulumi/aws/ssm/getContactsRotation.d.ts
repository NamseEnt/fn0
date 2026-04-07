import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssm.getContactsRotation({
 *     arn: "arn:aws:ssm-contacts:us-east-1:012345678910:rotation/example",
 * });
 * ```
 */
export declare function getContactsRotation(args: GetContactsRotationArgs, opts?: pulumi.InvokeOptions): Promise<GetContactsRotationResult>;
/**
 * A collection of arguments for invoking getContactsRotation.
 */
export interface GetContactsRotationArgs {
    /**
     * The Amazon Resource Name (ARN) of the rotation.
     */
    arn: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getContactsRotation.
 */
export interface GetContactsRotationResult {
    readonly arn: string;
    /**
     * The Amazon Resource Names (ARNs) of the contacts to add to the rotation. The order in which you list the contacts is their shift order in the rotation schedule.
     */
    readonly contactIds: string[];
    readonly id: string;
    /**
     * The name for the rotation.
     */
    readonly name: string;
    /**
     * Information about when an on-call rotation is in effect and how long the rotation period lasts.
     */
    readonly recurrences: outputs.ssm.GetContactsRotationRecurrence[];
    readonly region: string;
    /**
     * The date and time, in RFC 3339 format, that the rotation goes into effect.
     */
    readonly startTime: string;
    /**
     * A map of tags to assign to the resource.
     */
    readonly tags: {
        [key: string]: string;
    };
    /**
     * The time zone to base the rotation’s activity on in Internet Assigned Numbers Authority (IANA) format.
     */
    readonly timeZoneId: string;
}
/**
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssm.getContactsRotation({
 *     arn: "arn:aws:ssm-contacts:us-east-1:012345678910:rotation/example",
 * });
 * ```
 */
export declare function getContactsRotationOutput(args: GetContactsRotationOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetContactsRotationResult>;
/**
 * A collection of arguments for invoking getContactsRotation.
 */
export interface GetContactsRotationOutputArgs {
    /**
     * The Amazon Resource Name (ARN) of the rotation.
     */
    arn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
