import * as pulumi from "@pulumi/pulumi";
export declare function getApplicationKey(args: GetApplicationKeyArgs, opts?: pulumi.InvokeOptions): Promise<GetApplicationKeyResult>;
/**
 * A collection of arguments for invoking getApplicationKey.
 */
export interface GetApplicationKeyArgs {
    id?: string;
    keyName: string;
    namePrefix?: string;
}
/**
 * A collection of values returned by getApplicationKey.
 */
export interface GetApplicationKeyResult {
    readonly applicationKeyId: string;
    /**
     * @deprecated Deprecated
     */
    readonly bucketId: string;
    readonly bucketIds: string[];
    readonly capabilities: string[];
    readonly id: string;
    readonly keyName: string;
    readonly namePrefix: string;
    readonly options: string[];
}
export declare function getApplicationKeyOutput(args: GetApplicationKeyOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetApplicationKeyResult>;
/**
 * A collection of arguments for invoking getApplicationKey.
 */
export interface GetApplicationKeyOutputArgs {
    id?: pulumi.Input<string>;
    keyName: pulumi.Input<string>;
    namePrefix?: pulumi.Input<string>;
}
