import { useEffect } from "react";
import { Counter } from "./Counter";
import { prefetch } from "vike/client/router";

export { Page };

function Page() {
  useEffect(() => {
    prefetch("/about");
  });
  return (
    <>
      <h1>Welcome</h1>
      This page is:
      <ul>
        <li>Rendered to HTML.</li>
        <li>
          Interactive. <Counter />
        </li>
      </ul>
    </>
  );
}
