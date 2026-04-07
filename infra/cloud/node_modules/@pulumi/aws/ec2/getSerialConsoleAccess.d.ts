import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a way to check whether serial console access is enabled for your AWS account in the current AWS region.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.ec2.getSerialConsoleAccess({});
 * ```
 */
export declare function getSerialConsoleAccess(args?: GetSerialConsoleAccessArgs, opts?: pulumi.InvokeOptions): Promise<GetSerialConsoleAccessResult>;
/**
 * A collection of arguments for invoking getSerialConsoleAccess.
 */
export interface GetSerialConsoleAccessArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getSerialConsoleAccess.
 */
export interface GetSerialConsoleAccessResult {
    /**
     * Whether or not serial console access is enabled. Returns as `true` or `false`.
     */
    readonly enabled: boolean;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Provides a way to check whether serial console access is enabled for your AWS account in the current AWS region.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.ec2.getSerialConsoleAccess({});
 * ```
 */
export declare function getSerialConsoleAccessOutput(args?: GetSerialConsoleAccessOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSerialConsoleAccessResult>;
/**
 * A collection of arguments for invoking getSerialConsoleAccess.
 */
export interface GetSerialConsoleAccessOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
