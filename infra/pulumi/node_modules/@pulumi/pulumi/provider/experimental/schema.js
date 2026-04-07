"use strict";
// Copyright 2025-2025, Pulumi Corporation.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
Object.defineProperty(exports, "__esModule", { value: true });
function generateSchema(providerName, version, description, components, typeDefinitions, dependencies, namespace) {
    const result = {
        name: providerName,
        version: version,
        description: description,
        namespace: namespace,
        resources: {},
        types: {},
        language: {
            nodejs: {
                respectSchemaVersion: true,
            },
            python: {
                respectSchemaVersion: true,
            },
            csharp: {
                respectSchemaVersion: true,
            },
            java: {
                respectSchemaVersion: true,
            },
            go: {
                respectSchemaVersion: true,
            },
        },
        dependencies,
    };
    for (const [name, component] of Object.entries(components)) {
        result.resources[`${providerName}:index:${name}`] = {
            type: "object",
            isComponent: true,
            inputProperties: component.inputs,
            requiredInputs: required(component.inputs),
            properties: component.outputs,
            required: required(component.outputs),
            description: component.description,
        };
    }
    for (const [name, type] of Object.entries(typeDefinitions)) {
        result.types[`${providerName}:index:${name}`] = {
            type: "object",
            properties: type.properties,
            required: required(type.properties),
        };
    }
    return result;
}
exports.generateSchema = generateSchema;
function required(properties) {
    return Object.entries(properties)
        .filter(([_, def]) => !def.optional)
        .map(([propName, _]) => propName)
        .sort();
}
//# sourceMappingURL=schema.js.map