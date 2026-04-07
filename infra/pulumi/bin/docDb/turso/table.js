"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TursoTable = void 0;
const pulumi = require("@pulumi/pulumi");
class TursoTable extends pulumi.dynamic.Resource {
    constructor(name, args, opts) {
        super(new TursoTableProvider(), name, args, opts);
    }
}
exports.TursoTable = TursoTable;
class TursoTableProvider {
    async create(inputs) {
        const config = new pulumi.Config("turso");
        const apiKey = config.require("apiToken");
        const tokenResp = await fetch(`https://api.turso.tech/v1/organizations/${inputs.organizationSlug}/databases/${inputs.databaseName}/auth/tokens`, {
            method: "POST",
            headers: { Authorization: `Bearer ${apiKey}` },
        });
        if (!tokenResp.ok) {
            throw new Error(`Failed to get DB token: ${await tokenResp.text()}`);
        }
        const { jwt } = await tokenResp.json();
        const url = `https://${inputs.databaseName}-${inputs.organizationSlug}.${inputs.location}.turso.io/v2/pipeline`;
        const response = await fetch(url, {
            method: "POST",
            headers: {
                Authorization: `Bearer ${jwt}`,
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                requests: [
                    {
                        type: "execute",
                        stmt: {
                            sql: inputs.createTableSql,
                        },
                    },
                    {
                        type: "close",
                    },
                ],
            }),
        });
        if (!response.ok) {
            const error = await response.text();
            throw new Error(`Failed to create table at ${url}: ${error}`);
        }
        const crypto = await Promise.resolve().then(() => require("crypto"));
        const hash = crypto.createHash("sha256");
        hash.update(inputs.createTableSql);
        const digest = hash.digest("hex");
        return {
            id: digest,
            outs: inputs,
        };
    }
    async delete(id, outputs) {
        const response = await fetch(`https://${outputs.databaseName}-${outputs.organizationSlug}.turso.io`, {
            method: "POST",
            headers: {
                Authorization: `Bearer ${outputs.jwt}`,
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                requests: [
                    {
                        type: "execute",
                        stmt: {
                            sql: `DROP TABLE IF EXISTS ${id}`,
                        },
                    },
                    {
                        type: "close",
                    },
                ],
            }),
        });
        if (!response.ok) {
            const error = await response.text();
            throw new Error(`Failed to delete table: ${error}`);
        }
    }
}
//# sourceMappingURL=table.js.map