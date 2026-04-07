import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a Glue Classifier resource.
 *
 * > **NOTE:** It is only valid to create one type of classifier (CSV, grok, JSON, or XML). Changing classifier types will recreate the classifier.
 *
 * ## Example Usage
 *
 * ### CSV Classifier
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Classifier("example", {
 *     name: "example",
 *     csvClassifier: {
 *         allowSingleColumn: false,
 *         containsHeader: "PRESENT",
 *         delimiter: ",",
 *         disableValueTrimming: false,
 *         headers: [
 *             "example1",
 *             "example2",
 *         ],
 *         quoteSymbol: "'",
 *     },
 * });
 * ```
 *
 * ### Grok Classifier
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Classifier("example", {
 *     name: "example",
 *     grokClassifier: {
 *         classification: "example",
 *         grokPattern: "example",
 *     },
 * });
 * ```
 *
 * ### JSON Classifier
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Classifier("example", {
 *     name: "example",
 *     jsonClassifier: {
 *         jsonPath: "example",
 *     },
 * });
 * ```
 *
 * ### XML Classifier
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.glue.Classifier("example", {
 *     name: "example",
 *     xmlClassifier: {
 *         classification: "example",
 *         rowTag: "example",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Glue Classifiers using their name. For example:
 *
 * ```sh
 * $ pulumi import aws:glue/classifier:Classifier MyClassifier MyClassifier
 * ```
 */
export declare class Classifier extends pulumi.CustomResource {
    /**
     * Get an existing Classifier resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClassifierState, opts?: pulumi.CustomResourceOptions): Classifier;
    /**
     * Returns true if the given object is an instance of Classifier.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Classifier;
    /**
     * A classifier for CSV content. Defined below.
     */
    readonly csvClassifier: pulumi.Output<outputs.glue.ClassifierCsvClassifier | undefined>;
    /**
     * A classifier that uses grok patterns. Defined below.
     */
    readonly grokClassifier: pulumi.Output<outputs.glue.ClassifierGrokClassifier | undefined>;
    /**
     * A classifier for JSON content. Defined below.
     */
    readonly jsonClassifier: pulumi.Output<outputs.glue.ClassifierJsonClassifier | undefined>;
    /**
     * The name of the classifier.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A classifier for XML content. Defined below.
     */
    readonly xmlClassifier: pulumi.Output<outputs.glue.ClassifierXmlClassifier | undefined>;
    /**
     * Create a Classifier resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ClassifierArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Classifier resources.
 */
export interface ClassifierState {
    /**
     * A classifier for CSV content. Defined below.
     */
    csvClassifier?: pulumi.Input<inputs.glue.ClassifierCsvClassifier>;
    /**
     * A classifier that uses grok patterns. Defined below.
     */
    grokClassifier?: pulumi.Input<inputs.glue.ClassifierGrokClassifier>;
    /**
     * A classifier for JSON content. Defined below.
     */
    jsonClassifier?: pulumi.Input<inputs.glue.ClassifierJsonClassifier>;
    /**
     * The name of the classifier.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A classifier for XML content. Defined below.
     */
    xmlClassifier?: pulumi.Input<inputs.glue.ClassifierXmlClassifier>;
}
/**
 * The set of arguments for constructing a Classifier resource.
 */
export interface ClassifierArgs {
    /**
     * A classifier for CSV content. Defined below.
     */
    csvClassifier?: pulumi.Input<inputs.glue.ClassifierCsvClassifier>;
    /**
     * A classifier that uses grok patterns. Defined below.
     */
    grokClassifier?: pulumi.Input<inputs.glue.ClassifierGrokClassifier>;
    /**
     * A classifier for JSON content. Defined below.
     */
    jsonClassifier?: pulumi.Input<inputs.glue.ClassifierJsonClassifier>;
    /**
     * The name of the classifier.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A classifier for XML content. Defined below.
     */
    xmlClassifier?: pulumi.Input<inputs.glue.ClassifierXmlClassifier>;
}
