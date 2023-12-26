import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import "./App.css";
import { Page } from "./Page";
import { TransportProvider } from "@connectrpc/connect-query";
import { createGrpcWebTransport } from "@connectrpc/connect-web";

const queryClient = new QueryClient();
const transport = createGrpcWebTransport({
  baseUrl: "http://localhost:9000",
});

function App() {
  return (
    <TransportProvider transport={transport}>
      <QueryClientProvider client={queryClient}>
        <Page />
      </QueryClientProvider>
    </TransportProvider>
  );
}

export default App;
