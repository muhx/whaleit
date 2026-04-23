import {
  type ConnectionConfig,
  getConnectionConfig,
  setConnectionConfig,
  testConnection,
} from "@/lib/connection-config";
import {
  ApplicationShell,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Input,
  Label,
} from "@whaleit/ui";
import { FormEvent, useState } from "react";

interface ConnectSetupPageProps {
  onConnected: () => void;
}

export function ConnectSetupPage({ onConnected }: ConnectSetupPageProps) {
  const [apiHost, setApiHost] = useState(() => {
    const existing = getConnectionConfig();
    return existing?.apiHost ?? "https://";
  });
  const [apiKey, setApiKey] = useState(() => {
    const existing = getConnectionConfig();
    return existing?.apiKey ?? "";
  });
  const [error, setError] = useState<string | null>(null);
  const [testing, setTesting] = useState(false);

  const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError(null);

    const host = apiHost.trim().replace(/\/+$/, "");
    if (!host || !host.startsWith("http")) {
      setError("Enter a valid server URL (e.g. https://app.whaleit.com)");
      return;
    }
    const key = apiKey.trim();
    if (!key) {
      setError("Enter your API key");
      return;
    }

    setTesting(true);
    const config: ConnectionConfig = { apiHost: host, apiKey: key };
    const result = await testConnection(config);
    setTesting(false);

    if (result.success) {
      setConnectionConfig(config);
      onConnected();
    } else {
      setError(result.error ?? "Connection failed");
    }
  };

  return (
    <ApplicationShell className="fixed inset-0 flex items-center justify-center p-6">
      <div className="w-full max-w-md -translate-y-[5vh]">
        <Card className="w-full border-none bg-transparent shadow-none">
          <CardHeader className="space-y-4 text-center">
            <div className="flex justify-center">
              <img
                src="/logo-vantage.png"
                alt="WhaleIt logo"
                className="h-16 w-16 sm:h-20 sm:w-20"
              />
            </div>
            <div className="space-y-2">
              <CardTitle>Connect to WhaleIt</CardTitle>
              <CardDescription>
                Enter your server URL and API key to get started.
              </CardDescription>
            </div>
          </CardHeader>
          <CardContent>
            <form className="space-y-4" onSubmit={handleSubmit}>
              <div className="space-y-2">
                <Label htmlFor="apiHost">Server URL</Label>
                <Input
                  id="apiHost"
                  type="url"
                  value={apiHost}
                  onChange={(e) => {
                    if (error) setError(null);
                    setApiHost(e.target.value);
                  }}
                  disabled={testing}
                  required
                  placeholder="https://app.whaleit.com"
                  className="h-12 rounded-full shadow-none"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="apiKey">API Key</Label>
                <Input
                  id="apiKey"
                  type="password"
                  value={apiKey}
                  onChange={(e) => {
                    if (error) setError(null);
                    setApiKey(e.target.value);
                  }}
                  disabled={testing}
                  required
                  placeholder="wfk_..."
                  className="h-12 rounded-full shadow-none"
                />
                {error ? (
                  <p className="text-destructive text-sm" role="alert">
                    {error}
                  </p>
                ) : null}
              </div>

              <Button type="submit" className="w-full" disabled={testing}>
                {testing ? "Connecting..." : "Connect"}
              </Button>
            </form>

            <div className="mt-6 border-t pt-4">
              <p className="text-muted-foreground text-center text-xs">
                Create an API key from your WhaleIt web dashboard under Settings → API Keys.
              </p>
            </div>
          </CardContent>
        </Card>
      </div>
    </ApplicationShell>
  );
}
