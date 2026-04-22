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

export function VerifyEmailPage() {
  const { verifyEmail, loginLoading, loginError, clearError } = useAuth();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const tokenFromUrl = searchParams.get("token") ?? "";
  const [token, setToken] = useState(tokenFromUrl);
  const [success, setSuccess] = useState(false);

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    clearError();
    try {
      await verifyEmail(token);
      setSuccess(true);
      setTimeout(() => navigate("/login"), 3000);
    } catch (error) {
      console.error("Verification failed", error);
    }
  };

  if (success) {
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
                <CardTitle>Email Verified</CardTitle>
                <CardDescription>
                  Your email has been verified. Redirecting to sign in...
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
              <CardTitle>Verify Your Email</CardTitle>
              <CardDescription>
                Enter the verification token sent to your email.
              </CardDescription>
            </div>
          </CardHeader>
          <CardContent>
            <form className="space-y-4" onSubmit={handleSubmit}>
              <div className="space-y-2">
                <Label htmlFor="token">Verification Token</Label>
                <Input
                  id="token"
                  type="text"
                  value={token}
                  onChange={(event) => setToken(event.target.value)}
                  disabled={loginLoading}
                  required
                  placeholder="Paste your token here"
                  className="h-12 rounded-full shadow-none"
                />
                {loginError ? (
                  <p className="text-destructive text-sm" role="alert">
                    {loginError}
                  </p>
                ) : null}
              </div>
              <Button type="submit" className="w-full" disabled={loginLoading || !token.trim()}>
                {loginLoading ? "Verifying..." : "Verify Email"}
              </Button>
            </form>
          </CardContent>
          <CardFooter className="flex flex-col gap-2 text-center text-xs">
            <Link
              to="/login"
              className="text-muted-foreground underline hover:no-underline"
            >
              Back to sign in
            </Link>
          </CardFooter>
        </Card>
      </div>
    </ApplicationShell>
  );
}
