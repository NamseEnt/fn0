import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS Audit Manager Framework.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.auditmanager.getFramework({
 *     name: "Essential Eight",
 *     frameworkType: "Standard",
 * });
 * ```
 */
export declare function getFramework(args: GetFrameworkArgs, opts?: pulumi.InvokeOptions): Promise<GetFrameworkResult>;
/**
 * A collection of arguments for invoking getFramework.
 */
export interface GetFrameworkArgs {
    frameworkType: string;
    /**
     * Name of the framework.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getFramework.
 */
export interface GetFrameworkResult {
    readonly arn: string;
    readonly complianceType: string;
    readonly controlSets: outputs.auditmanager.GetFrameworkControlSet[];
    readonly description: string;
    readonly frameworkType: string;
    readonly id: string;
    readonly name: string;
    readonly region: string;
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Data source for managing an AWS Audit Manager Framework.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.auditmanager.getFramework({
 *     name: "Essential Eight",
 *     frameworkType: "Standard",
 * });
 * ```
 */
export declare function getFrameworkOutput(args: GetFrameworkOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetFrameworkResult>;
/**
 * A collection of arguments for invoking getFramework.
 */
export interface GetFrameworkOutputArgs {
    frameworkType: pulumi.Input<string>;
    /**
     * Name of the framework.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
