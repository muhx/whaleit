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
import { useTaxonomy } from "@/hooks/use-taxonomies";

const TRANSACTION_TAXONOMY_ID = "sys_taxonomy_transaction_categories";

interface CategoryFilterProps {
  selected: string[];
  onChange: (ids: string[]) => void;
}

export function CategoryFilter({ selected, onChange }: CategoryFilterProps) {
  const [open, setOpen] = useState(false);
  const { data: taxonomy } = useTaxonomy(TRANSACTION_TAXONOMY_ID);
  const categories = taxonomy?.categories ?? [];

  function toggle(id: string) {
    if (selected.includes(id)) {
      onChange(selected.filter((s) => s !== id));
    } else {
      onChange([...selected, id]);
    }
  }

  function label() {
    if (selected.length === 0) return "Any category";
    if (selected.length === 1) {
      return categories.find((c) => c.id === selected[0])?.name ?? "1 category";
    }
    return `${selected.length} categories`;
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
          <CommandInput placeholder="Search categories..." />
          <CommandList>
            <CommandEmpty>No categories found.</CommandEmpty>
            <CommandGroup>
              {categories.map((cat) => (
                <CommandItem
                  key={cat.id}
                  value={cat.name}
                  onSelect={() => toggle(cat.id)}
                  className="flex items-center gap-2"
                >
                  <Icons.Check
                    className={cn(
                      "size-4",
                      selected.includes(cat.id) ? "opacity-100" : "opacity-0",
                    )}
                  />
                  <span>{cat.name}</span>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
