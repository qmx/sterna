import type { Plugin, PluginInput } from "@opencode-ai/plugin";

type OpencodeClient = PluginInput["client"];

const STERNA_CLI_USAGE = `## CLI Usage
Use the \`st\` CLI via bash for Sterna operations:
- \`st ready --json\` - List ready tasks (open, unclaimed, unblocked)
- \`st get <id> --json\` - Show issue details
- \`st create "title" -d "desc" -t bug|feature|task -p 0-4\` - Create issue
- \`st claim <id> --context "branch"\` - Claim issue
- \`st close <id> --reason "message"\` - Close issue
- \`st release <id> --reason "message"\` - Release claim
- \`st reopen <id> --reason "message"\` - Reopen issue
- \`st list --status open --json\` - List issues
- \`st dep add <id> --needs <other>\` - Add dependency
- \`st dep remove <id> --needs <other>\` - Remove dependency
- \`st sync\` - Pull then push

Always use \`--json\` flag for structured output.`;

const STERNA_GUIDANCE = `<sterna-guidance>
${STERNA_CLI_USAGE}

## Agent Delegation
For multi-command beads work, use the \`task\` tool with \`subagent_type: "sterna-task-agent"\`:
- Status overviews ("what's next", "what's blocked")
- Finding and completing ready work
- Working through multiple issues

Use CLI directly for single atomic operations (create one issue, close one issue).
</sterna-guidance>`;

const TASK_AGENT_PROMPT = `You are a Sterna task completion agent.

${STERNA_CLI_USAGE}

## Your Purpose
Handle status queries AND autonomous task completion.

For status requests: Run \`st\` commands, parse JSON, return concise human-readable summary.
For task completion: Find ready work, claim it, execute it, close it.

Never dump raw JSON - summarize it.`;

async function getSessionContext(client: OpencodeClient, sessionID: string) {
  try {
    const response = await client.session.messages({
      path: { id: sessionID },
      query: { limit: 50 },
    });
    if (response.data) {
      for (const msg of response.data) {
        if (msg.info.role === "user" && "model" in msg.info && msg.info.model) {
          return { model: msg.info.model, agent: msg.info.agent };
        }
      }
    }
  } catch {}
  return undefined;
}

async function injectSternaContext(
  client: OpencodeClient,
  $: PluginInput["$"],
  sessionID: string,
  context?: { model?: { providerID: string; modelID: string }; agent?: string }
) {
  try {
    const primeOutput = await $`st prime`.text();
    if (!primeOutput?.trim()) return;

    const sternaContext = `<sterna-context>
${primeOutput.trim()}
</sterna-context>

${STERNA_GUIDANCE}`;

    await client.session.prompt({
      path: { id: sessionID },
      body: {
        noReply: true,
        model: context?.model,
        agent: context?.agent,
        parts: [{ type: "text", text: sternaContext, synthetic: true }],
      },
    });
  } catch {}
}

export const SternaPlugin: Plugin = async ({ client, $ }) => {
  const injectedSessions = new Set<string>();

  return {
    "chat.message": async (_input, output) => {
      const sessionID = output.message.sessionID;
      if (injectedSessions.has(sessionID)) return;

      try {
        const existing = await client.session.messages({ path: { id: sessionID } });
        if (existing.data?.some(msg => {
          const parts = (msg as any).parts || (msg.info as any).parts;
          return parts?.some((p: any) => p.text?.includes("<sterna-context>"));
        })) {
          injectedSessions.add(sessionID);
          return;
        }
      } catch {}

      injectedSessions.add(sessionID);
      await injectSternaContext(client, $, sessionID, {
        model: output.message.model,
        agent: output.message.agent,
      });
    },

    event: async ({ event }) => {
      if (event.type === "session.compacted") {
        const sessionID = event.properties.sessionID;
        const context = await getSessionContext(client, sessionID);
        await injectSternaContext(client, $, sessionID, context);
      }
    },

    config: async (config) => {
      config.agent = {
        ...config.agent,
        "sterna-task-agent": {
          description: "Sterna task completion agent",
          prompt: TASK_AGENT_PROMPT,
          mode: "subagent",
        },
      };

      if (!config.permission) config.permission = {};
      if (!config.permission.bash || typeof config.permission.bash === "string") {
        config.permission.bash = {};
      }
      config.permission.bash["st *"] = "allow";
    },
  };
};
