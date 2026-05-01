// Transaction Template Picker (Phase 4, plan 04-08).
//
// Forked from apps/frontend/src/pages/activity/import/components/template-picker.tsx.
// D-16: user-saved only — no system/built-in templates.
// D-17: on select, validates header signature; mismatch shown in MappingStep.
// D-18: global scope — templates appear for all accounts.
// Supports save-as-new and delete.

import { useState } from "react";
import { Button } from "@whaleit/ui/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
} from "@whaleit/ui/components/ui/command";
import { Icons } from "@whaleit/ui/components/ui/icons";
import { Input } from "@whaleit/ui/components/ui/input";
import { Label } from "@whaleit/ui/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@whaleit/ui/components/ui/popover";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@whaleit/ui/components/ui/alert-dialog";
import { cn } from "@whaleit/ui/lib/utils";
import {
  useTransactionTemplates,
  useSaveTransactionTemplate,
  useDeleteTransactionTemplate,
} from "@/hooks/use-transaction-templates";
import { useTransactionImportMapping } from "../hooks/use-transaction-import-mapping";
import { useTransactionImport } from "../context/transaction-import-context";
import type { CsvFieldMapping } from "@/lib/types/transaction";

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

interface TransactionTemplatePickerProps {
  /** Current CSV headers — needed for save (header signature) and D-17 validation */
  currentHeaders: string[];
  /** Current mapping — needed for save */
  currentMapping: CsvFieldMapping | null;
  className?: string;
}

// ─────────────────────────────────────────────────────────────────────────────
// Main Component
// ─────────────────────────────────────────────────────────────────────────────

export function TransactionTemplatePicker({
  currentHeaders,
  currentMapping,
  className,
}: TransactionTemplatePickerProps) {
  const { state } = useTransactionImport();
  const { applyTemplate, clearTemplate } = useTransactionImportMapping();

  const { data: templates = [] } = useTransactionTemplates();
  const saveTemplate = useSaveTransactionTemplate();
  const deleteTemplate = useDeleteTransactionTemplate();

  const [open, setOpen] = useState(false);
  const [saveDialogOpen, setSaveDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [templateToDelete, setTemplateToDelete] = useState<{ id: string; name: string } | null>(
    null,
  );
  const [newTemplateName, setNewTemplateName] = useState("");

  const selectedId = state.selectedTemplateId;
  const selected = templates.find((t) => t.id === selectedId);

  function handleSelect(templateId: string) {
    const tpl = templates.find((t) => t.id === templateId);
    if (!tpl) return;
    applyTemplate(tpl);
    setOpen(false);
  }

  function handleClear(e: React.MouseEvent | React.KeyboardEvent) {
    e.stopPropagation();
    clearTemplate();
  }

  function handleSave() {
    if (!newTemplateName.trim() || !currentMapping) return;
    saveTemplate.mutate(
      {
        name: newTemplateName.trim(),
        mapping: currentMapping,
        headerSignature: currentHeaders,
      },
      {
        onSuccess: () => {
          setSaveDialogOpen(false);
          setNewTemplateName("");
        },
      },
    );
  }

  function handleDeleteRequest(e: React.MouseEvent, id: string, name: string) {
    e.stopPropagation();
    setTemplateToDelete({ id, name });
    setDeleteDialogOpen(true);
  }

  function handleDeleteConfirm() {
    if (!templateToDelete) return;
    deleteTemplate.mutate(templateToDelete.id, {
      onSuccess: () => {
        if (selectedId === templateToDelete.id) {
          clearTemplate();
        }
        setTemplateToDelete(null);
        setDeleteDialogOpen(false);
      },
    });
  }

  return (
    <>
      <div className={cn("flex items-center gap-2", className)}>
        <Popover open={open} onOpenChange={setOpen}>
          <PopoverTrigger asChild>
            <Button
              variant="outline"
              role="combobox"
              aria-expanded={open}
              className="flex-1 justify-between rounded-lg font-normal"
            >
              {selected ? (
                <span className="flex items-center gap-2 truncate">
                  <Icons.User className="text-muted-foreground h-3.5 w-3.5 shrink-0" />
                  {selected.name}
                </span>
              ) : (
                <span className="text-muted-foreground">Apply saved template…</span>
              )}
              <div className="flex shrink-0 items-center gap-1">
                {selectedId && (
                  <span
                    role="button"
                    tabIndex={0}
                    onClick={handleClear}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" || e.key === " ") handleClear(e);
                    }}
                    className="text-muted-foreground hover:text-foreground rounded-sm p-0.5 transition-colors"
                    aria-label="Clear template"
                  >
                    <Icons.X className="h-3.5 w-3.5" />
                  </span>
                )}
                <Icons.ChevronDown className="text-muted-foreground h-4 w-4" />
              </div>
            </Button>
          </PopoverTrigger>
          <PopoverContent className="w-[var(--radix-popover-trigger-width)] p-0" align="start">
            <Command>
              <CommandInput placeholder="Search templates…" className="h-9" />
              <CommandEmpty>
                {templates.length === 0 ? (
                  <div className="text-muted-foreground py-3 text-center text-sm">
                    Save a template after mapping to reuse it next time.
                  </div>
                ) : (
                  <div className="text-muted-foreground py-2 text-center text-sm">
                    No matching templates.
                  </div>
                )}
              </CommandEmpty>
              {templates.length > 0 && (
                <CommandGroup heading="Saved templates">
                  {templates.map((t) => (
                    <CommandItem
                      key={t.id}
                      value={t.name}
                      onSelect={() => handleSelect(t.id)}
                      className="flex items-center gap-2"
                    >
                      <Icons.User className="text-muted-foreground h-3.5 w-3.5 shrink-0" />
                      <span className="flex-1">{t.name}</span>
                      <Icons.Check
                        className={cn(
                          "h-4 w-4 shrink-0",
                          selectedId === t.id ? "opacity-100" : "opacity-0",
                        )}
                      />
                      <span
                        role="button"
                        tabIndex={0}
                        onClick={(e) => handleDeleteRequest(e, t.id, t.name)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter" || e.key === " ")
                            handleDeleteRequest(e as unknown as React.MouseEvent, t.id, t.name);
                        }}
                        className="text-muted-foreground hover:text-destructive ml-1 rounded-sm p-0.5 transition-colors"
                        aria-label={`Delete template ${t.name}`}
                      >
                        <Icons.Trash className="h-3.5 w-3.5" />
                      </span>
                    </CommandItem>
                  ))}
                </CommandGroup>
              )}
            </Command>
          </PopoverContent>
        </Popover>

        {/* Save current mapping as new template */}
        <Button
          variant="outline"
          size="sm"
          disabled={!currentMapping}
          onClick={() => setSaveDialogOpen(true)}
          className="shrink-0"
        >
          <Icons.Save className="mr-1.5 h-3.5 w-3.5" />
          Save template
        </Button>
      </div>

      {/* Save template dialog */}
      <AlertDialog open={saveDialogOpen} onOpenChange={setSaveDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Save as template</AlertDialogTitle>
            <AlertDialogDescription>
              Give this column mapping a name so you can reuse it for future imports.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <div className="py-2">
            <Label htmlFor="template-name" className="mb-1.5 block text-sm">
              Template name
            </Label>
            <Input
              id="template-name"
              value={newTemplateName}
              onChange={(e) => setNewTemplateName(e.target.value)}
              placeholder="e.g. Chase Checking CSV"
              onKeyDown={(e) => {
                if (e.key === "Enter") handleSave();
              }}
            />
          </div>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleSave}
              disabled={!newTemplateName.trim() || saveTemplate.isPending}
            >
              {saveTemplate.isPending ? "Saving…" : "Save"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Delete confirmation dialog */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete template?</AlertDialogTitle>
            <AlertDialogDescription>
              &ldquo;{templateToDelete?.name}&rdquo; will be permanently deleted. This cannot be
              undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDeleteConfirm}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              disabled={deleteTemplate.isPending}
            >
              {deleteTemplate.isPending ? "Deleting…" : "Delete"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
