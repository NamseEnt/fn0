"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TursoDocDb = void 0;
const pulumi = require("@pulumi/pulumi");
const group_1 = require("./turso/group");
const database_1 = require("./turso/database");
const dbToken_1 = require("./turso/dbToken");
const table_1 = require("./turso/table");
const random = require("@pulumi/random");
class TursoDocDb extends pulumi.ComponentResource {
    constructor(name, args, opts) {
        super("pkg:index:turso-doc-db", name, args, opts);
        const { location, organizationSlug } = args;
        const nameSuffix8 = new random.RandomString("name-suffix-8", {
            length: 8,
            special: false,
            upper: false,
        }, { parent: this }).result;
        const group = new group_1.TursoGroup("group", {
            organizationSlug,
            name: "fn0-doc-db",
            location,
        }, { parent: this });
        const database = new database_1.TursoDatabase("database", {
            organizationSlug,
            name: pulumi.interpolate `fn0-doc-db-${nameSuffix8}`,
            group: group.name,
        }, { parent: this });
        const token = new dbToken_1.TursoDatabaseToken("token", {
            organizationSlug,
            databaseName: database.name,
        }, { parent: this });
        new table_1.TursoTable("docs-table", {
            organizationSlug,
            location,
            jwt: token.jwt,
            databaseName: database.name,
            createTableSql: `
CREATE TABLE IF NOT EXISTS docs (
  pk BLOB NOT NULL,
  sk BLOB NOT NULL,
  value BLOB NOT NULL,
  PRIMARY KEY (pk, sk)
) WITHOUT ROWID;`.trim(),
        }, { parent: this });
        this.url = pulumi.interpolate `libsql://${database.name}-${organizationSlug}.${location}.turso.io`;
        const tursoConfig = new pulumi.Config("turso");
        this.token = database.name.apply((dbName) => {
            const apiKey = tursoConfig.require("apiToken");
            return fetch(`https://api.turso.tech/v1/organizations/${organizationSlug}/databases/${dbName}/auth/tokens`, { method: "POST", headers: { Authorization: `Bearer ${apiKey}` } }).then((r) => r.json()).then((d) => d.jwt);
        });
    }
}
exports.TursoDocDb = TursoDocDb;
//# sourceMappingURL=TursoDocDb.js.map