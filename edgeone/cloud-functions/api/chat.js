export async function onRequest({ request, env }) {
  // Keeps model keys server-side while the CLI stays local.
  if (request.method !== "POST") {
    return new Response("method not allowed", { status: 405 });
  }

  const apiKey = env.MAKERS_MODELS_KEY;
  if (!apiKey) {
    return Response.json({ error: "MAKERS_MODELS_KEY is not configured" }, { status: 500 });
  }

  const baseUrl = normalizeBaseUrl(
    env.EDGEONE_GATEWAY_BASE_URL || env.EDGEONE_BASE_URL || "https://ai-gateway.edgeone.link/v1",
  );
  const body = await request.text();
  return fetch(`${baseUrl}/chat/completions`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    },
    body,
  });
}

function normalizeBaseUrl(value) {
  const baseUrl = value.replace(/\/+$/, "");
  return baseUrl.endsWith("/v1") ? baseUrl : `${baseUrl}/v1`;
}
