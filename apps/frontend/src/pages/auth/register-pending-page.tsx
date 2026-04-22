import {
  ApplicationShell,
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@whaleit/ui";
import { Link } from "react-router-dom";

export function RegisterPendingPage() {
  return (
    <ApplicationShell className="fixed inset-0 flex items-center justify-center p-6">
      <div className="w-full max-w-md -translate-y-[5vh]">
        <Card className="w-full border-none bg-transparent shadow-none">
          <CardHeader className="space-y-4 text-center">
            <div className="flex justify-center">
              <img
                src="/logo-vantage.png"
                alt="WhaleIt logo vantage"
                className="h-16 w-16 sm:h-20 sm:w-20"
              />
            </div>
            <div className="space-y-2">
              <CardTitle>Check Your Email</CardTitle>
              <CardDescription>
                We&apos;ve sent a verification link to your email address. Please click it to
                verify your account, then sign in.
              </CardDescription>
            </div>
            <Link
              to="/login"
              className="text-muted-foreground underline hover:no-underline"
            >
              Back to sign in
            </Link>
          </CardHeader>
        </Card>
      </div>
    </ApplicationShell>
  );
}
