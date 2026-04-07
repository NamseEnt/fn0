import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing a Plan of an AWS SSM Contact.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.ssmcontacts.getPlan({
 *     contactId: "arn:aws:ssm-contacts:us-west-2:123456789012:contact/contactalias",
 * });
 * ```
 */
export declare function getPlan(args: GetPlanArgs, opts?: pulumi.InvokeOptions): Promise<GetPlanResult>;
/**
 * A collection of arguments for invoking getPlan.
 */
export interface GetPlanArgs {
    /**
     * The Amazon Resource Name (ARN) of the contact or escalation plan.
     */
    contactId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getPlan.
 */
export interface GetPlanResult {
    readonly contactId: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    /**
     * List of stages. A contact has an engagement plan with stages that contact specified contact channels. An escalation plan uses stages that contact specified contacts.
     */
    readonly stages: outputs.ssmcontacts.GetPlanStage[];
}
/**
 * Data source for managing a Plan of an AWS SSM Contact.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.ssmcontacts.getPlan({
 *     contactId: "arn:aws:ssm-contacts:us-west-2:123456789012:contact/contactalias",
 * });
 * ```
 */
export declare function getPlanOutput(args: GetPlanOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetPlanResult>;
/**
 * A collection of arguments for invoking getPlan.
 */
export interface GetPlanOutputArgs {
    /**
     * The Amazon Resource Name (ARN) of the contact or escalation plan.
     */
    contactId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
