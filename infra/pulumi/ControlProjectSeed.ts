import * as pulumi from "@pulumi/pulumi";

interface ControlProjectSeedArgs {
  organizationSlug: pulumi.Input<string>;
  location: pulumi.Input<string>;
  databaseName: pulumi.Input<string>;
  jwt: pulumi.Input<string>;
  projectId: pulumi.Input<string>;
  ownerGithubId: pulumi.Input<number>;
  ownerGithubLogin: pulumi.Input<string>;
  displayName: pulumi.Input<string>;
}

export class ControlProjectSeed extends pulumi.dynamic.Resource {
  constructor(
    name: string,
    args: ControlProjectSeedArgs,
    opts?: pulumi.CustomResourceOptions
  ) {
    super(new ControlProjectSeedProvider(), name, args, opts);
  }
}

interface ControlProjectSeedInputs {
  organizationSlug: string;
  location: string;
  databaseName: string;
  jwt: string;
  projectId: string;
  ownerGithubId: number;
  ownerGithubLogin: string;
  displayName: string;
}

const CREATE_DOCS_TABLE_SQL =
  "CREATE TABLE IF NOT EXISTS docs (pk TEXT, sk TEXT, data BLOB, version INTEGER NOT NULL DEFAULT 0, PRIMARY KEY (pk, sk))";

const UPSERT_DOC_SQL =
  "INSERT INTO docs (pk, sk, data, version) VALUES (?, ?, ?, 0) ON CONFLICT(pk, sk) DO NOTHING";

function pkInteger(value: number): string {
  return value.toString().padStart(20, "0");
}

function nowIso(): string {
  return new Date().toISOString().replace(/\.\d+Z$/, "Z");
}

function toBase64(s: string): string {
  return Buffer.from(s, "utf8").toString("base64");
}

class ControlProjectSeedProvider implements pulumi.dynamic.ResourceProvider {
  async create(
    inputs: ControlProjectSeedInputs
  ): Promise<pulumi.dynamic.CreateResult> {
    const {
      organizationSlug,
      location,
      databaseName,
      jwt,
      projectId,
      ownerGithubId,
      ownerGithubLogin,
      displayName,
    } = inputs;

    const url = `https://${databaseName}-${organizationSlug}.${location}.turso.io/v2/pipeline`;
    const createdAt = nowIso();

    const projectDoc = {
      project_id: projectId,
      owner_github_id: ownerGithubId,
      name: displayName,
      created_at: createdAt,
    };
    const userDoc = {
      github_id: ownerGithubId,
      github_login: ownerGithubLogin,
      created_at: createdAt,
      cli_tokens: [],
      web_sessions: [],
      projects: [{ project_id: projectId, name: displayName }],
    };

    const projectPk = `ProjectDoc/project_id=${projectId}`;
    const userPk = `UserDoc/github_id=${pkInteger(ownerGithubId)}`;

    const requests = [
      { type: "execute", stmt: { sql: CREATE_DOCS_TABLE_SQL } },
      {
        type: "execute",
        stmt: {
          sql: UPSERT_DOC_SQL,
          args: [
            { type: "text", value: projectPk },
            { type: "text", value: "" },
            { type: "blob", base64: toBase64(JSON.stringify(projectDoc)) },
          ],
        },
      },
      {
        type: "execute",
        stmt: {
          sql: UPSERT_DOC_SQL,
          args: [
            { type: "text", value: userPk },
            { type: "text", value: "" },
            { type: "blob", base64: toBase64(JSON.stringify(userDoc)) },
          ],
        },
      },
      { type: "close" },
    ];

    const response = await fetch(url, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${jwt}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ requests }),
    });

    if (!response.ok) {
      throw new Error(
        `ControlProjectSeed create failed (${response.status}): ${await response.text()}`
      );
    }

    const body = await response.json();
    if (Array.isArray(body.results)) {
      for (const r of body.results) {
        if (r.type === "error") {
          throw new Error(
            `ControlProjectSeed SQL error: ${JSON.stringify(r.error)}`
          );
        }
      }
    }

    return {
      id: `${projectId}@${databaseName}`,
      outs: inputs,
    };
  }

  async update(): Promise<pulumi.dynamic.UpdateResult> {
    return { outs: {} };
  }

  async delete(): Promise<void> {
    return;
  }
}
