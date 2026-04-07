import * as pulumi from "@pulumi/pulumi";
/**
 * Provides details about a specific Connect Lambda Function Association.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.connect.getLambdaFunctionAssociation({
 *     functionArn: "arn:aws:lambda:us-west-2:123456789123:function:abcdefg",
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 * });
 * ```
 */
export declare function getLambdaFunctionAssociation(args: GetLambdaFunctionAssociationArgs, opts?: pulumi.InvokeOptions): Promise<GetLambdaFunctionAssociationResult>;
/**
 * A collection of arguments for invoking getLambdaFunctionAssociation.
 */
export interface GetLambdaFunctionAssociationArgs {
    /**
     * ARN of the Lambda Function, omitting any version or alias qualifier.
     */
    functionArn: string;
    /**
     * Identifier of the Amazon Connect instance. You can find the instanceId in the ARN of the instance.
     */
    instanceId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getLambdaFunctionAssociation.
 */
export interface GetLambdaFunctionAssociationResult {
    readonly functionArn: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly instanceId: string;
    readonly region: string;
}
/**
 * Provides details about a specific Connect Lambda Function Association.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.connect.getLambdaFunctionAssociation({
 *     functionArn: "arn:aws:lambda:us-west-2:123456789123:function:abcdefg",
 *     instanceId: "aaaaaaaa-bbbb-cccc-dddd-111111111111",
 * });
 * ```
 */
export declare function getLambdaFunctionAssociationOutput(args: GetLambdaFunctionAssociationOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetLambdaFunctionAssociationResult>;
/**
 * A collection of arguments for invoking getLambdaFunctionAssociation.
 */
export interface GetLambdaFunctionAssociationOutputArgs {
    /**
     * ARN of the Lambda Function, omitting any version or alias qualifier.
     */
    functionArn: pulumi.Input<string>;
    /**
     * Identifier of the Amazon Connect instance. You can find the instanceId in the ARN of the instance.
     */
    instanceId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
