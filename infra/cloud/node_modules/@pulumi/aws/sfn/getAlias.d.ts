import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS SFN (Step Functions) State Machine Alias.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.sfn.getAlias({
 *     name: "my_sfn_alias",
 *     statemachineArn: sfnTest.arn,
 * });
 * ```
 */
export declare function getAlias(args: GetAliasArgs, opts?: pulumi.InvokeOptions): Promise<GetAliasResult>;
/**
 * A collection of arguments for invoking getAlias.
 */
export interface GetAliasArgs {
    /**
     * Description of state machine alias.
     */
    description?: string;
    /**
     * Name of the State Machine alias.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * ARN of the State Machine.
     */
    statemachineArn: string;
}
/**
 * A collection of values returned by getAlias.
 */
export interface GetAliasResult {
    /**
     * ARN identifying the State Machine alias.
     */
    readonly arn: string;
    /**
     * Date the state machine Alias was created.
     */
    readonly creationDate: string;
    /**
     * Description of state machine alias.
     */
    readonly description?: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
    readonly region: string;
    /**
     * Routing Configuration of state machine alias
     */
    readonly routingConfigurations: outputs.sfn.GetAliasRoutingConfiguration[];
    readonly statemachineArn: string;
}
/**
 * Data source for managing an AWS SFN (Step Functions) State Machine Alias.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.sfn.getAlias({
 *     name: "my_sfn_alias",
 *     statemachineArn: sfnTest.arn,
 * });
 * ```
 */
export declare function getAliasOutput(args: GetAliasOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetAliasResult>;
/**
 * A collection of arguments for invoking getAlias.
 */
export interface GetAliasOutputArgs {
    /**
     * Description of state machine alias.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the State Machine alias.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the State Machine.
     */
    statemachineArn: pulumi.Input<string>;
}
