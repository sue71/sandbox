// https://vike.dev/onRenderClient
export { onRenderClient };

import type { OnRenderClientAsync } from "vike/types";
import ReactDOM from "react-dom/client";
import { PageLayout } from "./PageLayout";

let root: ReactDOM.Root;

const onRenderClient: OnRenderClientAsync = async (
  pageContext
): ReturnType<OnRenderClientAsync> => {
  const { Page, pageProps } = pageContext;
  if (!Page)
    throw new Error(
      "Client-side render() hook expects pageContext.Page to be defined"
    );
  const container = document.getElementById("root");
  const page = (
    <PageLayout>
      <Page {...pageProps} />
    </PageLayout>
  );
  if (!root) {
    root = ReactDOM.createRoot(container!);
  }
  root.render(page);
};
