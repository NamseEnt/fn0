"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TursoDatabase = void 0;
const pulumi = require("@pulumi/pulumi");
class TursoDatabase extends pulumi.dynamic.Resource {
    constructor(name, args, opts) {
        super(new TursoDatabaseProvider(), name, args, opts);
    }
}
exports.TursoDatabase = TursoDatabase;
class TursoDatabaseProvider {
    async create(inputs) {
        const config = new pulumi.Config("turso");
        const apiKey = config.require("apiToken");
        const response = await fetch(`https://api.turso.tech/v1/organizations/${inputs.organizationSlug}/databases`, {
            method: "POST",
            headers: {
                Authorization: `Bearer ${apiKey}`,
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                name: inputs.name,
                group: inputs.group,
            }),
        });
        if (!response.ok) {
            const error = await response.text();
            throw new Error(`Failed to create database: ${error}`);
        }
        const data = await response.json();
        return {
            id: data.database.Name,
            outs: {
                ...inputs,
                name: data.database.Name,
            },
        };
    }
    async delete(id, outputs) {
        const config = new pulumi.Config("turso");
        const apiKey = config.require("apiToken");
        const response = await fetch(`https://api.turso.tech/v1/organizations/${outputs.organizationSlug}/databases/${id}`, {
            method: "DELETE",
            headers: {
                Authorization: `Bearer ${apiKey}`,
            },
        });
        if (!response.ok && response.status !== 404) {
            const error = await response.text();
            throw new Error(`Failed to delete database: ${error}`);
        }
    }
}
//# sourceMappingURL=database.js.map