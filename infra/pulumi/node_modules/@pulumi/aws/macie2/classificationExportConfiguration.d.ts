import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage an [Amazon Macie Classification Export Configuration](https://docs.aws.amazon.com/macie/latest/APIReference/classification-export-configuration.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.macie2.Account("example", {});
 * const exampleClassificationExportConfiguration = new aws.macie2.ClassificationExportConfiguration("example", {s3Destination: {
 *     bucketName: exampleAwsS3Bucket.bucket,
 *     keyPrefix: "exampleprefix/",
 *     kmsKeyArn: exampleAwsKmsKey.arn,
 * }}, {
 *     dependsOn: [example],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_macie2_classification_export_configuration` using the region. For example:
 *
 * ```sh
 * $ pulumi import aws:macie2/classificationExportConfiguration:ClassificationExportConfiguration example us-west-2
 * ```
 */
export declare class ClassificationExportConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing ClassificationExportConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClassificationExportConfigurationState, opts?: pulumi.CustomResourceOptions): ClassificationExportConfiguration;
    /**
     * Returns true if the given object is an instance of ClassificationExportConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ClassificationExportConfiguration;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration block for a S3 Destination. Defined below
     */
    readonly s3Destination: pulumi.Output<outputs.macie2.ClassificationExportConfigurationS3Destination>;
    /**
     * Create a ClassificationExportConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ClassificationExportConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ClassificationExportConfiguration resources.
 */
export interface ClassificationExportConfigurationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for a S3 Destination. Defined below
     */
    s3Destination?: pulumi.Input<inputs.macie2.ClassificationExportConfigurationS3Destination>;
}
/**
 * The set of arguments for constructing a ClassificationExportConfiguration resource.
 */
export interface ClassificationExportConfigurationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for a S3 Destination. Defined below
     */
    s3Destination: pulumi.Input<inputs.macie2.ClassificationExportConfigurationS3Destination>;
}
