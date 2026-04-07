import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS Elemental MediaLive Input.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.medialive.getInput({
 *     id: exampleAwsMedialiveInput.id,
 * });
 * ```
 */
export declare function getInput(args: GetInputArgs, opts?: pulumi.InvokeOptions): Promise<GetInputResult>;
/**
 * A collection of arguments for invoking getInput.
 */
export interface GetInputArgs {
    /**
     * The ID of the Input.
     */
    id: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getInput.
 */
export interface GetInputResult {
    /**
     * ARN of the Input.
     */
    readonly arn: string;
    /**
     * Channels attached to Input.
     */
    readonly attachedChannels: string[];
    readonly destinations: outputs.medialive.GetInputDestination[];
    readonly id: string;
    /**
     * The input class.
     */
    readonly inputClass: string;
    /**
     * Settings for the devices.
     */
    readonly inputDevices: outputs.medialive.GetInputInputDevice[];
    /**
     * A list of IDs for all Inputs which are partners of this one.
     */
    readonly inputPartnerIds: string[];
    /**
     * Source type of the input.
     */
    readonly inputSourceType: string;
    /**
     * A list of the MediaConnect Flows.
     */
    readonly mediaConnectFlows: outputs.medialive.GetInputMediaConnectFlow[];
    /**
     * Name of the input.
     */
    readonly name: string;
    readonly region: string;
    /**
     * The ARN of the role this input assumes during and after creation.
     */
    readonly roleArn: string;
    /**
     * List of input security groups.
     */
    readonly securityGroups: string[];
    /**
     * The source URLs for a PULL-type input.
     */
    readonly sources: outputs.medialive.GetInputSource[];
    /**
     * The state of the input.
     */
    readonly state: string;
    /**
     * A map of tags assigned to the Input.
     */
    readonly tags: {
        [key: string]: string;
    };
    /**
     * The type of the input.
     */
    readonly type: string;
}
/**
 * Data source for managing an AWS Elemental MediaLive Input.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.medialive.getInput({
 *     id: exampleAwsMedialiveInput.id,
 * });
 * ```
 */
export declare function getInputOutput(args: GetInputOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetInputResult>;
/**
 * A collection of arguments for invoking getInput.
 */
export interface GetInputOutputArgs {
    /**
     * The ID of the Input.
     */
    id: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
