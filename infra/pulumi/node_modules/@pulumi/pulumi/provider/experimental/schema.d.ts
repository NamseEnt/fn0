import { ComponentDefinition, TypeDefinition, Dependency } from "./analyzer";
export declare type PropertyType = "string" | "integer" | "number" | "boolean" | "array" | "object";
/**
 * https://www.pulumi.com/docs/iac/using-pulumi/pulumi-packages/schema/#type
 */
export declare type Type = ({
    type: PropertyType;
} | {
    $ref: string;
}) & {
    items?: Type;
    additionalProperties?: Type;
    plain?: boolean;
};
/**
 * https://www.pulumi.com/docs/iac/using-pulumi/pulumi-packages/schema/#property
 */
export declare type Property = Type & {
    description?: string;
};
/**
 * https://www.pulumi.com/docs/iac/using-pulumi/pulumi-packages/schema/#objecttype
 */
export interface ObjectType {
    type: PropertyType;
    description?: string;
    properties?: {
        [key: string]: Property;
    };
    required?: string[];
}
/**
 * https://www.pulumi.com/docs/iac/using-pulumi/pulumi-packages/schema/#complextype
 */
export interface ComplexType extends ObjectType {
    enum?: string[];
}
/**
 * https://www.pulumi.com/docs/iac/using-pulumi/pulumi-packages/schema/#resource
 */
export interface Resource extends ObjectType {
    isComponent?: boolean;
    inputProperties?: {
        [key: string]: Property;
    };
    requiredInputs?: string[];
}
export interface PackageDescriptor {
    name: string;
    version?: string;
    downloadURL?: string;
}
/**
 * https://www.pulumi.com/docs/iac/using-pulumi/pulumi-packages/schema/#package
 */
export interface PackageSpec {
    name: string;
    version?: string;
    description?: string;
    namespace?: string;
    resources: {
        [key: string]: Resource;
    };
    types: {
        [key: string]: ComplexType;
    };
    language?: {
        [key: string]: any;
    };
    dependencies?: PackageDescriptor[];
}
export declare function generateSchema(providerName: string, version: string, description: string, components: Record<string, ComponentDefinition>, typeDefinitions: Record<string, TypeDefinition>, dependencies: Dependency[], namespace?: string): PackageSpec;
