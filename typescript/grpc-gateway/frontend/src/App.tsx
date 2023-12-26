import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import "./App.css";
import { Page } from "./Page";

const queryClient = new QueryClient();

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <Page />
    </QueryClientProvider>
  );
}

export default App;
