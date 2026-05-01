import { useState } from "react";
import { Button } from "@whaleit/ui/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@whaleit/ui/components/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "@whaleit/ui/components/ui/popover";
import { cn } from "@/lib/utils";
import { Icons } from "@whaleit/ui/components/ui/icons";
import type { Account } from "@/lib/types";

interface AccountFilterProps {
  accounts: Account[];
  selected: string[];
  onChange: (ids: string[]) => void;
}

export function AccountFilter({ accounts, selected, onChange }: AccountFilterProps) {
  const [open, setOpen] = useState(false);

  function toggle(id: string) {
    if (selected.includes(id)) {
      onChange(selected.filter((s) => s !== id));
    } else {
      onChange([...selected, id]);
    }
  }

  function label() {
    if (selected.length === 0) return "All accounts";
    if (selected.length === 1) {
      return accounts.find((a) => a.id === selected[0])?.name ?? "1 account";
    }
    return `${selected.length} accounts`;
  }

  const isActive = selected.length > 0;

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          size="sm"
          data-state={isActive ? "on" : "off"}
          className={cn("h-8 rounded-full px-3 text-xs", isActive && "border-primary text-primary")}
        >
          {label()}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-56 p-0" align="start">
        <Command>
          <CommandInput placeholder="Search accounts..." />
          <CommandList>
            <CommandEmpty>No accounts found.</CommandEmpty>
            <CommandGroup>
              {accounts.map((acc) => (
                <CommandItem
                  key={acc.id}
                  value={acc.name}
                  onSelect={() => toggle(acc.id)}
                  className="flex items-center gap-2"
                >
                  <Icons.Check
                    className={cn(
                      "size-4",
                      selected.includes(acc.id) ? "opacity-100" : "opacity-0",
                    )}
                  />
                  <span>{acc.name}</span>
                  <span className="text-muted-foreground ml-auto text-xs">{acc.currency}</span>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
