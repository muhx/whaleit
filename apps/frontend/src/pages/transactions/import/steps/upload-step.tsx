// Transaction Upload Step (Phase 4, plan 04-08).
//
// Accepts CSV + OFX files. Detects format by sniffing first 500 bytes.
// CSV: parses headers + first 500 rows client-side (preview), advances to Mapping.
// OFX: sets format=OFX in context, advances directly to Review (D-19).

import { useCallback, useRef, useState } from "react";
import { useTransactionImport } from "../context/transaction-import-context";
import { computeTransactionFieldMappings } from "../hooks/use-transaction-import-mapping";
import type { CsvFieldMapping } from "@/lib/types/transaction";

const ACCEPTED_MIME_TYPES = [
  "text/csv",
  "application/vnd.ms-excel",
  "application/x-ofx",
  "application/octet-stream",
  ".csv",
  ".ofx",
];

function detectFormat(bytes: Uint8Array): "CSV" | "OFX" {
  const preview = new TextDecoder("utf-8", { fatal: false }).decode(bytes.slice(0, 500));
  if (/OFXHEADER:/i.test(preview) || /<OFX>/i.test(preview)) {
    return "OFX";
  }
  return "CSV";
}

function parseCsvPreview(text: string): { headers: string[]; rows: string[][] } {
  const lines = text.split(/\r?\n/).filter((l) => l.trim().length > 0);
  if (lines.length === 0) return { headers: [], rows: [] };

  function splitLine(line: string): string[] {
    const result: string[] = [];
    let current = "";
    let inQuotes = false;
    for (let i = 0; i < line.length; i++) {
      const ch = line[i];
      if (ch === '"') {
        inQuotes = !inQuotes;
      } else if (ch === "," && !inQuotes) {
        result.push(current.trim());
        current = "";
      } else {
        current += ch;
      }
    }
    result.push(current.trim());
    return result;
  }

  const headers = splitLine(lines[0] ?? "");
  const MAX_PREVIEW = 500;
  const rows = lines.slice(1, MAX_PREVIEW + 1).map((l) => splitLine(l));

  return { headers, rows };
}

function buildDefaultMapping(headers: string[]): CsvFieldMapping {
  const auto = computeTransactionFieldMappings(headers);
  return {
    dateColumn: auto.dateColumn ?? "",
    amountColumn: auto.amountColumn ?? null,
    debitColumn: auto.debitColumn ?? null,
    creditColumn: auto.creditColumn ?? null,
    payeeColumn: auto.payeeColumn ?? "",
    categoryColumn: auto.categoryColumn ?? null,
    notesColumn: auto.notesColumn ?? null,
    currencyColumn: auto.currencyColumn ?? null,
    externalIdColumn: auto.externalIdColumn ?? null,
    dateFormat: "auto",
    decimalSeparator: ".",
    thousandsSeparator: null,
  };
}

export function UploadStep() {
  const { state, dispatch } = useTransactionImport();
  const [dragOver, setDragOver] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const processFile = useCallback(
    async (file: File) => {
      setError(null);
      try {
        const buffer = await file.arrayBuffer();
        const bytes = new Uint8Array(buffer);
        const format = detectFormat(bytes);

        dispatch({ type: "SET_FILE", payload: file });
        dispatch({ type: "SET_FORMAT", payload: format });

        if (format === "OFX") {
          // D-19: OFX skips Mapping step entirely
          dispatch({ type: "SET_STEP", payload: "review" });
        } else {
          const text = new TextDecoder("utf-8").decode(bytes);
          const { headers, rows } = parseCsvPreview(text);
          dispatch({
            type: "SET_PARSED_CSV",
            payload: { headers, rows, rawText: text },
          });
          const mapping = buildDefaultMapping(headers);
          dispatch({ type: "SET_MAPPING", payload: mapping });
          dispatch({ type: "SET_STEP", payload: "mapping" });
        }
      } catch {
        setError("Could not read file. Please try again.");
      }
    },
    [dispatch],
  );

  const handleInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) void processFile(file);
    },
    [processFile],
  );

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setDragOver(false);
      const file = e.dataTransfer.files[0];
      if (file) void processFile(file);
    },
    [processFile],
  );

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold">Upload your bank file</h2>
        <p className="text-muted-foreground mt-1 text-sm">
          Supports CSV and OFX files. OFX files are imported directly — no column mapping needed.
        </p>
      </div>

      {/* File dropzone */}
      <div
        role="button"
        tabIndex={0}
        aria-label="Drop a CSV or OFX file here, or click to browse"
        onDrop={handleDrop}
        onDragOver={(e) => {
          e.preventDefault();
          setDragOver(true);
        }}
        onDragLeave={() => setDragOver(false)}
        onClick={() => inputRef.current?.click()}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") inputRef.current?.click();
        }}
        className={`cursor-pointer rounded-lg border-2 border-dashed p-10 text-center transition-colors ${
          dragOver ? "border-primary bg-primary/5" : "border-border hover:border-primary/50"
        }`}
      >
        <input
          ref={inputRef}
          type="file"
          accept={ACCEPTED_MIME_TYPES.join(",")}
          className="hidden"
          onChange={handleInputChange}
        />
        <p className="text-muted-foreground text-sm">
          {state.file ? state.file.name : "Drop a .csv or .ofx file here, or click to browse"}
        </p>
        {state.file && (
          <p className="text-muted-foreground mt-1 text-xs">
            Format detected: <strong>{state.format}</strong>
          </p>
        )}
      </div>

      {error && <p className="text-destructive text-sm">{error}</p>}
    </div>
  );
}
