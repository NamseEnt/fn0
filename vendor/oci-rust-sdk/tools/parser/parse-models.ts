#!/usr/bin/env ts-node
/**
 * TypeScript Model Parser
 * Parses TypeScript interfaces from OCI SDK and extracts field metadata
 */

import { Project, InterfaceDeclaration, PropertySignature, EnumDeclaration, TypeAliasDeclaration } from 'ts-morph';
import * as path from 'path';
import * as fs from 'fs';
import { fileURLToPath } from 'url';

// ES module equivalent of __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

interface ParsedField {
  name: string;          // camelCase from TypeScript
  rustName: string;      // snake_case for Rust
  tsType: string;        // TypeScript type as string
  isRequired: boolean;   // !field.hasQuestionToken()
  documentation: string;
}

interface ParsedModel {
  name: string;          // PascalCase
  fileName: string;      // kebab-case
  kind: 'interface' | 'enum' | 'type-alias';
  sourceDir: 'model' | 'request' | 'response';  // Track which directory this came from
  documentation: string;
  fields?: ParsedField[];
  variants?: { name: string; value: string }[];
  discriminator?: string;
  baseType?: string;
}

// Rust keywords that need r# prefix
const RUST_KEYWORDS = [
  'type', 'impl', 'fn', 'let', 'mut', 'const', 'static',
  'trait', 'struct', 'enum', 'match', 'if', 'else', 'while',
  'for', 'loop', 'return', 'break', 'continue', 'as', 'use',
  'mod', 'pub', 'crate', 'super', 'self', 'Self', 'async',
  'await', 'dyn', 'move', 'ref', 'where', 'unsafe', 'extern'
];

function toSnakeCase(str: string): string {
  if (!str) return str;

  let result = '';

  for (let i = 0; i < str.length; i++) {
    const char = str[i];
    const nextChar = i < str.length - 1 ? str[i + 1] : null;

    const isLower = char >= 'a' && char <= 'z';
    const isDigit = char >= '0' && char <= '9';
    const isNextUpper = nextChar && nextChar >= 'A' && nextChar <= 'Z';

    result += char.toLowerCase();

    // Add underscore after lowercase/digit when followed by uppercase
    // Examples: displayName -> display_name, memoryInGBs -> memory_in_gbs, ABcdEfG -> abcd_ef_g
    if ((isLower || isDigit) && isNextUpper) {
      result += '_';
    }
  }

  return result;
}

function escapeRustKeyword(rustName: string): string {
  if (RUST_KEYWORDS.includes(rustName)) {
    return `r#${rustName}`;
  }
  return rustName;
}

function parseInterface(interfaceDecl: InterfaceDeclaration, sourceDir: 'model' | 'request' | 'response'): ParsedModel {
  const name = interfaceDecl.getName();
  const fileName = toSnakeCase(name);
  const documentation = interfaceDecl.getJsDocs()[0]?.getDescription() || '';

  const properties = interfaceDecl.getProperties();
  const fields: ParsedField[] = properties.map(prop => {
    const fieldName = prop.getName().replace(/^"(.*)"$/, '$1'); // Remove quotes
    const isRequired = !prop.hasQuestionToken();
    const typeNode = prop.getTypeNode();
    let tsType = typeNode ? typeNode.getText() : prop.getType().getText();

    // Clean up type references
    tsType = tsType.replace(/import\(.*?\)\./g, '').replace(/model\./g, '');

    const fieldDoc = prop.getJsDocs()[0]?.getDescription() || '';

    // Generate Rust field name with keyword escaping
    const baseRustName = toSnakeCase(fieldName);
    const rustName = escapeRustKeyword(baseRustName);

    return {
      name: fieldName,
      rustName,
      tsType,
      isRequired,
      documentation: fieldDoc.trim(),
    };
  });

  // Detect discriminator field (common pattern for polymorphic types)
  let discriminator: string | undefined;
  const extendsTypes = interfaceDecl.getExtends();
  if (extendsTypes.length > 0) {
    // This might be a polymorphic type variant
    const baseType = extendsTypes[0].getText();
    return {
      name,
      fileName,
      kind: 'interface',
      sourceDir,
      documentation,
      fields,
      baseType,
    };
  }

  // Check if this has a single discriminator field (base type pattern)
  if (fields.length === 1 && fields[0].tsType === 'string') {
    discriminator = fields[0].name;
  }

  return {
    name,
    fileName,
    kind: 'interface',
    sourceDir,
    documentation,
    fields,
    discriminator,
  };
}

function parseEnum(enumDecl: EnumDeclaration, sourceDir: 'model' | 'request' | 'response'): ParsedModel {
  const name = enumDecl.getName();
  const fileName = toSnakeCase(name);
  const documentation = enumDecl.getJsDocs()[0]?.getDescription() || '';

  const variants = enumDecl.getMembers().map(member => ({
    name: member.getName(),
    value: member.getValue()?.toString() || member.getName(),
  }));

  return {
    name,
    fileName,
    kind: 'enum',
    sourceDir,
    documentation,
    variants,
  };
}

async function parseModels(serviceName: string): Promise<ParsedModel[]> {
  const sdkPath = path.join(__dirname, '../../oci-typescript-sdk');
  const serviceLibPath = path.join(sdkPath, 'lib', serviceName, 'lib');
  const modelPath = path.join(serviceLibPath, 'model');
  const requestPath = path.join(serviceLibPath, 'request');
  const responsePath = path.join(serviceLibPath, 'response');

  if (!fs.existsSync(modelPath)) {
    console.error(`Error: Model path not found: ${modelPath}`);
    return [];
  }

  const models: ParsedModel[] = [];
  const namespaceEnumMap = new Map<string, string>(); // "Shape.BaselineOcpuUtilizations" → "ShapeBaselineOcpuUtilizations"

  // Helper function to parse files from a directory
  const parseDirectory = (dirPath: string, sourceDir: 'model' | 'request' | 'response') => {
    if (!fs.existsSync(dirPath)) {
      return;
    }

    console.error(`Parsing ${sourceDir}s from ${dirPath}`);

    const project = new Project({
      tsConfigFilePath: path.join(__dirname, 'tsconfig.json'),
      skipAddingFilesFromTsConfig: true,
    });

    const files = fs.readdirSync(dirPath)
      .filter(f => f.endsWith('.ts') && f !== 'index.ts')
      .map(f => path.join(dirPath, f));

    project.addSourceFilesAtPaths(files);

    for (const sourceFile of project.getSourceFiles()) {
      // Parse interfaces
      for (const interfaceDecl of sourceFile.getInterfaces()) {
        // Skip exported namespaces (these are enum containers)
        if (interfaceDecl.getName().includes('.')) continue;

        const model = parseInterface(interfaceDecl, sourceDir);
        models.push(model);
      }

      // Parse enums
      for (const enumDecl of sourceFile.getEnums()) {
        const model = parseEnum(enumDecl, sourceDir);
        models.push(model);
      }

      // Parse namespace enums (e.g., Instance.LifecycleState or ListContainerInstancesRequest.SortBy)
      for (const namespace of sourceFile.getModules()) {
        for (const enumDecl of namespace.getEnums()) {
          const parentName = namespace.getName();
          const enumName = enumDecl.getName();
          const fullName = `${parentName}${enumName}`; // No underscore - PascalCase
          const dottedRef = `${parentName}.${enumName}`; // TypeScript reference

          const model = parseEnum(enumDecl, sourceDir);
          model.name = fullName;
          // Convert to snake_case for filename: ShapeBaselineOcpuUtilizations → shape_baseline_ocpu_utilizations
          model.fileName = toSnakeCase(fullName);

          namespaceEnumMap.set(dottedRef, fullName);
          models.push(model);
        }
      }
    }
  };

  // Parse model, request, and response directories
  parseDirectory(modelPath, 'model');
  parseDirectory(requestPath, 'request');
  parseDirectory(responsePath, 'response');

  // Replace all dotted namespace enum references in field types
  for (const model of models) {
    if (model.fields) {
      for (const field of model.fields) {
        let tsType = field.tsType;

        // Replace all namespace enum references
        for (const [dottedRef, flatName] of namespaceEnumMap.entries()) {
          // Match the dotted reference (e.g., "Shape.BaselineOcpuUtilizations")
          // Use word boundary to avoid partial matches
          const regex = new RegExp(dottedRef.replace('.', '\\.'), 'g');
          tsType = tsType.replace(regex, flatName);
        }

        field.tsType = tsType;
      }
    }
  }

  console.error(`Parsed ${models.length} models (${models.filter(m => m.kind === 'interface').length} interfaces, ${models.filter(m => m.kind === 'enum').length} enums)`);

  return models;
}

async function main() {
  const args = process.argv.slice(2);
  const serviceArg = args.find(arg => arg.startsWith('--service='));

  if (!serviceArg) {
    console.error('Usage: ts-node parse-models.ts --service=<service-name>');
    console.error('Example: ts-node parse-models.ts --service=core');
    process.exit(1);
  }

  const serviceName = serviceArg.split('=')[1];
  console.error(`\nOCI SDK Model Parser - ${serviceName}`);
  console.error('='.repeat(50));

  const models = await parseModels(serviceName);

  // Output JSON to stdout
  console.log(JSON.stringify(models, null, 2));
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
