import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Retrieve information about an Elastic Beanstalk Application.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.elasticbeanstalk.getApplication({
 *     name: "example",
 * });
 * export const arn = example.then(example => example.arn);
 * export const description = example.then(example => example.description);
 * ```
 */
export declare function getApplication(args: GetApplicationArgs, opts?: pulumi.InvokeOptions): Promise<GetApplicationResult>;
/**
 * A collection of arguments for invoking getApplication.
 */
export interface GetApplicationArgs {
    /**
     * Name of the application
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getApplication.
 */
export interface GetApplicationResult {
    readonly appversionLifecycle: outputs.elasticbeanstalk.GetApplicationAppversionLifecycle;
    /**
     * ARN of the application.
     */
    readonly arn: string;
    /**
     * Short description of the application
     */
    readonly description: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
    readonly region: string;
}
/**
 * Retrieve information about an Elastic Beanstalk Application.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.elasticbeanstalk.getApplication({
 *     name: "example",
 * });
 * export const arn = example.then(example => example.arn);
 * export const description = example.then(example => example.description);
 * ```
 */
export declare function getApplicationOutput(args: GetApplicationOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetApplicationResult>;
/**
 * A collection of arguments for invoking getApplication.
 */
export interface GetApplicationOutputArgs {
    /**
     * Name of the application
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
