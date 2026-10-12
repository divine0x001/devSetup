/**
 * Prism — Spotify Authorization Code + PKCE (100% client-side).
 *
 * No backend, no client secret. The PKCE verifier lives in sessionStorage until
 * the redirect comes back; tokens persist in localStorage under
 * `prism.spotify.tokens`.
 *
 * Refresh strategy: `getValidAccessToken()` refreshes transparently when the
 * access token is within EXPIRY_SKEW_MS of expiring. Concurrent refreshes are
 * coalesced into a single in-flight request (see `inFlightRefresh`) so a burst
 * of API calls never triggers parallel token requests.
 */

import { assertSpotifyConfigured, spotifyConfig } from './config';
import { createDemoTokens, disableDemoMode, enableDemoMode, isDemoMode } from './demo';

import type { SpotifyTokenResponse, SpotifyTokens } from './types';

const VERIFIER_KEY = 'prism.spotify.pkce.verifier';
const STATE_KEY = 'prism.spotify.pkce.state';
const RETURN_PATH_KEY = 'prism.spotify.pkce.returnPath';
const TOKENS_KEY = 'prism.spotify.tokens';

/** Safety margin (ms): refresh 60s before the token actually expires. */
const EXPIRY_SKEW_MS = 60_000;

/* -------------------------------------------------------------------------- */
/* PKCE helpers                                                                */
/* -------------------------------------------------------------------------- */

function base64UrlEncode(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.length; i += 1) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

/** Random, high-entropy code verifier (RFC 7636). */
export function generateCodeVerifier(byteLength = 64): string {
  const bytes = new Uint8Array(byteLength);
  crypto.getRandomValues(bytes);
  return base64UrlEncode(bytes.buffer);
}

/** SHA-256 → base64url transform of the verifier (S256 method). */
export async function generateCodeChallenge(verifier: string): Promise<string> {
  const data = new TextEncoder().encode(verifier);
  const digest = await crypto.subtle.digest('SHA-256', data);
  return base64UrlEncode(digest);
}

/* -------------------------------------------------------------------------- */
/* Login flow                                                                  */
/* -------------------------------------------------------------------------- */

/**
 * Starts the PKCE login: builds the authorize URL and redirects the browser.
 * @param returnPath path to restore after the callback (defaults to "/").
 */
export async function beginLogin(returnPath = '/'): Promise<void> {
  // Demo mode: skip the redirect entirely and persist a fake session.
  if (isDemoMode()) {
    enableDemoMode();
    saveTokens(createDemoTokens());
    return;
  }

  assertSpotifyConfigured();

  const verifier = generateCodeVerifier();
  const challenge = await generateCodeChallenge(verifier);
  const state = generateCodeVerifier(24);

  sessionStorage.setItem(VERIFIER_KEY, verifier);
  sessionStorage.setItem(STATE_KEY, state);
  sessionStorage.setItem(RETURN_PATH_KEY, returnPath);

  const params = new URLSearchParams({
    client_id: spotifyConfig.clientId,
    response_type: 'code',
    redirect_uri: spotifyConfig.redirectUri,
    code_challenge_method: 'S256',
    code_challenge: challenge,
    state,
    scope: spotifyConfig.scopes.join(' '),
  });

  window.location.assign(`${spotifyConfig.authorizeEndpoint}?${params.toString()}`);
}

/** Exchanges an authorization code for tokens and persists them. */
export async function exchangeCodeForTokens(code: string): Promise<SpotifyTokens> {
  assertSpotifyConfigured();

  const verifier = sessionStorage.getItem(VERIFIER_KEY);
  if (!verifier) {
    throw new Error('Prism: missing PKCE verifier (was beginLogin called?).');
  }

  const body = new URLSearchParams({
    client_id: spotifyConfig.clientId,
    grant_type: 'authorization_code',
    code,
    redirect_uri: spotifyConfig.redirectUri,
    code_verifier: verifier,
  });

  const response = await fetch(spotifyConfig.tokenEndpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body,
  });

  if (!response.ok) {
    throw new Error(`Prism: token exchange failed (${response.status}).`);
  }

  const tokenResponse = (await response.json()) as SpotifyTokenResponse;
  const tokens = toSpotifyTokens(tokenResponse);
  saveTokens(tokens);
  sessionStorage.removeItem(VERIFIER_KEY);
  sessionStorage.removeItem(STATE_KEY);

  return tokens;
}

/**
 * Reads `?code`/`?state` from the current URL, validates state and exchanges
 * the code. Returns `null` when there is nothing to handle.
 */
export async function handleAuthCallback(
  search: string = window.location.search,
): Promise<{ tokens: SpotifyTokens; returnPath: string } | null> {
  const params = new URLSearchParams(search);
  const error = params.get('error');
  if (error) {
    clearAuthSession();
    throw new Error(`Prism: Spotify authorization failed (${error}).`);
  }

  const code = params.get('code');
  if (!code) return null;

  // Anti-CSRF: when a login flow was started, the returned state MUST match.
  const expectedState = sessionStorage.getItem(STATE_KEY);
  const receivedState = params.get('state');
  if (expectedState && receivedState !== expectedState) {
    clearAuthSession();
    throw new Error('Prism: OAuth state mismatch — possible CSRF, aborting.');
  }

  const tokens = await exchangeCodeForTokens(code);
  const returnPath = sessionStorage.getItem(RETURN_PATH_KEY) ?? '/';
  sessionStorage.removeItem(RETURN_PATH_KEY);

  return { tokens, returnPath };
}

/* -------------------------------------------------------------------------- */
/* Token storage & refresh                                                     */
/* -------------------------------------------------------------------------- */

function toSpotifyTokens(response: SpotifyTokenResponse): SpotifyTokens {
  return {
    accessToken: response.access_token,
    refreshToken: response.refresh_token ?? null,
    tokenType: 'Bearer',
    scope: response.scope,
    expiresAt: Date.now() + response.expires_in * 1000,
  };
}

export function getStoredTokens(): SpotifyTokens | null {
  if (isDemoMode()) return createDemoTokens();

  const raw = localStorage.getItem(TOKENS_KEY);
  if (!raw) return null;
  try {
    return JSON.parse(raw) as SpotifyTokens;
  } catch {
    localStorage.removeItem(TOKENS_KEY);
    return null;
  }
}

export function saveTokens(tokens: SpotifyTokens): void {
  localStorage.setItem(TOKENS_KEY, JSON.stringify(tokens));
}

export function clearTokens(): void {
  localStorage.removeItem(TOKENS_KEY);
}

function isExpired(tokens: SpotifyTokens): boolean {
  return Date.now() >= tokens.expiresAt - EXPIRY_SKEW_MS;
}

/** Single in-flight refresh, shared by every caller (avoids refresh storms). */
let inFlightRefresh: Promise<SpotifyTokens> | null = null;

async function performRefresh(tokens: SpotifyTokens): Promise<SpotifyTokens> {
  const body = new URLSearchParams({
    client_id: spotifyConfig.clientId,
    grant_type: 'refresh_token',
    refresh_token: tokens.refreshToken as string,
  });

  const response = await fetch(spotifyConfig.tokenEndpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body,
  });

  if (!response.ok) {
    clearTokens();
    throw new Error(`Prism: token refresh failed (${response.status}).`);
  }

  const tokenResponse = (await response.json()) as SpotifyTokenResponse;
  const refreshed: SpotifyTokens = {
    accessToken: tokenResponse.access_token,
    // Spotify may omit a new refresh token; keep the previous one.
    refreshToken: tokenResponse.refresh_token ?? tokens.refreshToken,
    tokenType: 'Bearer',
    scope: tokenResponse.scope,
    expiresAt: Date.now() + tokenResponse.expires_in * 1000,
  };
  saveTokens(refreshed);
  return refreshed;
}

/** Refreshes the access token using the stored refresh token. */
export function refreshAccessToken(tokens: SpotifyTokens): Promise<SpotifyTokens> {
  if (!tokens.refreshToken) {
    return Promise.reject(
      new Error('Prism: no refresh token available, please sign in again.'),
    );
  }
  if (!inFlightRefresh) {
    inFlightRefresh = performRefresh(tokens).finally(() => {
      inFlightRefresh = null;
    });
  }
  return inFlightRefresh;
}

/**
 * Returns a valid access token, refreshing it transparently if needed.
 * Returns `null` when there is no session or the refresh failed.
 */
export async function getValidAccessToken(): Promise<string | null> {
  const tokens = getStoredTokens();
  if (!tokens) return null;
  if (!isExpired(tokens)) return tokens.accessToken;
  try {
    const refreshed = await refreshAccessToken(tokens);
    return refreshed.accessToken;
  } catch {
    // Refresh failed (network, revoked refresh token…): treat as signed out.
    return null;
  }
}

/** Full sign-out: clears tokens and the PKCE session (caller resets stores). */
export function signOut(): void {
  if (isDemoMode()) disableDemoMode();
  clearTokens();
  clearAuthSession();
}

function clearAuthSession(): void {
  sessionStorage.removeItem(VERIFIER_KEY);
  sessionStorage.removeItem(STATE_KEY);
  sessionStorage.removeItem(RETURN_PATH_KEY);
}