import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Shield Application Layer Automatic Response for automatic DDoS mitigation.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getRegion({});
 * const currentGetCallerIdentity = aws.getCallerIdentity({});
 * const currentGetPartition = aws.getPartition({});
 * const config = new pulumi.Config();
 * // The Cloudfront Distribution on which to enable the Application Layer Automatic Response.
 * const distributionId = config.require("distributionId");
 * const example = new aws.shield.ApplicationLayerAutomaticResponse("example", {
 *     resourceArn: Promise.all([currentGetPartition, currentGetCallerIdentity]).then(([currentGetPartition, currentGetCallerIdentity]) => `arn:${currentGetPartition.partition}:cloudfront:${currentGetCallerIdentity.accountId}:distribution/${distributionId}`),
 *     action: "COUNT",
 * });
 * ```
 */
export declare class ApplicationLayerAutomaticResponse extends pulumi.CustomResource {
    /**
     * Get an existing ApplicationLayerAutomaticResponse resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ApplicationLayerAutomaticResponseState, opts?: pulumi.CustomResourceOptions): ApplicationLayerAutomaticResponse;
    /**
     * Returns true if the given object is an instance of ApplicationLayerAutomaticResponse.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ApplicationLayerAutomaticResponse;
    /**
     * One of `COUNT` or `BLOCK`
     */
    readonly action: pulumi.Output<string>;
    /**
     * ARN of the resource to protect (Cloudfront Distributions and ALBs only at this time).
     */
    readonly resourceArn: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.shield.ApplicationLayerAutomaticResponseTimeouts | undefined>;
    /**
     * Create a ApplicationLayerAutomaticResponse resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ApplicationLayerAutomaticResponseArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ApplicationLayerAutomaticResponse resources.
 */
export interface ApplicationLayerAutomaticResponseState {
    /**
     * One of `COUNT` or `BLOCK`
     */
    action?: pulumi.Input<string>;
    /**
     * ARN of the resource to protect (Cloudfront Distributions and ALBs only at this time).
     */
    resourceArn?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.shield.ApplicationLayerAutomaticResponseTimeouts>;
}
/**
 * The set of arguments for constructing a ApplicationLayerAutomaticResponse resource.
 */
export interface ApplicationLayerAutomaticResponseArgs {
    /**
     * One of `COUNT` or `BLOCK`
     */
    action: pulumi.Input<string>;
    /**
     * ARN of the resource to protect (Cloudfront Distributions and ALBs only at this time).
     */
    resourceArn: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.shield.ApplicationLayerAutomaticResponseTimeouts>;
}
