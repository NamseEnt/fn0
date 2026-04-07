import * as pulumi from "@pulumi/pulumi";
import * as outputs from "./types/output";
export declare function getBucketNotificationRules(args: GetBucketNotificationRulesArgs, opts?: pulumi.InvokeOptions): Promise<GetBucketNotificationRulesResult>;
/**
 * A collection of arguments for invoking getBucketNotificationRules.
 */
export interface GetBucketNotificationRulesArgs {
    bucketId: string;
    id?: string;
}
/**
 * A collection of values returned by getBucketNotificationRules.
 */
export interface GetBucketNotificationRulesResult {
    readonly bucketId: string;
    readonly id: string;
    readonly notificationRules: outputs.GetBucketNotificationRulesNotificationRule[];
}
export declare function getBucketNotificationRulesOutput(args: GetBucketNotificationRulesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetBucketNotificationRulesResult>;
/**
 * A collection of arguments for invoking getBucketNotificationRules.
 */
export interface GetBucketNotificationRulesOutputArgs {
    bucketId: pulumi.Input<string>;
    id?: pulumi.Input<string>;
}
