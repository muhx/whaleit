import { useState } from "react";
import { Button } from "@whaleit/ui/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@whaleit/ui/components/ui/popover";
import { Label } from "@whaleit/ui/components/ui/label";
import { MoneyInput } from "@whaleit/ui";
import { cn } from "@/lib/utils";

interface AmountFilterProps {
  amountMin: number | undefined;
  amountMax: number | undefined;
  onChange: (min: number | undefined, max: number | undefined) => void;
}

export function AmountFilter({ amountMin, amountMax, onChange }: AmountFilterProps) {
  const [open, setOpen] = useState(false);
  const [localMin, setLocalMin] = useState<string>(amountMin?.toString() ?? "");
  const [localMax, setLocalMax] = useState<string>(amountMax?.toString() ?? "");

  function label() {
    if (amountMin == null && amountMax == null) return "Any amount";
    const min = amountMin != null ? `$${amountMin}` : "...";
    const max = amountMax != null ? `$${amountMax}` : "...";
    return `${min} – ${max}`;
  }

  const isActive = amountMin != null || amountMax != null;

  function apply() {
    const min = localMin !== "" ? Number(localMin) : undefined;
    const max = localMax !== "" ? Number(localMax) : undefined;
    onChange(min, max);
    setOpen(false);
  }

  function clear() {
    setLocalMin("");
    setLocalMax("");
    onChange(undefined, undefined);
    setOpen(false);
  }

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
      <PopoverContent className="w-56 p-3" align="start">
        <div className="space-y-3">
          <div className="space-y-1">
            <Label className="text-xs">Min amount</Label>
            <MoneyInput
              value={localMin}
              onChange={(e) => setLocalMin(e.target.value)}
              placeholder="0"
              className="h-8 text-xs"
            />
          </div>
          <div className="space-y-1">
            <Label className="text-xs">Max amount</Label>
            <MoneyInput
              value={localMax}
              onChange={(e) => setLocalMax(e.target.value)}
              placeholder="Any"
              className="h-8 text-xs"
            />
          </div>
          <div className="flex gap-2">
            <Button size="sm" className="flex-1 text-xs" onClick={apply}>
              Apply
            </Button>
            {isActive && (
              <Button variant="ghost" size="sm" className="text-xs" onClick={clear}>
                Clear
              </Button>
            )}
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
}
