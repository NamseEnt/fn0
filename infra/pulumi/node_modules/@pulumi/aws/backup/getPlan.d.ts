import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Use this data source to get information on an existing backup plan.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.backup.getPlan({
 *     planId: "my_example_backup_plan_id",
 * });
 * ```
 */
export declare function getPlan(args: GetPlanArgs, opts?: pulumi.InvokeOptions): Promise<GetPlanResult>;
/**
 * A collection of arguments for invoking getPlan.
 */
export interface GetPlanArgs {
    /**
     * Backup plan ID.
     */
    planId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Metadata that you can assign to help organize the plans you create.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getPlan.
 */
export interface GetPlanResult {
    /**
     * ARN of the backup plan.
     */
    readonly arn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Display name of a backup plan.
     */
    readonly name: string;
    readonly planId: string;
    readonly region: string;
    /**
     * Rules of a backup plan.
     */
    readonly rules: outputs.backup.GetPlanRule[];
    /**
     * Metadata that you can assign to help organize the plans you create.
     */
    readonly tags: {
        [key: string]: string;
    };
    /**
     * Unique, randomly generated, Unicode, UTF-8 encoded string that serves as the version ID of the backup plan.
     */
    readonly version: string;
}
/**
 * Use this data source to get information on an existing backup plan.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.backup.getPlan({
 *     planId: "my_example_backup_plan_id",
 * });
 * ```
 */
export declare function getPlanOutput(args: GetPlanOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetPlanResult>;
/**
 * A collection of arguments for invoking getPlan.
 */
export interface GetPlanOutputArgs {
    /**
     * Backup plan ID.
     */
    planId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Metadata that you can assign to help organize the plans you create.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
