#!/usr/bin/env ts-node
/**
 * TypeScript Client Parser
 * Parses TypeScript client methods from OCI SDK to extract API endpoints
 */

import { Project, MethodDeclaration, SyntaxKind } from 'ts-morph';
import * as path from 'path';
import * as fs from 'fs';
import { fileURLToPath } from 'url';

// ES module equivalent of __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

interface ParsedClientMethod {
  methodName: string;        // camelCase from TypeScript
  requestType: string;       // Request type name
  responseType: string;      // Response type name
  httpMethod: string;        // GET, POST, PUT, DELETE
  httpPath: string;          // API path template
  pathParams: string[];      // Path parameter field names
  queryParams: string[];     // Query parameter field names
  bodyParam?: string;        // Body parameter field name (for POST/PUT)
  responseBodyKey?: string;  // Response body key (for GET)
}

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

    if ((isLower || isDigit) && isNextUpper) {
      result += '_';
    }
  }

  return result;
}

function parseClientMethod(method: MethodDeclaration): ParsedClientMethod | null {
  const methodName = method.getName();

  // Get request and response types from signature
  const parameters = method.getParameters();
  if (parameters.length === 0) return null;

  const requestParam = parameters[0];
  const requestType = requestParam.getType().getText().replace(/^.*\./, ''); // Remove namespace

  const returnType = method.getReturnType().getText();
  const responseMatch = returnType.match(/Promise<.*\.(\w+)>/);
  if (!responseMatch) return null;
  const responseType = responseMatch[1];

  // Parse method body to extract HTTP details
  const body = method.getBodyText() || '';

  // Extract path
  const pathMatch = body.match(/path:\s*["']([^"']+)["']/);
  if (!pathMatch) return null;
  const httpPath = pathMatch[1];

  // Extract HTTP method
  const methodMatch = body.match(/method:\s*["'](\w+)["']/);
  if (!methodMatch) return null;
  const httpMethod = methodMatch[1];

  // Extract path parameters from the path template
  const pathParams: string[] = [];
  const pathParamMatches = httpPath.matchAll(/\{(\w+)\}/g);
  for (const match of pathParamMatches) {
    // Convert to snake_case (e.g., containerInstanceId -> container_instance_id)
    const camelCase = match[1];
    pathParams.push(camelCase);
  }

  // Extract query parameters
  const queryParams: string[] = [];
  const queryParamsMatch = body.match(/const queryParams = \{([^}]+)\}/s);
  if (queryParamsMatch) {
    const queryParamMatches = queryParamsMatch[1].matchAll(/["'](\w+)["']:\s*\w+Request\.(\w+)/g);
    for (const match of queryParamMatches) {
      queryParams.push(match[2]);
    }
  }

  // Extract body parameter (for POST/PUT)
  let bodyParam: string | undefined;
  const bodyMatch = body.match(/bodyContent:[\s\S]*?(\w+)Request\.(\w+)/);
  if (bodyMatch) {
    bodyParam = bodyMatch[2];
  }

  // Extract response body key (for GET)
  let responseBodyKey: string | undefined;
  const bodyKeyMatch = body.match(/bodyKey:\s*["'](\w+)["']/);
  if (bodyKeyMatch) {
    responseBodyKey = bodyKeyMatch[1];
  }

  return {
    methodName,
    requestType,
    responseType,
    httpMethod,
    httpPath,
    pathParams,
    queryParams,
    bodyParam,
    responseBodyKey,
  };
}

async function parseClient(serviceName: string): Promise<ParsedClientMethod[]> {
  const sdkPath = path.join(__dirname, '../../oci-typescript-sdk');
  const clientPath = path.join(sdkPath, 'lib', serviceName, 'lib', 'client.ts');

  if (!fs.existsSync(clientPath)) {
    console.error(`Error: Client file not found: ${clientPath}`);
    return [];
  }

  console.error(`Parsing client from ${clientPath}`);

  const project = new Project({
    tsConfigFilePath: path.join(__dirname, 'tsconfig.json'),
    skipAddingFilesFromTsConfig: true,
  });

  project.addSourceFileAtPath(clientPath);
  const sourceFile = project.getSourceFile(clientPath);
  if (!sourceFile) return [];

  const methods: ParsedClientMethod[] = [];

  // Find the client class
  for (const classDecl of sourceFile.getClasses()) {
    // Parse public async methods
    for (const method of classDecl.getMethods()) {
      if (!method.isAsync()) continue;
      // Check if method is public (methods are public by default in TypeScript)
      const modifiers = method.getModifiers();
      const isPublic = modifiers.some(m => m.getKind() === SyntaxKind.PublicKeyword) ||
                       !modifiers.some(m => m.getKind() === SyntaxKind.PrivateKeyword || m.getKind() === SyntaxKind.ProtectedKeyword);
      if (!isPublic) continue;

      const parsed = parseClientMethod(method);
      if (parsed) {
        methods.push(parsed);
      }
    }
  }

  console.error(`Parsed ${methods.length} client methods`);

  return methods;
}

async function main() {
  const args = process.argv.slice(2);
  const serviceArg = args.find(arg => arg.startsWith('--service='));

  if (!serviceArg) {
    console.error('Usage: ts-node parse-client.ts --service=<service-name>');
    console.error('Example: ts-node parse-client.ts --service=containerinstances');
    process.exit(1);
  }

  const serviceName = serviceArg.split('=')[1];
  console.error(`\nOCI SDK Client Parser - ${serviceName}`);
  console.error('='.repeat(50));

  const methods = await parseClient(serviceName);

  // Output JSON to stdout
  console.log(JSON.stringify(methods, null, 2));
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
