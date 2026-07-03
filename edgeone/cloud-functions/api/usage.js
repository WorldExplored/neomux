export function onRequest() {
  return Response.json({
    product: "neomux",
    pricing: "unknown/unverified",
    usage: "not connected",
    note: "Wire this to an official EdgeOne usage API when one is available.",
  });
}
