import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing a QuickSight Data Set.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.quicksight.getDataSet({
 *     dataSetId: "example-id",
 * });
 * ```
 */
export declare function getDataSet(args: GetDataSetArgs, opts?: pulumi.InvokeOptions): Promise<GetDataSetResult>;
/**
 * A collection of arguments for invoking getDataSet.
 */
export interface GetDataSetArgs {
    awsAccountId?: string;
    /**
     * Identifier for the data set.
     */
    dataSetId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getDataSet.
 */
export interface GetDataSetResult {
    readonly arn: string;
    readonly awsAccountId: string;
    readonly columnGroups: outputs.quicksight.GetDataSetColumnGroup[];
    readonly columnLevelPermissionRules: outputs.quicksight.GetDataSetColumnLevelPermissionRule[];
    readonly dataSetId: string;
    readonly dataSetUsageConfigurations: outputs.quicksight.GetDataSetDataSetUsageConfiguration[];
    readonly fieldFolders: outputs.quicksight.GetDataSetFieldFolder[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly importMode: string;
    readonly logicalTableMaps: outputs.quicksight.GetDataSetLogicalTableMap[];
    readonly name: string;
    readonly permissions: outputs.quicksight.GetDataSetPermission[];
    readonly physicalTableMaps: outputs.quicksight.GetDataSetPhysicalTableMap[];
    readonly region: string;
    readonly rowLevelPermissionDataSets: outputs.quicksight.GetDataSetRowLevelPermissionDataSet[];
    readonly rowLevelPermissionTagConfigurations: outputs.quicksight.GetDataSetRowLevelPermissionTagConfiguration[];
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Data source for managing a QuickSight Data Set.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.quicksight.getDataSet({
 *     dataSetId: "example-id",
 * });
 * ```
 */
export declare function getDataSetOutput(args: GetDataSetOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDataSetResult>;
/**
 * A collection of arguments for invoking getDataSet.
 */
export interface GetDataSetOutputArgs {
    awsAccountId?: pulumi.Input<string>;
    /**
     * Identifier for the data set.
     */
    dataSetId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
