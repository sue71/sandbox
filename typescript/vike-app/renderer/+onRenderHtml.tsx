// /renderer/+onRenderHtml.js
// Environment: server

import { escapeInject } from "vike/server";

export { onRenderHtml };

async function onRenderHtml() {
  // Note that `div#root` is empty
  return escapeInject`<html>
    <body>
      <div id="root"></div>
    </body>
  </html>`;
}
