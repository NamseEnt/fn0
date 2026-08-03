# `forte cloud init` Specification

Status: **Implemented**.

This document defines the non-interactive Cloudflare setup contract for the
Forte CLI. The command remains a single operation: it resolves the requested
zone, registers the project, provisions its Cloudflare resources, creates the
credentials Forte needs, and prints the DNS record that must be added.

## Invocation

```sh
forte cloud init \
  --project . \
  --project-name my-app \
  --zone example.com
```

The Cloudflare setup token is supplied through `CLOUDFLARE_API_TOKEN`. It is
not a command-line argument and the command never reads from standard input.

If the environment variable or any required argument is missing, the command
prints an error to stderr and exits with a non-zero status. It never prompts,
waits for input, or selects a default value.

## Arguments

| Argument | Required | Meaning |
|---|---|---|
| `-p, --project <dir>` | no | Forte project directory; defaults to `.` |
| `--project-name <name>` | for a new project | Project identity and DNS label |
| `--zone <name>` | for a new project | Cloudflare zone name, such as `example.com` |

`--domain` is not part of the default contract. The public hostname is
derived as `<project-name>.<zone>`.

For example, `my-app` in the `example.com` zone answers on
`my-app.example.com`.

## Project name validation

`--project-name` is one DNS hostname label, not an arbitrary display name. It
must satisfy all of these rules:

- 1 to 63 ASCII characters
- only lowercase `a-z`, digits, and `-`
- starts and ends with a lowercase letter or digit
- no spaces, dots, underscores, uppercase letters, or Unicode characters

The CLI rejects invalid input rather than normalizing it. The derived full
hostname must also fit within the DNS maximum length of 253 characters.

## Zone resolution

`--zone` receives the zone name, not the Cloudflare zone ID. `example.com` is
the zone name. Cloudflare's internal hexadecimal zone ID is resolved locally
for provisioning; `Forte.toml` stores the human-readable zone name.

The CLI must resolve the exact requested zone. It must not choose the first
zone returned by the API. A missing or inaccessible zone is an error.

## Configuration

After successful setup, `Forte.toml` stores:

- `project_id`
- `project_name`
- `zone`
- `domain`

The `domain` value is derived from `project_name` and `zone`. On later runs,
the stored values are checked against the requested values. A mismatch is
reported as an error instead of starting a reconfiguration flow.

## Authentication and secrets

`CLOUDFLARE_API_TOKEN` is a bootstrap credential with the permissions needed
to provision the account and create the narrower project credentials. The
CLI keeps it local to the setup process, never sends it to fn0, never prints
it, and never includes it in an error message.

The bootstrap credential is reusable across projects. Project credentials are
created with only the permissions and resource scope required by each project.

## Operation order

1. Validate all local arguments and `CLOUDFLARE_API_TOKEN`.
2. Resolve the exact Cloudflare zone name.
3. Derive and validate `<project-name>.<zone>`.
4. Create or resolve the Forte project identity.
5. Write the four cloud fields to `Forte.toml` so a failed retry reuses the
   same project identity.
6. Provision or reuse the project's Cloudflare buckets and zone resources.
7. Create or reuse the project's narrow credentials.
8. Sign and register the origin certificate for the derived hostname.
9. Print the required proxied CNAME record.

Every step must be safe to retry. Existing resources belonging to the same
project are reused rather than duplicated.

## Output and failure behavior

Normal output is human-readable text. Errors go to stderr and use a non-zero
exit status. JSON output is not required by this contract.

The command must fail before making Cloudflare changes when local validation
fails, including:

- missing `CLOUDFLARE_API_TOKEN`
- missing `--project-name` or `--zone` for a new project
- invalid DNS label
- inaccessible zone
- a configuration mismatch on an existing project
