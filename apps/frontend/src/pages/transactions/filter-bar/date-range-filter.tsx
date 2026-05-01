import { useState } from "react";
import { subDays, format as formatDate } from "date-fns";
import { Button } from "@whaleit/ui/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@whaleit/ui/components/ui/popover";
import { Input } from "@whaleit/ui/components/ui/input";
import { Label } from "@whaleit/ui/components/ui/label";
import { cn } from "@/lib/utils";

interface DateRangeFilterProps {
  dateFrom: string | undefined;
  dateTo: string | undefined;
  onChange: (from: string | undefined, to: string | undefined) => void;
}

export function DateRangeFilter({ dateFrom, dateTo, onChange }: DateRangeFilterProps) {
  const [open, setOpen] = useState(false);

  // Default: last 30 days
  const defaultFrom = formatDate(subDays(new Date(), 30), "yyyy-MM-dd");
  const defaultTo = formatDate(new Date(), "yyyy-MM-dd");

  const from = dateFrom ?? defaultFrom;
  const to = dateTo ?? defaultTo;

  function label() {
    if (!dateFrom && !dateTo) return "Last 30 days";
    const f = dateFrom ? formatDate(new Date(dateFrom + "T00:00:00"), "MMM d") : "...";
    const t = dateTo ? formatDate(new Date(dateTo + "T00:00:00"), "MMM d") : "...";
    return `${f} – ${t}`;
  }

  const isActive = !!dateFrom || !!dateTo;

  function apply(f: string, t: string) {
    onChange(f || undefined, t || undefined);
    setOpen(false);
  }

  function clear() {
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
      <PopoverContent className="w-64 p-3" align="start">
        <div className="space-y-3">
          <div className="space-y-1">
            <Label className="text-xs">From</Label>
            <Input
              type="date"
              value={from}
              onChange={(e) => apply(e.target.value, to)}
              className="h-8 text-xs"
            />
          </div>
          <div className="space-y-1">
            <Label className="text-xs">To</Label>
            <Input
              type="date"
              value={to}
              onChange={(e) => apply(from, e.target.value)}
              className="h-8 text-xs"
            />
          </div>
          {isActive && (
            <Button variant="ghost" size="sm" className="w-full text-xs" onClick={clear}>
              Clear dates
            </Button>
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
}
