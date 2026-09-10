---
name: cloudflare-setup-token
description: Create the Cloudflare setup token needed by forte cloud init or rotate by driving the Cloudflare dashboard and handing the token to forte through the clipboard. Use when the user asks Codex to connect or repair a Cloudflare broker; pause for login, 2FA, and final token-creation approval.
---

# Cloudflare Setup Token

Use this skill when `forte cloud init` or `forte cloud rotate` needs a fresh
Cloudflare setup token and the user wants Codex to operate the dashboard. The
token must have exactly `User -> API Tokens -> Edit` permission.

## Start the CLI handoff

Run the relevant command in a background shell immediately before the dashboard
token is ready to be copied:

```sh
forte cloud init --project <project-dir> --project-name <name> --zone <zone> --setup-token-from-clipboard
```

For an existing broker, use:

```sh
forte cloud rotate --project <project-dir> --setup-token-from-clipboard
```

The command polls the OS clipboard, verifies a candidate against Cloudflare,
wipes it after acceptance, and continues the bootstrap or broker republish.

## Dashboard procedure

Use the browser controls available in this Codex session. Open:
`https://dash.cloudflare.com/profile/api-tokens`.

1. If Cloudflare asks for a password, 2FA code, or identity confirmation, stop
   and ask the user to complete it in the browser. Never type credentials or
   authentication codes.
2. Click **Create Token**.
3. Choose the **Create Additional Tokens** template and click **Use template**.
   Do not choose **Create Custom Token**.
4. Confirm that the form grants `User | API Tokens | Edit`, then continue to the
   summary.
5. Ask the user to explicitly approve the final token creation. Do not click
   **Create Token** until approval is given.
6. After creation, click only the page's native **Copy** control. Do not read,
   extract, print, type, or screenshot the token value, and do not inspect the
   result page text. The CLI receives the value from the clipboard.
7. Wait for the CLI to report success, then close the dashboard tab.

## Safety and failure handling

- Do not use web search to bypass the dashboard login.
- Do not call clipboard-reading tools or put the token into chat or command
  arguments.
- If the dashboard layout changes, the browser is unavailable, or the CLI
  rejects the copied value, stop and report the visible problem. Do not create
  another token without asking for approval again.
- A successful rotate ends with `Cloudflare broker setup token rotated and
  Worker republished`.
