"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TursoGroup = void 0;
const pulumi = require("@pulumi/pulumi");
class TursoGroup extends pulumi.dynamic.Resource {
    constructor(name, args, opts) {
        super(new TursoGroupProvider(), name, args, opts);
    }
}
exports.TursoGroup = TursoGroup;
class TursoGroupProvider {
    async create(inputs) {
        const config = new pulumi.Config("turso");
        const apiKey = config.require("apiToken");
        const response = await fetch(`https://api.turso.tech/v1/organizations/${inputs.organizationSlug}/groups`, {
            method: "POST",
            headers: {
                Authorization: `Bearer ${apiKey}`,
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                name: inputs.name,
                location: inputs.location,
            }),
        });
        if (!response.ok) {
            const error = await response.text();
            throw new Error(`Failed to create group: ${error}`);
        }
        const data = await response.json();
        const group = data.group;
        return {
            id: group.uuid,
            outs: inputs,
        };
    }
    async delete(id, outputs) {
        const config = new pulumi.Config("turso");
        const apiKey = config.require("apiToken");
        const groupName = id.split("/").pop();
        const response = await fetch(`https://api.turso.tech/v1/organizations/${outputs.organizationSlug}/groups/${groupName}`, {
            method: "DELETE",
            headers: {
                Authorization: `Bearer ${apiKey}`,
            },
        });
        if (!response.ok && response.status !== 404) {
            const error = await response.text();
            throw new Error(`Failed to delete group: ${error}`);
        }
    }
}
//# sourceMappingURL=group.js.map