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
import { Link, useNavigate } from "react-router-dom";
import { registerSchema } from "@/lib/schemas";

export function RegisterPage() {
  const { register, loginLoading, loginError, clearError } = useAuth();
  const navigate = useNavigate();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [validationError, setValidationError] = useState<string | null>(null);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    clearError();
    setValidationError(null);

    const result = registerSchema.safeParse({ email, password, confirmPassword, displayName });
    if (!result.success) {
      setValidationError(result.error.issues[0]?.message ?? "Validation failed");
      return;
    }

    try {
      await register(email, password, displayName || undefined);
      navigate("/register/pending");
    } catch (error) {
      console.error("Registration failed", error);
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
              <CardTitle>Create Account</CardTitle>
              <CardDescription>Sign up to start tracking your portfolio.</CardDescription>
            </div>
          </CardHeader>
          <CardContent>
            <form className="space-y-4" onSubmit={handleSubmit}>
              <div className="space-y-2">
                <Label htmlFor="displayName">Display Name (optional)</Label>
                <Input
                  id="displayName"
                  type="text"
                  value={displayName}
                  onChange={(event) => setDisplayName(event.target.value)}
                  disabled={loginLoading}
                  placeholder="Your name"
                  className="h-12 rounded-full shadow-none"
                />
              </div>
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
              </div>
              <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
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
                <Label htmlFor="confirmPassword">Confirm Password</Label>
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
              {(validationError || loginError) ? (
                <p className="text-destructive text-sm" role="alert">
                  {validationError || loginError}
                </p>
              ) : null}
              <Button type="submit" className="w-full" disabled={loginLoading}>
                {loginLoading ? "Creating account..." : "Create Account"}
              </Button>
            </form>
          </CardContent>
          <CardFooter className="flex flex-col gap-2 text-center text-xs">
            <div className="text-muted-foreground flex gap-1">
              <span>Already have an account?</span>
              <Link to="/login" className="text-foreground underline hover:no-underline">
                Sign in
              </Link>
            </div>
          </CardFooter>
        </Card>
      </div>
    </ApplicationShell>
  );
}
