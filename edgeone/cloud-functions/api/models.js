export function onRequest() {
  return Response.json({
    models: [
      {
        id: "@makers/deepseek-v4-flash",
        vendor: "DeepSeek",
        streaming: true,
        pricing: "unknown/unverified",
      },
      {
        id: "@makers/hunyuan-turbos-latest",
        vendor: "Hunyuan",
        streaming: true,
        pricing: "unknown/unverified",
      },
    ],
  });
}
