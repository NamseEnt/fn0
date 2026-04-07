import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * !> **WARNING:** This data source's functionality was removed in version 2.0.0 of the AWS Provider. You can migrate existing configurations to the `aws.kms.getSecrets` data source following instructions available in the Version 2 Upgrade Guide. This data source will be removed in a future version.
 */
export declare function getSecret(args: GetSecretArgs, opts?: pulumi.InvokeOptions): Promise<GetSecretResult>;
/**
 * A collection of arguments for invoking getSecret.
 */
export interface GetSecretArgs {
    region?: string;
    secrets: inputs.kms.GetSecretSecret[];
}
/**
 * A collection of values returned by getSecret.
 */
export interface GetSecretResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    readonly secrets: outputs.kms.GetSecretSecret[];
}
/**
 * !> **WARNING:** This data source's functionality was removed in version 2.0.0 of the AWS Provider. You can migrate existing configurations to the `aws.kms.getSecrets` data source following instructions available in the Version 2 Upgrade Guide. This data source will be removed in a future version.
 */
export declare function getSecretOutput(args: GetSecretOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSecretResult>;
/**
 * A collection of arguments for invoking getSecret.
 */
export interface GetSecretOutputArgs {
    region?: pulumi.Input<string>;
    secrets: pulumi.Input<pulumi.Input<inputs.kms.GetSecretSecretArgs>[]>;
}
