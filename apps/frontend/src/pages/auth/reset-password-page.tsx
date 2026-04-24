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
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { resetPasswordSchema } from "@/lib/schemas";

export function ResetPasswordPage() {
  const { resetPassword, loginLoading, loginError, clearError } = useAuth();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const tokenFromUrl = searchParams.get("token") ?? "";
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [validationError, setValidationError] = useState<string | null>(null);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    clearError();
    setValidationError(null);

    const result = resetPasswordSchema.safeParse({
      token: tokenFromUrl,
      password,
      confirmPassword,
    });
    if (!result.success) {
      setValidationError(result.error.issues[0]?.message ?? "Validation failed");
      return;
    }

    try {
      await resetPassword(tokenFromUrl, password);
      navigate("/login");
    } catch (error) {
      console.error("Password reset failed", error);
    }
  };

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
              <CardTitle>Set New Password</CardTitle>
              <CardDescription>Enter your new password below.</CardDescription>
            </div>
          </CardHeader>
          <CardContent>
            <form className="space-y-4" onSubmit={handleSubmit}>
              <div className="space-y-2">
                <Label htmlFor="password">New Password</Label>
                <Input
                  id="password"
                  type="password"
                  value={password}
                  autoComplete="new-password"
                  onChange={(event) => setPassword(event.target.value)}
                  disabled={loginLoading}
                  required
                  placeholder="At least 8 characters"
                  className="h-12 rounded-full shadow-none"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="confirmPassword">Confirm New Password</Label>
                <Input
                  id="confirmPassword"
                  type="password"
                  value={confirmPassword}
                  autoComplete="new-password"
                  onChange={(event) => setConfirmPassword(event.target.value)}
                  disabled={loginLoading}
                  required
                  placeholder="Re-enter your password"
                  className="h-12 rounded-full shadow-none"
                />
              </div>
              {validationError || loginError ? (
                <p className="text-destructive text-sm" role="alert">
                  {validationError || loginError}
                </p>
              ) : null}
              <Button type="submit" className="w-full" disabled={loginLoading}>
                {loginLoading ? "Resetting..." : "Reset Password"}
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
