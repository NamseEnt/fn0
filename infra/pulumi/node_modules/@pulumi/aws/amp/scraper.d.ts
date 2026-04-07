import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Import
 *
 * Using `pulumi import`, import the Managed Scraper using its identifier.
 * For example:
 *
 * ```sh
 * $ pulumi import aws:amp/scraper:Scraper example s-0123abc-0000-0123-a000-000000000000
 * ```
 */
export declare class Scraper extends pulumi.CustomResource {
    /**
     * Get an existing Scraper resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ScraperState, opts?: pulumi.CustomResourceOptions): Scraper;
    /**
     * Returns true if the given object is an instance of Scraper.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Scraper;
    /**
     * a name to associate with the managed scraper. This is for your use, and does not need to be unique.
     */
    readonly alias: pulumi.Output<string | undefined>;
    /**
     * The Amazon Resource Name (ARN) of the new scraper.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Configuration block for the managed scraper to send metrics to. See `destination`.
     */
    readonly destination: pulumi.Output<outputs.amp.ScraperDestination | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the IAM role that provides permissions for the scraper to discover, collect, and produce metrics
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * Configuration block to enable writing to an Amazon Managed Service for Prometheus workspace in a different account. See `roleConfiguration` below.
     */
    readonly roleConfiguration: pulumi.Output<outputs.amp.ScraperRoleConfiguration | undefined>;
    /**
     * The configuration file to use in the new scraper. For more information, see [Scraper configuration](https://docs.aws.amazon.com/prometheus/latest/userguide/AMP-collector-how-to.html#AMP-collector-configuration).
     */
    readonly scrapeConfiguration: pulumi.Output<string>;
    /**
     * Configuration block to specify where the managed scraper will collect metrics from. See `source`.
     *
     * The following arguments are optional:
     */
    readonly source: pulumi.Output<outputs.amp.ScraperSource | undefined>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.amp.ScraperTimeouts | undefined>;
    /**
     * Create a Scraper resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ScraperArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Scraper resources.
 */
export interface ScraperState {
    /**
     * a name to associate with the managed scraper. This is for your use, and does not need to be unique.
     */
    alias?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the new scraper.
     */
    arn?: pulumi.Input<string>;
    /**
     * Configuration block for the managed scraper to send metrics to. See `destination`.
     */
    destination?: pulumi.Input<inputs.amp.ScraperDestination>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the IAM role that provides permissions for the scraper to discover, collect, and produce metrics
     */
    roleArn?: pulumi.Input<string>;
    /**
     * Configuration block to enable writing to an Amazon Managed Service for Prometheus workspace in a different account. See `roleConfiguration` below.
     */
    roleConfiguration?: pulumi.Input<inputs.amp.ScraperRoleConfiguration>;
    /**
     * The configuration file to use in the new scraper. For more information, see [Scraper configuration](https://docs.aws.amazon.com/prometheus/latest/userguide/AMP-collector-how-to.html#AMP-collector-configuration).
     */
    scrapeConfiguration?: pulumi.Input<string>;
    /**
     * Configuration block to specify where the managed scraper will collect metrics from. See `source`.
     *
     * The following arguments are optional:
     */
    source?: pulumi.Input<inputs.amp.ScraperSource>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.amp.ScraperTimeouts>;
}
/**
 * The set of arguments for constructing a Scraper resource.
 */
export interface ScraperArgs {
    /**
     * a name to associate with the managed scraper. This is for your use, and does not need to be unique.
     */
    alias?: pulumi.Input<string>;
    /**
     * Configuration block for the managed scraper to send metrics to. See `destination`.
     */
    destination?: pulumi.Input<inputs.amp.ScraperDestination>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block to enable writing to an Amazon Managed Service for Prometheus workspace in a different account. See `roleConfiguration` below.
     */
    roleConfiguration?: pulumi.Input<inputs.amp.ScraperRoleConfiguration>;
    /**
     * The configuration file to use in the new scraper. For more information, see [Scraper configuration](https://docs.aws.amazon.com/prometheus/latest/userguide/AMP-collector-how-to.html#AMP-collector-configuration).
     */
    scrapeConfiguration: pulumi.Input<string>;
    /**
     * Configuration block to specify where the managed scraper will collect metrics from. See `source`.
     *
     * The following arguments are optional:
     */
    source?: pulumi.Input<inputs.amp.ScraperSource>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.amp.ScraperTimeouts>;
}
