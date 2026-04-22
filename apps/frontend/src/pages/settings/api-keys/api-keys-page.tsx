import { useMemo, useState } from "react";
import { Separator } from "@whaleit/ui/components/ui/separator";
import { Button } from "@whaleit/ui/components/ui/button";
import { Input } from "@whaleit/ui/components/ui/input";
import { Label } from "@whaleit/ui/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@whaleit/ui/components/ui/dialog";
import { SettingsHeader } from "../settings-header";
import { createApiKeySchema } from "@/lib/schemas";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

interface ApiKey {
  id: string;
  name: string;
  prefix: string;
  createdAt: string;
  lastUsedAt: string | null;
}

interface CreateApiKeyResponse {
  id: string;
  name: string;
  key: string;
}

async function fetchApiKeys(): Promise<ApiKey[]> {
  const res = await fetch("/api/v1/auth/api-keys", { credentials: "same-origin" });
  if (!res.ok) throw new Error("Failed to load API keys");
  return res.json();
}

async function createApiKey(name: string): Promise<CreateApiKeyResponse> {
  const res = await fetch("/api/v1/auth/api-keys", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name }),
    credentials: "same-origin",
  });
  if (!res.ok) throw new Error("Failed to create API key");
  return res.json();
}

async function deleteApiKey(id: string): Promise<void> {
  const res = await fetch(`/api/v1/auth/api-keys/${id}`, {
    method: "DELETE",
    credentials: "same-origin",
  });
  if (!res.ok) throw new Error("Failed to revoke API key");
}

export default function ApiKeysPage() {
  const queryClient = useQueryClient();
  const [newKeyName, setNewKeyName] = useState("");
  const [validationError, setValidationError] = useState<string | null>(null);
  const [createdKey, setCreatedKey] = useState<string | null>(null);

  const { data, isLoading } = useQuery({
    queryKey: ["api-keys"],
    queryFn: fetchApiKeys,
  });

  const createMutation = useMutation({
    mutationFn: createApiKey,
    onSuccess: (response) => {
      setCreatedKey(response.key);
      setNewKeyName("");
      setValidationError(null);
      queryClient.invalidateQueries({ queryKey: ["api-keys"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: deleteApiKey,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["api-keys"] });
    },
  });

  const keys = useMemo(() => data ?? [], [data]);

  const handleCreate = () => {
    setValidationError(null);
    const result = createApiKeySchema.safeParse({ name: newKeyName });
    if (!result.success) {
      setValidationError(result.error.issues[0]?.message ?? "Invalid name");
      return;
    }
    createMutation.mutate(newKeyName);
  };

  const [dialogOpen, setDialogOpen] = useState(false);

  if (isLoading) {
    return (
      <div className="text-foreground space-y-6">
        <SettingsHeader heading="API Keys" text="Manage API keys for programmatic access." />
        <Separator />
      </div>
    );
  }

  return (
    <div className="text-foreground space-y-6">
      <SettingsHeader heading="API Keys" text="Manage API keys for programmatic access.">
        <Dialog
          open={dialogOpen}
          onOpenChange={(open) => {
            setDialogOpen(open);
            if (!open) {
              setCreatedKey(null);
              setValidationError(null);
            }
          }}
        >
          <DialogTrigger asChild>
            <Button size="sm">Create API Key</Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Create API Key</DialogTitle>
              <DialogDescription>
                Give your API key a name to help you identify it later.
              </DialogDescription>
            </DialogHeader>
            {createdKey ? (
              <div className="space-y-3">
                <p className="text-sm">
                  Your API key has been created. Copy it now — you won&apos;t be able to see it
                  again.
                </p>
                <div className="bg-muted rounded-md p-3">
                  <code className="text-xs break-all">{createdKey}</code>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => navigator.clipboard.writeText(createdKey)}
                >
                  Copy to Clipboard
                </Button>
              </div>
            ) : (
              <>
                <div className="space-y-2">
                  <Label htmlFor="keyName">Name</Label>
                  <Input
                    id="keyName"
                    value={newKeyName}
                    onChange={(e) => {
                      setNewKeyName(e.target.value);
                      setValidationError(null);
                    }}
                    placeholder="e.g., My Script Key"
                    maxLength={100}
                  />
                  {validationError && (
                    <p className="text-destructive text-sm">{validationError}</p>
                  )}
                </div>
                <DialogFooter>
                  <Button onClick={handleCreate} disabled={createMutation.isPending}>
                    {createMutation.isPending ? "Creating..." : "Create"}
                  </Button>
                </DialogFooter>
              </>
            )}
          </DialogContent>
        </Dialog>
      </SettingsHeader>
      <Separator />

      {keys.length === 0 ? (
        <p className="text-muted-foreground text-sm">
          No API keys yet. Create one to get started.
        </p>
      ) : (
        <div className="overflow-hidden rounded-lg border">
          {keys.map((key) => (
            <div
              key={key.id}
              className="flex items-center justify-between border-b px-4 py-3 last:border-b-0"
            >
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <span className="font-medium">{key.name}</span>
                  <code className="text-muted-foreground bg-muted rounded px-1.5 py-0.5 text-xs">
                    {key.prefix}...
                  </code>
                </div>
                <p className="text-muted-foreground text-xs">
                  Created {new Date(key.createdAt).toLocaleDateString()}
                  {key.lastUsedAt
                    ? ` · Last used ${new Date(key.lastUsedAt).toLocaleDateString()}`
                    : " · Never used"}
                </p>
              </div>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => deleteMutation.mutate(key.id)}
                disabled={deleteMutation.isPending}
              >
                Revoke
              </Button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
