import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Opensearch Package.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const myOpensearchPackages = new aws.s3.Bucket("my_opensearch_packages", {bucket: "my-opensearch-packages"});
 * const example = new aws.s3.BucketObjectv2("example", {
 *     bucket: myOpensearchPackages.bucket,
 *     key: "example.txt",
 *     source: new pulumi.asset.FileAsset("./example.txt"),
 *     etag: std.filemd5({
 *         input: "./example.txt",
 *     }).then(invoke => invoke.result),
 * });
 * const examplePackage = new aws.opensearch.Package("example", {
 *     packageName: "example-txt",
 *     packageSource: {
 *         s3BucketName: myOpensearchPackages.bucket,
 *         s3Key: example.key,
 *     },
 *     packageType: "TXT-DICTIONARY",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AWS Opensearch Packages using the Package ID. For example:
 *
 * ```sh
 * $ pulumi import aws:opensearch/package:Package example package-id
 * ```
 */
export declare class Package extends pulumi.CustomResource {
    /**
     * Get an existing Package resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PackageState, opts?: pulumi.CustomResourceOptions): Package;
    /**
     * Returns true if the given object is an instance of Package.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Package;
    /**
     * The current version of the package.
     */
    readonly availablePackageVersion: pulumi.Output<string>;
    /**
     * Engine version that the package is compatible with. This argument is required and only valid when `packageType` is `ZIP-PLUGIN`. Format: `OpenSearch_X.Y` or `Elasticsearch_X.Y`, where `X` and `Y` are the major and minor version numbers, respectively.
     */
    readonly engineVersion: pulumi.Output<string | undefined>;
    /**
     * Description of the package.
     */
    readonly packageDescription: pulumi.Output<string | undefined>;
    readonly packageId: pulumi.Output<string>;
    /**
     * Unique name for the package.
     */
    readonly packageName: pulumi.Output<string>;
    /**
     * Configuration block for the package source options.
     */
    readonly packageSource: pulumi.Output<outputs.opensearch.PackagePackageSource>;
    /**
     * The type of package. Valid values are `TXT-DICTIONARY`, `ZIP-PLUGIN`, `PACKAGE-LICENSE` and `PACKAGE-CONFIG`.
     */
    readonly packageType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a Package resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PackageArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Package resources.
 */
export interface PackageState {
    /**
     * The current version of the package.
     */
    availablePackageVersion?: pulumi.Input<string>;
    /**
     * Engine version that the package is compatible with. This argument is required and only valid when `packageType` is `ZIP-PLUGIN`. Format: `OpenSearch_X.Y` or `Elasticsearch_X.Y`, where `X` and `Y` are the major and minor version numbers, respectively.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * Description of the package.
     */
    packageDescription?: pulumi.Input<string>;
    packageId?: pulumi.Input<string>;
    /**
     * Unique name for the package.
     */
    packageName?: pulumi.Input<string>;
    /**
     * Configuration block for the package source options.
     */
    packageSource?: pulumi.Input<inputs.opensearch.PackagePackageSource>;
    /**
     * The type of package. Valid values are `TXT-DICTIONARY`, `ZIP-PLUGIN`, `PACKAGE-LICENSE` and `PACKAGE-CONFIG`.
     */
    packageType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Package resource.
 */
export interface PackageArgs {
    /**
     * Engine version that the package is compatible with. This argument is required and only valid when `packageType` is `ZIP-PLUGIN`. Format: `OpenSearch_X.Y` or `Elasticsearch_X.Y`, where `X` and `Y` are the major and minor version numbers, respectively.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * Description of the package.
     */
    packageDescription?: pulumi.Input<string>;
    /**
     * Unique name for the package.
     */
    packageName: pulumi.Input<string>;
    /**
     * Configuration block for the package source options.
     */
    packageSource: pulumi.Input<inputs.opensearch.PackagePackageSource>;
    /**
     * The type of package. Valid values are `TXT-DICTIONARY`, `ZIP-PLUGIN`, `PACKAGE-LICENSE` and `PACKAGE-CONFIG`.
     */
    packageType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
