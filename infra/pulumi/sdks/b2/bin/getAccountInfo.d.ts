import * as pulumi from "@pulumi/pulumi";
import * as outputs from "./types/output";
export declare function getAccountInfo(args?: GetAccountInfoArgs, opts?: pulumi.InvokeOptions): Promise<GetAccountInfoResult>;
/**
 * A collection of arguments for invoking getAccountInfo.
 */
export interface GetAccountInfoArgs {
    id?: string;
}
/**
 * A collection of values returned by getAccountInfo.
 */
export interface GetAccountInfoResult {
    readonly absoluteMinimumPartSize: number;
    readonly accountAuthToken: string;
    readonly accountId: string;
    readonly alloweds: outputs.GetAccountInfoAllowed[];
    readonly apiUrl: string;
    readonly downloadUrl: string;
    readonly id: string;
    readonly recommendedPartSize: number;
    readonly s3ApiUrl: string;
}
export declare function getAccountInfoOutput(args?: GetAccountInfoOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetAccountInfoResult>;
/**
 * A collection of arguments for invoking getAccountInfo.
 */
export interface GetAccountInfoOutputArgs {
    id?: pulumi.Input<string>;
}
