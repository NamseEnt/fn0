import * as pulumi from "@pulumi/pulumi";
/**
 * Use this data source to get the default EBS encryption KMS key in the current region.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.ebs.getDefaultKmsKey({});
 * const example = new aws.ebs.Volume("example", {
 *     availabilityZone: "us-west-2a",
 *     encrypted: true,
 *     kmsKeyId: current.then(current => current.keyArn),
 * });
 * ```
 */
export declare function getDefaultKmsKey(args?: GetDefaultKmsKeyArgs, opts?: pulumi.InvokeOptions): Promise<GetDefaultKmsKeyResult>;
/**
 * A collection of arguments for invoking getDefaultKmsKey.
 */
export interface GetDefaultKmsKeyArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getDefaultKmsKey.
 */
export interface GetDefaultKmsKeyResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * ARN of the default KMS key uses to encrypt an EBS volume in this region when no key is specified in an API call that creates the volume and encryption by default is enabled.
     */
    readonly keyArn: string;
    readonly region: string;
}
/**
 * Use this data source to get the default EBS encryption KMS key in the current region.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.ebs.getDefaultKmsKey({});
 * const example = new aws.ebs.Volume("example", {
 *     availabilityZone: "us-west-2a",
 *     encrypted: true,
 *     kmsKeyId: current.then(current => current.keyArn),
 * });
 * ```
 */
export declare function getDefaultKmsKeyOutput(args?: GetDefaultKmsKeyOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDefaultKmsKeyResult>;
/**
 * A collection of arguments for invoking getDefaultKmsKey.
 */
export interface GetDefaultKmsKeyOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
