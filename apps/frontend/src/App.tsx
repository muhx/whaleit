import { isWeb } from "@/adapters";
import { AuthProvider } from "@/context/auth-context";
import { WhaleItConnectProvider } from "@/features/connect";
import { SettingsProvider } from "@/lib/settings-provider";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { TooltipProvider } from "@whaleit/ui";
import { useState } from "react";
import { PrivacyProvider } from "./context/privacy-context";
import { LoginPage } from "./pages/auth/login-page";
import { ProtectedAppRoutes } from "./routes";

function App() {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            refetchOnWindowFocus: false,
            staleTime: 5 * 60 * 1000,
            retry: false,
          },
        },
      }),
  );

  const isWebEnv = isWeb;

  window.__whaleit_query_client__ = queryClient;

  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <WhaleItConnectProvider>
          <PrivacyProvider>
            <SettingsProvider>
              <TooltipProvider>
                <ProtectedAppRoutes isWeb={isWebEnv} loginPage={<LoginPage />} />
              </TooltipProvider>
            </SettingsProvider>
          </PrivacyProvider>
        </WhaleItConnectProvider>
      </AuthProvider>
    </QueryClientProvider>
  );
}

export default App;
