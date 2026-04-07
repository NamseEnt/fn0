import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage an S3 Object Lambda Access Point.
 * An Object Lambda access point is associated with exactly one standard access point and thus one Amazon S3 bucket.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.s3.Bucket("example", {bucket: "example"});
 * const exampleAccessPoint = new aws.s3.AccessPoint("example", {
 *     bucket: example.id,
 *     name: "example",
 * });
 * const exampleObjectLambdaAccessPoint = new aws.s3control.ObjectLambdaAccessPoint("example", {
 *     name: "example",
 *     configuration: {
 *         supportingAccessPoint: exampleAccessPoint.arn,
 *         transformationConfigurations: [{
 *             actions: ["GetObject"],
 *             contentTransformation: {
 *                 awsLambda: {
 *                     functionArn: exampleAwsLambdaFunction.arn,
 *                 },
 *             },
 *         }],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Object Lambda Access Points using the `account_id` and `name`, separated by a colon (`:`). For example:
 *
 * ```sh
 * $ pulumi import aws:s3control/objectLambdaAccessPoint:ObjectLambdaAccessPoint example 123456789012:example
 * ```
 */
export declare class ObjectLambdaAccessPoint extends pulumi.CustomResource {
    /**
     * Get an existing ObjectLambdaAccessPoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ObjectLambdaAccessPointState, opts?: pulumi.CustomResourceOptions): ObjectLambdaAccessPoint;
    /**
     * Returns true if the given object is an instance of ObjectLambdaAccessPoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ObjectLambdaAccessPoint;
    /**
     * The AWS account ID for the owner of the bucket for which you want to create an Object Lambda Access Point. Defaults to automatically determined account ID of the AWS provider.
     */
    readonly accountId: pulumi.Output<string>;
    /**
     * Alias for the S3 Object Lambda Access Point.
     */
    readonly alias: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the Object Lambda Access Point.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A configuration block containing details about the Object Lambda Access Point. See Configuration below for more details.
     */
    readonly configuration: pulumi.Output<outputs.s3control.ObjectLambdaAccessPointConfiguration>;
    /**
     * The name for this Object Lambda Access Point.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a ObjectLambdaAccessPoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ObjectLambdaAccessPointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ObjectLambdaAccessPoint resources.
 */
export interface ObjectLambdaAccessPointState {
    /**
     * The AWS account ID for the owner of the bucket for which you want to create an Object Lambda Access Point. Defaults to automatically determined account ID of the AWS provider.
     */
    accountId?: pulumi.Input<string>;
    /**
     * Alias for the S3 Object Lambda Access Point.
     */
    alias?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the Object Lambda Access Point.
     */
    arn?: pulumi.Input<string>;
    /**
     * A configuration block containing details about the Object Lambda Access Point. See Configuration below for more details.
     */
    configuration?: pulumi.Input<inputs.s3control.ObjectLambdaAccessPointConfiguration>;
    /**
     * The name for this Object Lambda Access Point.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ObjectLambdaAccessPoint resource.
 */
export interface ObjectLambdaAccessPointArgs {
    /**
     * The AWS account ID for the owner of the bucket for which you want to create an Object Lambda Access Point. Defaults to automatically determined account ID of the AWS provider.
     */
    accountId?: pulumi.Input<string>;
    /**
     * A configuration block containing details about the Object Lambda Access Point. See Configuration below for more details.
     */
    configuration: pulumi.Input<inputs.s3control.ObjectLambdaAccessPointConfiguration>;
    /**
     * The name for this Object Lambda Access Point.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
