// GitHub's OAuth app form requires /callback. Device flow never reads this body.
export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (url.pathname === "/callback/") {
      return Response.redirect(new URL("/callback", url), 302);
    }
    if (url.pathname === "/callback") {
      const asset = new URL(request.url);
      asset.pathname = "/index.html";
      asset.search = "";
      return env.ASSETS.fetch(new Request(asset, request));
    }
    return new Response("Not found", {
      status: 404,
      headers: { "content-type": "text/plain; charset=utf-8" },
    });
  },
};
