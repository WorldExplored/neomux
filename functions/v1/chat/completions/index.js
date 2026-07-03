const corsHeaders = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "POST, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type, Authorization",
};

const streamingHeaders = {
  ...corsHeaders,
  "Content-Type": "text/event-stream",
  "Cache-Control": "no-cache",
  Connection: "keep-alive",
};

const jsonHeaders = {
  ...corsHeaders,
  "Content-Type": "application/json",
};

const systemMessage = {
  role: "system",
  content:
    "You are Agent Forge, a terse coding agent in a tmux pane. Be practical. Prefer simple code, clear diffs, and shell commands the user can inspect.",
};

const edgeModels = new Map([
  ["deepseek", "@tx/deepseek-ai/deepseek-v4"],
  ["deepseek-v4", "@tx/deepseek-ai/deepseek-v4"],
  ["deepseek-v3", "@tx/deepseek-ai/deepseek-v3-0324"],
  ["deepseek-v32", "@tx/deepseek-ai/deepseek-v32"],
  ["deepseek-r1", "@tx/deepseek-ai/deepseek-r1-0528"],
]);

const gatewayModels = new Map([
  ["makers-deepseek", "@makers/deepseek-v4-flash"],
  ["minimax", "@makers/minimax-m2.7"],
  ["hy3", "@makers/hy3-preview"],
  ["codex", "openai/gpt-5.4-mini"],
  ["codex-frontier", "openai/gpt-5.5"],
  ["fable", "anthropic/claude-fable-5"],
]);

const edgeModelValues = new Set(edgeModels.values());
const gatewayModelValues = new Set(gatewayModels.values());

function jsonResponse(payload, status) {
  return new Response(JSON.stringify(payload), {
    status,
    headers: jsonHeaders,
  });
}

function supportedModels() {
  return {
    edge: Object.fromEntries(edgeModels),
    gateway: Object.fromEntries(gatewayModels),
  };
}

function normalizeMessages(body) {
  if (Array.isArray(body.messages) && body.messages.length > 0) {
    return body.messages.map((message) => ({
      role: message.role,
      content: message.content,
    }));
  }

  if (typeof body.content === "string" && body.content.trim()) {
    return [systemMessage, { role: "user", content: body.content }];
  }

  throw new Error("Expected either messages[] or a non-empty content string.");
}

function normalizeModel(model) {
  const requested = typeof model === "string" ? model.trim() : "";
  const key = requested || "deepseek";

  if (edgeModels.has(key)) {
    return { model: edgeModels.get(key), source: "edge" };
  }

  if (gatewayModels.has(key)) {
    return { model: gatewayModels.get(key), source: "gateway" };
  }

  if (edgeModelValues.has(key)) {
    return { model: key, source: "edge" };
  }

  if (gatewayModelValues.has(key)) {
    return { model: key, source: "gateway" };
  }

  throw new Error(`Unsupported model: ${key}`);
}

function gatewayToken(request, env) {
  const authorization = request.headers.get("Authorization");
  if (authorization?.toLowerCase().startsWith("bearer ")) {
    return authorization.slice(7).trim();
  }

  if (env?.MAKERS_MODELS_KEY) {
    return env.MAKERS_MODELS_KEY;
  }

  if (typeof process !== "undefined" && process.env?.MAKERS_MODELS_KEY) {
    return process.env.MAKERS_MODELS_KEY;
  }

  if (typeof globalThis.MAKERS_MODELS_KEY === "string") {
    return globalThis.MAKERS_MODELS_KEY;
  }

  return "";
}

async function callGateway({ request, env, model, messages }) {
  const token = gatewayToken(request, env);
  if (!token) {
    return jsonResponse(
      {
        error: "GATEWAY_KEY_REQUIRED",
        message: "This model requires a Makers Models key.",
        supportedModels: supportedModels(),
      },
      401,
    );
  }

  const response = await fetch("https://ai-gateway.edgeone.link/v1/chat/completions", {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      model,
      messages,
      stream: true,
    }),
  });

  if (!response.ok || !response.body) {
    const text = await response.text();
    throw new Error(`Gateway request failed: ${response.status} ${text}`);
  }

  return new Response(response.body, {
    headers: streamingHeaders,
  });
}

export async function onRequestOptions() {
  return new Response(null, {
    status: 204,
    headers: corsHeaders,
  });
}

export async function onRequestPost({ request, env }) {
  let body;
  try {
    body = await request.json();
  } catch {
    return jsonResponse(
      {
        error: "INVALID_JSON",
        message: "Request body must be valid JSON.",
      },
      400,
    );
  }

  let messages;
  let modelConfig;
  try {
    messages = normalizeMessages(body);
    modelConfig = normalizeModel(body.model);
  } catch (error) {
    return jsonResponse(
      {
        error: "INVALID_REQUEST",
        message: error.message,
        supportedModels: supportedModels(),
      },
      400,
    );
  }

  try {
    if (modelConfig.source === "gateway") {
      return await callGateway({ request, env, model: modelConfig.model, messages });
    }

    const response = await AI.chatCompletions({
      model: modelConfig.model,
      messages,
      stream: true,
    });

    return new Response(response, {
      headers: streamingHeaders,
    });
  } catch (error) {
    return jsonResponse(
      {
        error: "MODEL_SERVICE_ERROR",
        message: error.message,
      },
      503,
    );
  }
}
