import type { FormEvent } from 'react';
import { useEffect, useState } from 'react';
import { getMe, login, logout, register, type AuthSessionResponse } from '@voxnexus/api-client';
import { createGatewayClient } from '@voxnexus/protocol';
import { HelloPanel } from '@voxnexus/ui';

type AuthView = 'home' | 'login' | 'register';

function pathToView(pathname: string): AuthView {
  if (pathname === '/login') {
    return 'login';
  }
  if (pathname === '/register') {
    return 'register';
  }
  return 'home';
}

function navigate(path: string) {
  window.history.pushState({}, '', path);
  window.dispatchEvent(new PopStateEvent('popstate'));
}

const credentials = { credentials: 'include' as const };
const gatewayDebug = import.meta.env.VITE_GATEWAY_DEBUG === 'true';

export function App() {
  const [view, setView] = useState<AuthView>(() => pathToView(window.location.pathname));
  const [session, setSession] = useState<AuthSessionResponse | null>(null);
  const [sessionKnown, setSessionKnown] = useState(false);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [gatewayReady, setGatewayReady] = useState<string | null>(null);

  useEffect(() => {
    const onPop = () => setView(pathToView(window.location.pathname));
    window.addEventListener('popstate', onPop);
    return () => window.removeEventListener('popstate', onPop);
  }, []);

  useEffect(() => {
    let cancelled = false;
    getMe(credentials)
      .then((result) => {
        if (cancelled) {
          return;
        }
        if (result.data) {
          setSession(result.data);
        } else {
          setSession(null);
        }
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

  useEffect(() => {
    if (!gatewayDebug || !session) {
      setGatewayReady(null);
      return;
    }
    const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
    const client = createGatewayClient({
      url: `${protocol}://${window.location.host}/api/v1/gateway`,
      onReady: (ready) => {
        setGatewayReady(ready.account_id);
      },
      onClose: () => {
        setGatewayReady(null);
      },
    });
    client.connect();
    return () => {
      client.disconnect();
    };
  }, [session]);

  async function onSubmit(event: FormEvent, mode: 'login' | 'register') {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const result =
        mode === 'register'
          ? await register({ body: { email, password }, ...credentials })
          : await login({ body: { email, password }, ...credentials });
      if (result.data) {
        setSession(result.data);
        setPassword('');
        navigate('/');
        return;
      }
      const message =
        result.error && typeof result.error === 'object' && 'message' in result.error
          ? String((result.error as { message: string }).message)
          : 'Request failed.';
      setError(message);
    } catch {
      setError('Could not reach the API. Is the server running?');
    } finally {
      setBusy(false);
    }
  }

  async function onLogout() {
    setBusy(true);
    setError(null);
    try {
      await logout(credentials);
      setSession(null);
      navigate('/');
    } catch {
      setError('Logout failed.');
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="vn-shell">
      <aside className="vn-rail" aria-label="Account">
        <span
          className={session ? 'vn-rail-mark vn-rail-mark-live' : 'vn-rail-mark'}
          aria-hidden="true"
        />
        <span className="vn-rail-label">{session ? 'signed in' : 'signed out'}</span>
      </aside>
      <main className="vn-main">
        {view === 'home' ? (
          <HelloPanel title="VoxNexus" kicker="Self-hostable community OS">
            <p>
              Discord-class chat and voice, Guilded-class Spaces, and a first-class app platform —
              on a server you run.
            </p>
            <dl className="vn-meta">
              <div>
                <dt>Session</dt>
                <dd>
                  {!sessionKnown
                    ? 'checking…'
                    : session?.account.email
                      ? session.account.email
                      : 'guest'}
                </dd>
              </div>
              <div>
                <dt>Account</dt>
                <dd className="vn-auth-actions">
                  {session ? (
                    <button type="button" className="vn-linkish" disabled={busy} onClick={onLogout}>
                      Log out
                    </button>
                  ) : (
                    <>
                      <a
                        href="/login"
                        onClick={(e) => {
                          e.preventDefault();
                          navigate('/login');
                        }}
                      >
                        Log in
                      </a>
                      <a
                        href="/register"
                        onClick={(e) => {
                          e.preventDefault();
                          navigate('/register');
                        }}
                      >
                        Register
                      </a>
                    </>
                  )}
                </dd>
              </div>
            </dl>
            {gatewayDebug ? (
              <p className="vn-meta-note">
                Gateway:{' '}
                {gatewayReady
                  ? `READY account ${gatewayReady}`
                  : session
                    ? 'connecting…'
                    : 'sign in'}
              </p>
            ) : null}
            {error ? <p className="vn-meta-note">{error}</p> : null}
          </HelloPanel>
        ) : null}

        {view === 'login' || view === 'register' ? (
          <HelloPanel
            title={view === 'login' ? 'Log in' : 'Register'}
            kicker="Local email and password"
          >
            <form
              className="vn-auth-form"
              onSubmit={(event) => onSubmit(event, view === 'login' ? 'login' : 'register')}
            >
              <label className="vn-field">
                <span>Email</span>
                <input
                  type="email"
                  name="email"
                  autoComplete="username"
                  required
                  value={email}
                  onChange={(event) => setEmail(event.target.value)}
                />
              </label>
              <label className="vn-field">
                <span>Password</span>
                <input
                  type="password"
                  name="password"
                  autoComplete={view === 'login' ? 'current-password' : 'new-password'}
                  required
                  minLength={8}
                  value={password}
                  onChange={(event) => setPassword(event.target.value)}
                />
              </label>
              {error ? <p className="vn-meta-note">{error}</p> : null}
              <div className="vn-auth-actions">
                <button type="submit" disabled={busy}>
                  {view === 'login' ? 'Log in' : 'Create account'}
                </button>
                <a
                  href="/"
                  onClick={(event) => {
                    event.preventDefault();
                    navigate('/');
                  }}
                >
                  Back
                </a>
              </div>
            </form>
          </HelloPanel>
        ) : null}
      </main>
    </div>
  );
}
