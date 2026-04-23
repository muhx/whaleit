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

export function LoginPage() {
  const { login, loginLoading, loginError, clearError, needsVerification } = useAuth();
  const navigate = useNavigate();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!email.trim() || !password.trim()) {
      return;
    }
    try {
      await login(email, password);
      setEmail("");
      setPassword("");
      navigate("/");
    } catch (error) {
      console.error("Login failed", error);
    }
  };

  if (needsVerification) {
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
                <CardTitle>Verify Your Email</CardTitle>
                <CardDescription>
                  Please check your email and click the verification link before signing in.
                </CardDescription>
              </div>
            </CardHeader>
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
              <CardTitle>WhaleIt</CardTitle>
              <CardDescription>Your private portfolio tracker.</CardDescription>
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
                  onChange={(event) => {
                    if (loginError) {
                      clearError();
                    }
                    setEmail(event.target.value);
                  }}
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
                  autoComplete="current-password"
                  onChange={(event) => {
                    if (loginError) {
                      clearError();
                    }
                    setPassword(event.target.value);
                  }}
                  disabled={loginLoading}
                  required
                  placeholder="Enter your password"
                  className="h-12 rounded-full shadow-none"
                />
                {loginError ? (
                  <p className="text-destructive text-sm" role="alert">
                    {loginError}
                  </p>
                ) : null}
              </div>

              <Button type="submit" className="w-full" disabled={loginLoading}>
                {loginLoading ? "Signing in..." : "Sign In"}
              </Button>
            </form>
          </CardContent>
          <CardFooter className="flex flex-col gap-2 text-center text-xs">
            <div className="text-muted-foreground flex gap-1">
              <span>Don&apos;t have an account?</span>
              <Link to="/register" className="text-foreground underline hover:no-underline">
                Sign up
              </Link>
            </div>
            <Link
              to="/forgot-password"
              className="text-muted-foreground underline hover:no-underline"
            >
              Forgot password?
            </Link>
          </CardFooter>
        </Card>
      </div>
    </ApplicationShell>
  );
}
