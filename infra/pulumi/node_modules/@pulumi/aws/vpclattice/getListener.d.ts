import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS VPC Lattice Listener.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.vpclattice.getListener({});
 * ```
 */
export declare function getListener(args: GetListenerArgs, opts?: pulumi.InvokeOptions): Promise<GetListenerResult>;
/**
 * A collection of arguments for invoking getListener.
 */
export interface GetListenerArgs {
    /**
     * ID or Amazon Resource Name (ARN) of the listener
     */
    listenerIdentifier: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * ID or Amazon Resource Name (ARN) of the service network
     */
    serviceIdentifier: string;
    /**
     * List of tags associated with the listener.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getListener.
 */
export interface GetListenerResult {
    /**
     * ARN of the listener.
     */
    readonly arn: string;
    /**
     * The date and time that the listener was created.
     */
    readonly createdAt: string;
    /**
     * The actions for the default listener rule.
     */
    readonly defaultActions: outputs.vpclattice.GetListenerDefaultAction[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * The date and time the listener was last updated.
     */
    readonly lastUpdatedAt: string;
    /**
     * The ID of the listener.
     */
    readonly listenerId: string;
    readonly listenerIdentifier: string;
    /**
     * The name of the listener.
     */
    readonly name: string;
    /**
     * The listener port.
     */
    readonly port: number;
    /**
     * The listener protocol. Either `HTTPS` or `HTTP`.
     */
    readonly protocol: string;
    readonly region: string;
    /**
     * The ARN of the service.
     */
    readonly serviceArn: string;
    /**
     * The ID of the service.
     */
    readonly serviceId: string;
    readonly serviceIdentifier: string;
    /**
     * List of tags associated with the listener.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Data source for managing an AWS VPC Lattice Listener.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.vpclattice.getListener({});
 * ```
 */
export declare function getListenerOutput(args: GetListenerOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetListenerResult>;
/**
 * A collection of arguments for invoking getListener.
 */
export interface GetListenerOutputArgs {
    /**
     * ID or Amazon Resource Name (ARN) of the listener
     */
    listenerIdentifier: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID or Amazon Resource Name (ARN) of the service network
     */
    serviceIdentifier: pulumi.Input<string>;
    /**
     * List of tags associated with the listener.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
