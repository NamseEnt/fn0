import * as pulumi from "@pulumi/pulumi";
/**
 * This resource can be useful for getting back a set of FIS experiment template IDs.
 *
 * ## Example Usage
 *
 * The following shows outputting a list of all FIS experiment template IDs
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * export = async () => {
 *     const all = await aws.fis.getExperimentTemplates({});
 *     return {
 *         all: all.ids,
 *     };
 * }
 * ```
 *
 * The following shows filtering FIS experiment templates by tag
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.fis.getExperimentTemplates({
 *     tags: {
 *         Name: "example",
 *         Tier: "1",
 *     },
 * });
 * const exampleGetPolicyDocument = example.then(example => aws.iam.getPolicyDocument({
 *     statements: [{
 *         sid: "StartFISExperiment",
 *         effect: "Allow",
 *         actions: ["fis:StartExperiment"],
 *         resources: [
 *             `arn:aws:fis:*:*:experiment-template/${example.ids?.[0]}`,
 *             "arn:aws:fis:*:*:experiment/*",
 *         ],
 *     }],
 * }));
 * ```
 */
export declare function getExperimentTemplates(args?: GetExperimentTemplatesArgs, opts?: pulumi.InvokeOptions): Promise<GetExperimentTemplatesResult>;
/**
 * A collection of arguments for invoking getExperimentTemplates.
 */
export interface GetExperimentTemplatesArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags, each pair of which must exactly match
     * a pair on the desired experiment templates.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getExperimentTemplates.
 */
export interface GetExperimentTemplatesResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * List of all the experiment template ids found.
     */
    readonly ids: string[];
    readonly region: string;
    readonly tags?: {
        [key: string]: string;
    };
}
/**
 * This resource can be useful for getting back a set of FIS experiment template IDs.
 *
 * ## Example Usage
 *
 * The following shows outputting a list of all FIS experiment template IDs
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * export = async () => {
 *     const all = await aws.fis.getExperimentTemplates({});
 *     return {
 *         all: all.ids,
 *     };
 * }
 * ```
 *
 * The following shows filtering FIS experiment templates by tag
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.fis.getExperimentTemplates({
 *     tags: {
 *         Name: "example",
 *         Tier: "1",
 *     },
 * });
 * const exampleGetPolicyDocument = example.then(example => aws.iam.getPolicyDocument({
 *     statements: [{
 *         sid: "StartFISExperiment",
 *         effect: "Allow",
 *         actions: ["fis:StartExperiment"],
 *         resources: [
 *             `arn:aws:fis:*:*:experiment-template/${example.ids?.[0]}`,
 *             "arn:aws:fis:*:*:experiment/*",
 *         ],
 *     }],
 * }));
 * ```
 */
export declare function getExperimentTemplatesOutput(args?: GetExperimentTemplatesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetExperimentTemplatesResult>;
/**
 * A collection of arguments for invoking getExperimentTemplates.
 */
export interface GetExperimentTemplatesOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags, each pair of which must exactly match
     * a pair on the desired experiment templates.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
