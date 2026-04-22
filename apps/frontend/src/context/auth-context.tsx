import { isWeb } from "@/adapters";
import { setUnauthorizedHandler } from "@/lib/auth-token";
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";

interface User {
  id: string;
  email: string;
  displayName: string | null;
  emailVerified: boolean;
}

interface AuthContextValue {
  requiresAuth: boolean;
  isAuthenticated: boolean;
  statusLoading: boolean;
  loginLoading: boolean;
  loginError: string | null;
  user: User | null;
  needsVerification: boolean;
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;
  clearError: () => void;
  register: (email: string, password: string, displayName?: string) => Promise<void>;
  verifyEmail: (token: string) => Promise<void>;
  forgotPassword: (email: string) => Promise<void>;
  resetPassword: (token: string, newPassword: string) => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [requiresAuth, setRequiresAuth] = useState(false);
  const [statusLoading, setStatusLoading] = useState(isWeb);
  const [cookieSession, setCookieSession] = useState(false);
  const [loginLoading, setLoginLoading] = useState(false);
  const [loginError, setLoginError] = useState<string | null>(null);
  const [user, setUser] = useState<User | null>(null);
  const [needsVerification, setNeedsVerification] = useState(false);
  const cookieSessionRef = useRef(false);

  useEffect(() => {
    cookieSessionRef.current = cookieSession;
  }, [cookieSession]);

  useEffect(() => {
    if (!isWeb) {
      setStatusLoading(false);
      return;
    }
    let cancelled = false;
    const loadStatus = async () => {
      try {
        const response = await fetch("/api/v1/auth/status", {
          credentials: "same-origin",
        });
        if (!response.ok) {
          throw new Error(`Failed to check authentication status: ${response.status}`);
        }
        const data = (await response.json()) as {
          requiresPassword: boolean;
          multiUser?: boolean;
        };
        if (cancelled) return;
        const needsAuth = Boolean(data?.requiresPassword);
        setRequiresAuth(needsAuth);

        if (needsAuth) {
          try {
            const meRes = await fetch("/api/v1/auth/me", {
              credentials: "same-origin",
            });
            if (meRes.status === 403) {
              const verifyHeader = meRes.headers.get("X-Verification-Required");
              if (verifyHeader === "true" && !cancelled) {
                setNeedsVerification(true);
              }
            } else if (meRes.ok && !cancelled) {
              const meData = (await meRes.json()) as {
                authenticated: boolean;
                user?: { id: string; email: string; displayName?: string; emailVerified?: boolean };
              };
              if (meData.user) {
                setUser({
                  id: meData.user.id,
                  email: meData.user.email,
                  displayName: meData.user.displayName ?? null,
                  emailVerified: meData.user.emailVerified ?? true,
                });
              }
              setCookieSession(true);
            }
          } catch {
            // No valid session, user will need to log in
          }
        }
      } catch (error) {
        console.error("Failed to load authentication status", error);
        if (!cancelled) {
          setRequiresAuth(false);
        }
      } finally {
        if (!cancelled) {
          setStatusLoading(false);
        }
      }
    };

    void loadStatus();
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const handler = () => {
      const hadSession = cookieSessionRef.current;
      setCookieSession(false);
      setUser(null);
      if (hadSession) {
        setLoginError("Session expired. Please sign in again.");
      }
    };
    setUnauthorizedHandler(handler);
    return () => {
      setUnauthorizedHandler(null);
    };
  }, []);

  const login = useCallback(async (email: string, password: string) => {
    setLoginLoading(true);
    setLoginError(null);
    try {
      const response = await fetch("/api/v1/auth/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, password }),
        credentials: "same-origin",
      });
      if (!response.ok) {
        if (response.status === 404) {
          setRequiresAuth(false);
        }
        if (response.status === 403) {
          const verifyHeader = response.headers.get("X-Verification-Required");
          if (verifyHeader === "true") {
            setNeedsVerification(true);
            throw new Error("Please verify your email before signing in.");
          }
        }
        let message = "Invalid email or password";
        try {
          const body = await response.json();
          message = body?.message ?? message;
        } catch (parseError) {
          console.error("Failed to parse login error", parseError);
        }
        throw new Error(message);
      }
      // Fetch user info after login
      try {
        const meRes = await fetch("/api/v1/auth/me", { credentials: "same-origin" });
        if (meRes.ok) {
          const meData = (await meRes.json()) as {
            user?: { id: string; email: string; displayName?: string; emailVerified?: boolean };
          };
          if (meData.user) {
            setUser({
              id: meData.user.id,
              email: meData.user.email,
              displayName: meData.user.displayName ?? null,
              emailVerified: meData.user.emailVerified ?? true,
            });
          }
        }
      } catch {
        // User info fetch failed, but login succeeded
      }
      setCookieSession(true);
      setNeedsVerification(false);
      setLoginError(null);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Login failed";
      setCookieSession(false);
      setLoginError(message);
      throw error;
    } finally {
      setLoginLoading(false);
    }
  }, []);

  const register = useCallback(
    async (email: string, password: string, displayName?: string) => {
      setLoginLoading(true);
      setLoginError(null);
      try {
        const response = await fetch("/api/v1/auth/register", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ email, password, displayName }),
          credentials: "same-origin",
        });
        if (!response.ok) {
          let message = "Registration failed";
          try {
            const body = await response.json();
            message = body?.message ?? message;
          } catch {
            // ignore
          }
          throw new Error(message);
        }
      } catch (error) {
        const message = error instanceof Error ? error.message : "Registration failed";
        setLoginError(message);
        throw error;
      } finally {
        setLoginLoading(false);
      }
    },
    [],
  );

  const verifyEmail = useCallback(async (token: string) => {
    setLoginLoading(true);
    setLoginError(null);
    try {
      const response = await fetch("/api/v1/auth/verify", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ token }),
        credentials: "same-origin",
      });
      if (!response.ok) {
        let message = "Email verification failed";
        try {
          const body = await response.json();
          message = body?.message ?? message;
        } catch {
          // ignore
        }
        throw new Error(message);
      }
      setNeedsVerification(false);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Email verification failed";
      setLoginError(message);
      throw error;
    } finally {
      setLoginLoading(false);
    }
  }, []);

  const forgotPassword = useCallback(async (email: string) => {
    setLoginLoading(true);
    setLoginError(null);
    try {
      const response = await fetch("/api/v1/auth/forgot-password", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email }),
        credentials: "same-origin",
      });
      if (!response.ok) {
        let message = "Failed to send reset email";
        try {
          const body = await response.json();
          message = body?.message ?? message;
        } catch {
          // ignore
        }
        throw new Error(message);
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to send reset email";
      setLoginError(message);
      throw error;
    } finally {
      setLoginLoading(false);
    }
  }, []);

  const resetPassword = useCallback(async (token: string, newPassword: string) => {
    setLoginLoading(true);
    setLoginError(null);
    try {
      const response = await fetch("/api/v1/auth/reset-password", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ token, password: newPassword }),
        credentials: "same-origin",
      });
      if (!response.ok) {
        let message = "Password reset failed";
        try {
          const body = await response.json();
          message = body?.message ?? message;
        } catch {
          // ignore
        }
        throw new Error(message);
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : "Password reset failed";
      setLoginError(message);
      throw error;
    } finally {
      setLoginLoading(false);
    }
  }, []);

  const logout = useCallback(() => {
    if (isWeb) {
      fetch("/api/v1/auth/logout", {
        method: "POST",
        credentials: "same-origin",
      }).catch(() => {});
    }
    setCookieSession(false);
    setUser(null);
    setLoginError(null);
  }, []);

  const clearError = useCallback(() => setLoginError(null), []);

  const value = useMemo<AuthContextValue>(
    () => ({
      requiresAuth,
      isAuthenticated: !requiresAuth || cookieSession,
      statusLoading,
      loginLoading,
      loginError,
      user,
      needsVerification,
      login,
      logout,
      clearError,
      register,
      verifyEmail,
      forgotPassword,
      resetPassword,
    }),
    [
      requiresAuth,
      cookieSession,
      statusLoading,
      loginLoading,
      loginError,
      user,
      needsVerification,
      login,
      logout,
      clearError,
      register,
      verifyEmail,
      forgotPassword,
      resetPassword,
    ],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export const useAuth = () => {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return ctx;
};

export function AuthGate({ children, fallback }: { children: ReactNode; fallback: ReactNode }) {
  const { requiresAuth, isAuthenticated, statusLoading } = useAuth();

  if (statusLoading) {
    return (
      <div className="bg-background text-muted-foreground flex min-h-screen items-center justify-center">
        Checking authentication...
      </div>
    );
  }

  if (requiresAuth && !isAuthenticated) {
    return <>{fallback}</>;
  }

  return <>{children}</>;
}
