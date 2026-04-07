import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Provides details about an AWS Billing Views.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.billing.getViews({
 *     billingViewTypes: ["PRIMARY"],
 * });
 * export const primaryViewArnByTypes = example.then(example => example.billingViews?.[0]?.arn);
 * ```
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.billing.getViews({});
 * export const viewArns = example.then(example => .map(view => (view.arn)));
 * export const primaryViewArnByName = example.then(example => .filter(view => view.name == "Primary View").map(view => (view.arn))[0]);
 * ```
 */
export declare function getViews(args?: GetViewsArgs, opts?: pulumi.InvokeOptions): Promise<GetViewsResult>;
/**
 * A collection of arguments for invoking getViews.
 */
export interface GetViewsArgs {
    /**
     * List of billing view types to retrieve. Valid values are `PRIMARY`, `BILLING_GROUP`, `CUSTOM`.
     */
    billingViewTypes?: string[];
}
/**
 * A collection of values returned by getViews.
 */
export interface GetViewsResult {
    readonly billingViewTypes?: string[];
    /**
     * List of billing view objects with the following attributes:
     */
    readonly billingViews: outputs.billing.GetViewsBillingView[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
}
/**
 * Provides details about an AWS Billing Views.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.billing.getViews({
 *     billingViewTypes: ["PRIMARY"],
 * });
 * export const primaryViewArnByTypes = example.then(example => example.billingViews?.[0]?.arn);
 * ```
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.billing.getViews({});
 * export const viewArns = example.then(example => .map(view => (view.arn)));
 * export const primaryViewArnByName = example.then(example => .filter(view => view.name == "Primary View").map(view => (view.arn))[0]);
 * ```
 */
export declare function getViewsOutput(args?: GetViewsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetViewsResult>;
/**
 * A collection of arguments for invoking getViews.
 */
export interface GetViewsOutputArgs {
    /**
     * List of billing view types to retrieve. Valid values are `PRIMARY`, `BILLING_GROUP`, `CUSTOM`.
     */
    billingViewTypes?: pulumi.Input<pulumi.Input<string>[]>;
}
