import { useAuth } from "@/context/auth-context";
import {
  ApplicationShell,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  Input,
  Label,
} from "@whaleit/ui";
import { FormEvent, useState } from "react";
import { Link } from "react-router-dom";
import { forgotPasswordSchema } from "@/lib/schemas";

export function ForgotPasswordPage() {
  const { forgotPassword, loginLoading, loginError, clearError } = useAuth();
  const [email, setEmail] = useState("");
  const [sent, setSent] = useState(false);
  const [validationError, setValidationError] = useState<string | null>(null);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    clearError();
    setValidationError(null);

    const result = forgotPasswordSchema.safeParse({ email });
    if (!result.success) {
      setValidationError(result.error.issues[0]?.message ?? "Invalid email");
      return;
    }

    try {
      await forgotPassword(email);
      setSent(true);
    } catch (error) {
      console.error("Forgot password failed", error);
    }
  };

  if (sent) {
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
                  If an account exists with that email, we&apos;ve sent a password reset link.
                </CardDescription>
              </div>
            </CardHeader>
            <CardFooter className="flex flex-col gap-2 text-center text-xs">
              <Link to="/login" className="text-muted-foreground underline hover:no-underline">
                Back to sign in
              </Link>
            </CardFooter>
          </Card>
        </div>
      </ApplicationShell>
    );
  }

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
              <CardTitle>Reset Password</CardTitle>
              <CardDescription>
                Enter your email and we&apos;ll send you a reset link.
              </CardDescription>
            </div>
          </CardHeader>
          <CardContent>
            <form className="space-y-4" onSubmit={handleSubmit}>
              <div className="space-y-2">
                <Label htmlFor="email">Email</Label>
                <Input
                  id="email"
                  type="email"
                  value={email}
                  autoComplete="email"
                  onChange={(event) => setEmail(event.target.value)}
                  disabled={loginLoading}
                  required
                  placeholder="you@example.com"
                  className="h-12 rounded-full shadow-none"
                />
                {validationError || loginError ? (
                  <p className="text-destructive text-sm" role="alert">
                    {validationError || loginError}
                  </p>
                ) : null}
              </div>
              <Button type="submit" className="w-full" disabled={loginLoading}>
                {loginLoading ? "Sending..." : "Send Reset Link"}
              </Button>
            </form>
          </CardContent>
          <CardFooter className="flex flex-col gap-2 text-center text-xs">
            <Link to="/login" className="text-muted-foreground underline hover:no-underline">
              Back to sign in
            </Link>
          </CardFooter>
        </Card>
      </div>
    </ApplicationShell>
  );
}
