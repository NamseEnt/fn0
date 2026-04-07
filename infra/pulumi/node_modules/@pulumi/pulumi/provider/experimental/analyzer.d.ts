import typescript from "typescript";
export declare type PropertyType = "string" | "integer" | "number" | "boolean" | "array" | "object";
export declare type PropertyDefinition = ({
    type: PropertyType;
} | {
    $ref: string;
}) & {
    description?: string;
    optional?: boolean;
    plain?: boolean;
    additionalProperties?: PropertyDefinition;
    items?: PropertyDefinition;
};
export declare type ComponentDefinition = {
    name: string;
    description?: string;
    inputs: Record<string, PropertyDefinition>;
    outputs: Record<string, PropertyDefinition>;
};
export declare type TypeDefinition = {
    name: string;
    properties: Record<string, PropertyDefinition>;
    description?: string;
};
export declare type Dependency = {
    name: string;
    version?: string;
    downloadURL?: string;
    parameterization?: {
        name: string;
        version: string;
        value: string;
    };
};
export declare type AnalyzeResult = {
    components: Record<string, ComponentDefinition>;
    typeDefinitions: Record<string, TypeDefinition>;
    dependencies: Dependency[];
};
export declare enum InputOutput {
    Neither = 0,
    Input = 1,
    Output = 2
}
export declare class Analyzer {
    private dir;
    private packageJSON;
    private providerName;
    private checker;
    private program;
    private components;
    private typeDefinitions;
    private dependencies;
    private docStrings;
    private componentNames;
    private programFiles;
    constructor(dir: string, name: string, packageJSON: Record<string, any>, componentNames: Set<string>);
    analyze(): AnalyzeResult;
    private findProgramEntryPoint;
    private collectImportedFiles;
    private analyzeComponent;
    private isPulumiComponent;
    private analyzeSymbols;
    private analyzeSymbol;
    private analyzeType;
    unwrapTypeReference(context: {
        component: string;
        property: string;
        inputOutput: InputOutput;
        typeName?: string;
    }, type: typescript.Type): typescript.Type;
    private formatErrorContext;
    /**
     * Gets the Pulumi resource type information for a resource reference.
     * A strong assumption is that the referenced resource class is in a package installed to node_modules
     * and contains a standard Pulumi-generated SDK compiled into JavaScript. To find the resource type token,
     * the function will attempt to find the JavaScript module file that contains the resource class, and then
     * extract the type from the __pulumiType property of the resource class. To find the package version,
     * the function will attempt to read the package.json file in the root directory of the referenced package.
     * @returns Object containing packageName, packageVersion, and pulumiType token
     * @throws Error if the resource type cannot be determined with detailed context information
     */
    private getResourceType;
}
