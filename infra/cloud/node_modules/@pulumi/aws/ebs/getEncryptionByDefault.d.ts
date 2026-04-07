import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a way to check whether default EBS encryption is enabled for your AWS account in the current AWS region.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.ebs.getEncryptionByDefault({});
 * ```
 */
export declare function getEncryptionByDefault(args?: GetEncryptionByDefaultArgs, opts?: pulumi.InvokeOptions): Promise<GetEncryptionByDefaultResult>;
/**
 * A collection of arguments for invoking getEncryptionByDefault.
 */
export interface GetEncryptionByDefaultArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getEncryptionByDefault.
 */
export interface GetEncryptionByDefaultResult {
    /**
     * Whether or not default EBS encryption is enabled. Returns as `true` or `false`.
     */
    readonly enabled: boolean;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Provides a way to check whether default EBS encryption is enabled for your AWS account in the current AWS region.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.ebs.getEncryptionByDefault({});
 * ```
 */
export declare function getEncryptionByDefaultOutput(args?: GetEncryptionByDefaultOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetEncryptionByDefaultResult>;
/**
 * A collection of arguments for invoking getEncryptionByDefault.
 */
export interface GetEncryptionByDefaultOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
