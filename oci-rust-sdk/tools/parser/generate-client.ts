#!/usr/bin/env ts-node
/**
 * Generate Rust client mod.rs from parsed client methods
 */

import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

interface ClientMethod {
  methodName: string;
  requestType: string;
  responseType: string;
  httpMethod: string;
  httpPath: string;
  pathParams: string[];
  queryParams: string[];
  bodyParam?: string;
  responseBodyKey?: string;
}

function toSnakeCase(str: string): string {
  if (!str) return str;
  let result = '';
  for (let i = 0; i < str.length; i++) {
    const char = str[i];
    const nextChar = i < str.length - 1 ? str[i + 1] : null;
    const isLower = char >= 'a' && char <= 'z';
    const isDigit = char >= '0' && char <= '9';
    const isNextUpper = nextChar && nextChar >= 'A' && nextChar <=  'Z';
    result += char.toLowerCase();
    if ((isLower || isDigit) && isNextUpper) {
      result += '_';
    }
  }
  return result;
}

function toPascalCase(str: string): string {
  if (!str) return str;
  // First convert to snake_case, then capitalize each part
  const snakeCase = toSnakeCase(str);
  return snakeCase.split('_').map(word => word.charAt(0).toUpperCase() + word.slice(1)).join('');
}

function generateMethod(method: ClientMethod, apiVersion: string): string {
  const rustMethodName = toSnakeCase(method.methodName);
  const pathTemplate = method.httpPath.replace(/\{(\w+)\}/g, '{}');
  const pathParams = method.pathParams.map(p => toSnakeCase(p));

  let code = `    /// ${method.methodName}\n`;
  code += `    pub async fn ${rustMethodName}(\n`;
  code += `        &self,\n`;
  code += `        request: ${method.requestType},\n`;
  code += `    ) -> Result<${method.responseType}> {\n`;
  code += `        let query_params = request.to_query_params();\n`;
  code += `        let query_string = if query_params.is_empty() {\n`;
  code += `            String::new()\n`;
  code += `        } else {\n`;
  code += `            format!(\n`;
  code += `                "?{}",\n`;
  code += `                query_params\n`;
  code += `                    .iter()\n`;
  code += `                    .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))\n`;
  code += `                    .collect::<Vec<_>>()\n`;
  code += `                    .join("&")\n`;
  code += `            )\n`;
  code += `        };\n\n`;

  code += `        let path = format!(\n`;
  code += `            "${apiVersion}${pathTemplate}{}",\n`;
  for (const param of pathParams) {
    code += `            request.${param},\n`;
  }
  code += `            query_string\n`;
  code += `        );\n`;

  // HTTP request
  if (method.httpMethod === 'GET') {
    code += `        let response = self.client.get(&path).await?;\n\n`;
  } else if (method.httpMethod === 'POST') {
    if (method.bodyParam) {
      const bodyParamSnake = toSnakeCase(method.bodyParam);
      if (method.responseBodyKey) {
        // POST with response body
        code += `        let response = self\n`;
        code += `            .client\n`;
        code += `            .post(&path, Some(&request.${bodyParamSnake}))\n`;
        code += `            .await?;\n\n`;
      } else {
        // POST without response body - need explicit type
        code += `        let response: OciResponse<serde_json::Value> = self\n`;
        code += `            .client\n`;
        code += `            .post(&path, Some(&request.${bodyParamSnake}))\n`;
        code += `            .await?;\n\n`;
      }
    } else {
      code += `        let response: OciResponse<serde_json::Value> = self\n`;
      code += `            .client\n`;
      code += `            .post(&path, None::<&serde_json::Value>)\n`;
      code += `            .await?;\n\n`;
    }
  } else if (method.httpMethod === 'DELETE') {
    code += `        let response: OciResponse<serde_json::Value> = self.client.delete(&path).await?;\n\n`;
  } else if (method.httpMethod === 'PUT') {
    if (method.bodyParam) {
      const bodyParamSnake = toSnakeCase(method.bodyParam);
      if (method.responseBodyKey) {
        // PUT with response body - need explicit type
        code += `        let response = self\n`;
        code += `            .client\n`;
        code += `            .put(&path, Some(&request.${bodyParamSnake}))\n`;
        code += `            .await?;\n\n`;
      } else {
        // PUT without response body
        code += `        let response: OciResponse<serde_json::Value> = self\n`;
        code += `            .client\n`;
        code += `            .put(&path, Some(&request.${bodyParamSnake}))\n`;
        code += `            .await?;\n\n`;
      }
    } else {
      code += `        let response: OciResponse<serde_json::Value> = self\n`;
      code += `            .client\n`;
      code += `            .put(&path, None::<&serde_json::Value>)\n`;
      code += `            .await?;\n\n`;
    }
  }

  // Response headers
  code += `        let opc_request_id = response.get_header("opc-request-id");\n`;
  if (method.httpMethod !== 'GET' || !method.responseBodyKey) {
    code += `        let opc_work_request_id = response.get_header("opc-work-request-id");\n`;
  }
  if (method.httpMethod === 'GET' && method.responseBodyKey) {
    code += `        let opc_next_page = response.get_header("opc-next-page");\n`;
  }
  code += `\n`;

  // Response construction
  code += `        Ok(${method.responseType} {\n`;
  if (method.responseBodyKey) {
    // Collection types use 'items' field
    const responseField = method.responseBodyKey.includes('Collection') ? 'items' : toSnakeCase(method.responseBodyKey);
    code += `            ${responseField}: response.body,\n`;
  }
  code += `            opc_request_id,\n`;
  if (method.httpMethod !== 'GET' || !method.responseBodyKey) {
    code += `            opc_work_request_id,\n`;
  }
  if (method.httpMethod === 'GET' && method.responseBodyKey) {
    code += `            opc_next_page,\n`;
  }
  code += `        })\n`;
  code += `    }\n`;

  return code;
}

function generateClient(serviceName: string, clientMethods: ClientMethod[], apiVersion: string, endpointPrefix: string): string {
  const clientName = `${serviceName.charAt(0).toUpperCase() + serviceName.slice(1)}Client`;

  let code = `//! ${serviceName} service module\n`;
  code += `pub mod models;\n`;
  code += `pub mod requests;\n\n`;
  code += `// Re-export commonly used types\n`;
  code += `pub use models::*;\n`;
  code += `pub use requests::*;\n\n`;
  code += `use crate::auth::provider::AuthProvider;\n`;
  code += `use crate::core::{\n`;
  code += `    Result,\n`;
  code += `    client::http_client::{OciClient, OciResponse},\n`;
  code += `    region::Region,\n`;
  code += `    retry::Retrier,\n`;
  code += `};\n`;
  code += `use std::sync::Arc;\n`;
  code += `use serde_json;\n\n`;
  code += `/// Client configuration for ${serviceName} service\n`;
  code += `pub struct ClientConfig {\n`;
  code += `    pub auth_provider: Arc<dyn AuthProvider>,\n`;
  code += `    pub region: Region,\n`;
  code += `    pub timeout: std::time::Duration,\n`;
  code += `    pub retry: Retrier,\n`;
  code += `}\n\n`;
  code += `/// Create a new ${serviceName} client\n`;
  code += `pub fn client(config: ClientConfig) -> Result<${clientName}> {\n`;
  code += `    let endpoint = format!(\n`;
  code += `        "https://${endpointPrefix}.{}.oci.oraclecloud.com",\n`;
  code += `        config.region.id()\n`;
  code += `    );\n`;
  code += `    let client = OciClient::new(config.auth_provider, endpoint)?.with_retrier(config.retry);\n\n`;
  code += `    Ok(${clientName} { client })\n`;
  code += `}\n\n`;
  code += `/// ${serviceName.charAt(0).toUpperCase() + serviceName.slice(1)} API client\n`;
  code += `pub struct ${clientName} {\n`;
  code += `    client: OciClient,\n`;
  code += `}\n\n`;
  code += `impl ${clientName} {\n`;

  for (const method of clientMethods) {
    code += generateMethod(method, apiVersion) + '\n';
  }

  code += `}\n\n`;

  // Generate response structs
  code += `/// Response types\n`;
  for (const method of clientMethods) {
    code += `pub struct ${method.responseType} {\n`;
    if (method.responseBodyKey) {
      const responseField = method.responseBodyKey.includes('Collection') ? 'items' : toSnakeCase(method.responseBodyKey);
      if (method.httpMethod === 'GET') {
        // For list operations, assume items: Vec<...>
        if (method.responseBodyKey.includes('Collection')) {
          // Convert camelCase to PascalCase, removing "Collection"
          // e.g., containerInstanceShapeCollection -> ContainerInstanceShape -> but we want ContainerInstanceShapeSummary
          // e.g., workRequestErrorCollection -> WorkRequestError (not WorkRequestErrorSummary!)
          const baseType = method.responseBodyKey.replace('Collection', '');
          // If it ends with "Shape" or doesn't already end with "Summary", add "Summary"
          const summaryType = baseType.endsWith('Summary') || baseType.endsWith('Error') || baseType.endsWith('Entry') ? baseType : baseType + 'Summary';
          const pascalCaseSummaryType = toPascalCase(summaryType);
          code += `    pub items: Vec<${pascalCaseSummaryType}>,\n`;
        } else {
          // Special case for 'value' field
          if (method.responseBodyKey === 'value') {
            code += `    pub ${responseField}: serde_json::Value,\n`;
          } else {
            // Convert camelCase to PascalCase for type name
            const typeName = toPascalCase(method.responseBodyKey);
            code += `    pub ${responseField}: ${typeName},\n`;
          }
        }
      } else {
        // For POST with body
        if (method.responseBodyKey === 'value') {
          code += `    pub ${responseField}: serde_json::Value,\n`;
        } else {
          const typeName = toPascalCase(method.responseBodyKey);
          code += `    pub ${responseField}: ${typeName},\n`;
        }
      }
    }
    code += `    pub opc_request_id: Option<String>,\n`;
    if (method.httpMethod !== 'GET' || !method.responseBodyKey) {
      code += `    pub opc_work_request_id: Option<String>,\n`;
    }
    if (method.httpMethod === 'GET' && method.responseBodyKey) {
      code += `    pub opc_next_page: Option<String>,\n`;
    }
    code += `}\n\n`;
  }

  return code;
}

async function main() {
  const args = process.argv.slice(2);
  const serviceArg = args.find(arg => arg.startsWith('--service='));
  const apiVersionArg = args.find(arg => arg.startsWith('--api-version='));
  const endpointArg = args.find(arg => arg.startsWith('--endpoint='));

  if (!serviceArg) {
    console.error('Usage: ts-node generate-client.ts --service=<service-name> --api-version=/20210410 --endpoint=containerinstances');
    process.exit(1);
  }

  const serviceName = serviceArg.split('=')[1];
  const apiVersion = apiVersionArg ? apiVersionArg.split('=')[1] : '/20210410';
  const endpointPrefix = endpointArg ? endpointArg.split('=')[1] : serviceName;

  const clientJsonPath = path.join(__dirname, '../../data/parsed', `${serviceName}-client.json`);

  if (!fs.existsSync(clientJsonPath)) {
    console.error(`Client JSON not found: ${clientJsonPath}`);
    process.exit(1);
  }

  const content = fs.readFileSync(clientJsonPath, 'utf-8');
  // Skip stderr lines at the beginning
  const jsonStart = content.indexOf('[');
  const jsonContent = content.substring(jsonStart);
  const methods: ClientMethod[] = JSON.parse(jsonContent);

  const clientCode = generateClient(serviceName, methods, apiVersion, endpointPrefix);
  console.log(clientCode);
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
