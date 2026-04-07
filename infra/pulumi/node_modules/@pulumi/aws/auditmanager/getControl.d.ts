import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS Audit Manager Control.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.auditmanager.getControl({
 *     name: "1. Risk Management",
 *     type: "Standard",
 * });
 * ```
 *
 * ### With Framework Resource
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.auditmanager.getControl({
 *     name: "1. Risk Management",
 *     type: "Standard",
 * });
 * const example2 = aws.auditmanager.getControl({
 *     name: "2. Personnel",
 *     type: "Standard",
 * });
 * const exampleFramework = new aws.auditmanager.Framework("example", {
 *     name: "example",
 *     controlSets: [
 *         {
 *             name: "example",
 *             controls: [{
 *                 id: example.then(example => example.id),
 *             }],
 *         },
 *         {
 *             name: "example2",
 *             controls: [{
 *                 id: example2.then(example2 => example2.id),
 *             }],
 *         },
 *     ],
 * });
 * ```
 */
export declare function getControl(args: GetControlArgs, opts?: pulumi.InvokeOptions): Promise<GetControlResult>;
/**
 * A collection of arguments for invoking getControl.
 */
export interface GetControlArgs {
    /**
     * Name of the control.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Type of control. Valid values are `Custom` and `Standard`.
     */
    type: string;
}
/**
 * A collection of values returned by getControl.
 */
export interface GetControlResult {
    readonly actionPlanInstructions: string;
    readonly actionPlanTitle: string;
    readonly arn: string;
    readonly controlMappingSources: outputs.auditmanager.GetControlControlMappingSource[];
    readonly description: string;
    readonly id: string;
    readonly name: string;
    readonly region: string;
    readonly tags: {
        [key: string]: string;
    };
    readonly testingInformation: string;
    readonly type: string;
}
/**
 * Data source for managing an AWS Audit Manager Control.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.auditmanager.getControl({
 *     name: "1. Risk Management",
 *     type: "Standard",
 * });
 * ```
 *
 * ### With Framework Resource
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.auditmanager.getControl({
 *     name: "1. Risk Management",
 *     type: "Standard",
 * });
 * const example2 = aws.auditmanager.getControl({
 *     name: "2. Personnel",
 *     type: "Standard",
 * });
 * const exampleFramework = new aws.auditmanager.Framework("example", {
 *     name: "example",
 *     controlSets: [
 *         {
 *             name: "example",
 *             controls: [{
 *                 id: example.then(example => example.id),
 *             }],
 *         },
 *         {
 *             name: "example2",
 *             controls: [{
 *                 id: example2.then(example2 => example2.id),
 *             }],
 *         },
 *     ],
 * });
 * ```
 */
export declare function getControlOutput(args: GetControlOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetControlResult>;
/**
 * A collection of arguments for invoking getControl.
 */
export interface GetControlOutputArgs {
    /**
     * Name of the control.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Type of control. Valid values are `Custom` and `Standard`.
     */
    type: pulumi.Input<string>;
}
