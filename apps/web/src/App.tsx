import {
  type AuthSessionResponse,
  getMe,
  getMeta,
  login,
  logout,
  register,
} from '@voxnexus/api-client';
import type { FormEvent } from 'react';
import { useEffect, useState } from 'react';
import { AuthContext } from './auth';
import { MetaProvider } from './meta';
import { PresenceProvider } from './presence';
import { Shell } from './Shell';

const credentials = { credentials: 'include' as const };

type AuthView = 'login' | 'register';

function pathToAuthView(pathname: string): AuthView {
  return pathname === '/register' ? 'register' : 'login';
}

function navigate(path: string) {
  window.history.pushState({}, '', path);
  window.dispatchEvent(new PopStateEvent('popstate'));
}

export function App() {
  const [session, setSession] = useState<AuthSessionResponse | null>(null);
  const [sessionKnown, setSessionKnown] = useState(false);
  const [authView, setAuthView] = useState<AuthView>(() =>
    pathToAuthView(window.location.pathname),
  );

  useEffect(() => {
    const onPop = () => setAuthView(pathToAuthView(window.location.pathname));
    window.addEventListener('popstate', onPop);
    return () => window.removeEventListener('popstate', onPop);
  }, []);

  async function refresh() {
    const result = await getMe(credentials);
    setSession(result.data ?? null);
  }

  useEffect(() => {
    let cancelled = false;
    getMe(credentials)
      .then((result) => {
        if (cancelled) {
          return;
        }
        setSession(result.data ?? null);
        setSessionKnown(true);
      })
      .catch(() => {
        if (!cancelled) {
          setSession(null);
          setSessionKnown(true);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  async function signOut() {
    await logout(credentials);
    setSession(null);
    navigate('/login');
  }

  if (!sessionKnown) {
    return (
      <div className="grid h-full w-full place-items-center bg-app text-ink-2">
        <p className="font-mono text-xs tracking-wide">Loading…</p>
      </div>
    );
  }

  if (!session) {
    return (
      <MetaProvider>
        <AuthScreen
          mode={authView}
          onModeChange={(mode) => {
            navigate(mode === 'register' ? '/register' : '/login');
            setAuthView(mode);
          }}
          onAuthenticated={(next) => {
            setSession(next);
            navigate('/');
          }}
        />
      </MetaProvider>
    );
  }

  return (
    <MetaProvider>
      <AuthContext.Provider value={{ session, refresh, signOut }}>
        <PresenceProvider>
          <Shell />
        </PresenceProvider>
      </AuthContext.Provider>
    </MetaProvider>
  );
}

type AuthScreenProps = {
  mode: AuthView;
  onModeChange: (mode: AuthView) => void;
  onAuthenticated: (session: AuthSessionResponse) => void;
};

function AuthScreen({ mode, onModeChange, onAuthenticated }: AuthScreenProps) {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [registrationOpen, setRegistrationOpen] = useState(true);
  const [oidcEnabled, setOidcEnabled] = useState(false);
  const [passwordLoginEnabled, setPasswordLoginEnabled] = useState(true);

  useEffect(() => {
    let cancelled = false;
    const params = new URLSearchParams(window.location.search);
    const oidcMessage = params.get('oidc_message');
    if (oidcMessage) {
      setError(oidcMessage);
      params.delete('oidc_error');
      params.delete('oidc_message');
      const query = params.toString();
      const next = `${window.location.pathname}${query ? `?${query}` : ''}`;
      window.history.replaceState({}, '', next);
    }
    getMeta()
      .then((result) => {
        if (!cancelled && result.data) {
          setRegistrationOpen(result.data.registration_mode === 'open');
          setOidcEnabled(result.data.oidc_enabled);
          setPasswordLoginEnabled(result.data.password_login_enabled);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setRegistrationOpen(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const result =
        mode === 'register'
          ? await register({ body: { email, password }, ...credentials })
          : await login({ body: { email, password }, ...credentials });
      if (result.data) {
        onAuthenticated(result.data);
        return;
      }
      const message =
        result.error && typeof result.error === 'object' && 'message' in result.error
          ? String((result.error as { message: string }).message)
          : mode === 'register'
            ? 'Registration failed.'
            : 'Login failed.';
      setError(message);
    } catch {
      setError(mode === 'register' ? 'Registration failed.' : 'Login failed.');
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="relative flex min-h-full items-center justify-center overflow-hidden bg-app px-4 py-10 text-ink">
      <div className="pointer-events-none absolute inset-0 opacity-40">
        <div className="absolute -left-24 top-10 h-72 w-72 rounded-full bg-accent/20 blur-3xl" />
        <div className="absolute -right-16 bottom-0 h-80 w-80 rounded-full bg-accent-2/15 blur-3xl" />
        <div className="absolute inset-0 grid-veil opacity-50" />
      </div>

      <div className="relative w-full max-w-md rounded-2xl border border-line-2/70 bg-panel/95 p-6 shadow-panel backdrop-blur-sm">
        <div className="mb-6 flex items-center gap-3">
          <span className="grid h-9 w-9 place-items-center rounded-lg bg-accent/15 font-mono text-sm font-bold text-accent">
            VX
          </span>
          <div>
            <p className="kicker">VoxNexus</p>
            <h1 className="font-sans text-xl font-semibold text-ink">
              {mode === 'register' ? 'Create account' : 'Sign in'}
            </h1>
          </div>
        </div>

        {oidcEnabled ? (
          <a
            href="/api/v1/auth/oidc/start"
            className="mb-4 flex w-full items-center justify-center rounded-lg border border-line-2/60 bg-input px-3 py-2.5 text-[13.5px] font-semibold text-ink transition hover:border-accent/50"
          >
            Sign in with SSO
          </a>
        ) : null}

        {passwordLoginEnabled ? (
          <form className="space-y-3" onSubmit={onSubmit}>
            <label className="block">
              <span className="kicker">Email</span>
              <input
                type="email"
                name="email"
                autoComplete="email"
                required
                value={email}
                onChange={(event) => setEmail(event.target.value)}
                className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60"
              />
            </label>
            <label className="block">
              <span className="kicker">Password</span>
              <input
                type="password"
                name="password"
                autoComplete={mode === 'register' ? 'new-password' : 'current-password'}
                required
                minLength={8}
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                className="mt-1 w-full rounded-lg border border-line-2/50 bg-input px-3 py-2 text-[13.5px] text-ink outline-none focus:border-accent/60"
              />
            </label>
            {error ? <p className="text-[13px] text-dnd">{error}</p> : null}
            <button
              type="submit"
              disabled={busy}
              className="mt-2 w-full rounded-lg bg-accent px-3 py-2.5 text-[13.5px] font-semibold text-app transition hover:brightness-110 disabled:opacity-60"
            >
              {busy ? 'Working…' : mode === 'register' ? 'Register' : 'Sign in'}
            </button>
          </form>
        ) : error ? (
          <p className="text-[13px] text-dnd">{error}</p>
        ) : null}

        {passwordLoginEnabled ? (
          <p className="mt-4 text-center text-[13px] text-ink-3">
            {mode === 'register' ? 'Already have an account?' : 'Need an account?'}{' '}
            {registrationOpen ? (
              <button
                type="button"
                className="font-medium text-accent hover:underline"
                onClick={() => onModeChange(mode === 'register' ? 'login' : 'register')}
              >
                {mode === 'register' ? 'Sign in' : 'Register'}
              </button>
            ) : (
              <span className="text-ink-4">Registration is closed on this instance.</span>
            )}
          </p>
        ) : null}
      </div>
    </div>
  );
}
