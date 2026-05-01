import { AlertCircle } from "lucide-react";
import { Button } from "@whaleit/ui/components/ui/button";
import { Link } from "react-router-dom";

interface DuplicateBannerProps {
  pendingCount: number;
  onReview?: () => void;
}

export function DuplicateBanner({ pendingCount, onReview }: DuplicateBannerProps) {
  if (pendingCount <= 0) return null;

  return (
    <div
      role="alert"
      className="border-warning/30 bg-warning/10 mx-4 my-2 flex items-center justify-between rounded-md border px-4 py-2"
      data-testid="duplicate-banner"
    >
      <div className="flex items-center gap-2 text-sm">
        <AlertCircle className="text-warning size-4" aria-hidden="true" />
        <span>
          {pendingCount} possible {pendingCount === 1 ? "duplicate" : "duplicates"} from your last
          import
        </span>
      </div>
      {onReview ? (
        <Button variant="ghost" size="sm" onClick={onReview}>
          Review duplicates →
        </Button>
      ) : (
        <Button variant="ghost" size="sm" asChild>
          <Link to="/transactions/duplicates">Review duplicates →</Link>
        </Button>
      )}
    </div>
  );
}
