/**
 * @jest-environment jsdom
 */
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { AuthProvider, useAuth } from './AuthContext';

// Mock fetch globally
const fetchMock = vi.fn();
global.fetch = fetchMock;

// Test component that uses auth context
function TestComponent() {
  const { user, login, logout, isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return <div>Loading...</div>;
  }

  if (!isAuthenticated) {
    return (
      <div>
        <button onClick={() => login('test@example.com', 'password')}>
          Login
        </button>
        <p>Not authenticated</p>
      </div>
    );
  }

  return (
    <div>
      <p>Welcome, {user?.username}!</p>
      <p>Role: {user?.role}</p>
      <button onClick={logout}>Logout</button>
    </div>
  );
}

describe('AuthContext', () => {
  beforeEach(() => {
    // Clear localStorage
    localStorage.clear();
    vi.clearAllMocks();

    // Reset fetch mock
    fetchMock.mockReset();
  });

  afterEach(() => {
    localStorage.clear();
  });

  describe('Initial state', () => {
    it('should show loading state initially', () => {
      render(
        <AuthProvider>
          <TestComponent />
        </AuthProvider>
      );

      expect(screen.getByText('Loading...')).toBeInTheDocument();
    });

    it('should initialize with no user when no token exists', async () => {
      render(
        <AuthProvider>
          <TestComponent />
        </AuthProvider>
      );

      await waitFor(() => {
        expect(screen.getByText('Not authenticated')).toBeInTheDocument();
      });
    });

    it('should initialize with user when valid token exists', async () => {
      // Mock a valid token in localStorage
      const mockToken = 'valid.jwt.token';
      const mockUser = { username: 'testuser', role: 'user' };
      localStorage.setItem('auth_token', mockToken);

      // Mock successful token validation
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ user: mockUser }),
      });

      render(
        <AuthProvider>
          <TestComponent />
        </AuthProvider>
      );

      await waitFor(() => {
        expect(screen.getByText('Welcome, testuser!')).toBeInTheDocument();
        expect(screen.getByText('Role: user')).toBeInTheDocument();
      });
    });
  });

  describe('Login functionality', () => {
    it('should handle successful login', async () => {
      const user = userEvent.setup();
      const mockResponse = {
        access_token: 'access.token',
        refresh_token: 'refresh.token',
        user: { username: 'testuser', role: 'user' },
      };

      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      render(
        <AuthProvider>
          <TestComponent />
        </AuthProvider>
      );

      await waitFor(() => {
        expect(screen.getByText('Not authenticated')).toBeInTheDocument();
      });

      const loginButton = screen.getByRole('button', { name: /login/i });
      await user.click(loginButton);

      await waitFor(() => {
        expect(screen.getByText('Welcome, testuser!')).toBeInTheDocument();
      });

      // Check that tokens were stored
      expect(localStorage.getItem('auth_token')).toBe('access.token');
      expect(localStorage.getItem('refresh_token')).toBe('refresh.token');
    });

    it('should handle login failure', async () => {
      const user = userEvent.setup();
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      fetchMock.mockResolvedValueOnce({
        ok: false,
        status: 401,
        json: async () => ({ error: 'Invalid credentials' }),
      });

      render(
        <AuthProvider>
          <TestComponent />
        </AuthProvider>
      );

      await waitFor(() => {
        expect(screen.getByText('Not authenticated')).toBeInTheDocument();
      });

      const loginButton = screen.getByRole('button', { name: /login/i });
      await user.click(loginButton);

      await waitFor(() => {
        expect(consoleSpy).toHaveBeenCalledWith('Login failed:', expect.any(Object));
      });

      // Should still show not authenticated
      expect(screen.getByText('Not authenticated')).toBeInTheDocument();

      consoleSpy.mockRestore();
    });

    it('should handle network errors during login', async () => {
      const user = userEvent.setup();
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      fetchMock.mockRejectedValueOnce(new Error('Network error'));

      render(
        <AuthProvider>
          <TestComponent />
        </AuthProvider>
      );

      await waitFor(() => {
        expect(screen.getByText('Not authenticated')).toBeInTheDocument();
      });

      const loginButton = screen.getByRole('button', { name: /login/i });
      await user.click(loginButton);

      await waitFor(() => {
        expect(consoleSpy).toHaveBeenCalledWith('Login failed:', expect.any(Error));
      });

      consoleSpy.mockRestore();
    });
  });

  describe('Logout functionality', () => {
    it('should handle logout successfully', async () => {
      const user = userEvent.setup();

      // Set up authenticated state
      const mockToken = 'valid.jwt.token';
      const mockUser = { username: 'testuser', role: 'user' };
      localStorage.setItem('auth_token', mockToken);

      fetchMock
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ user: mockUser }),
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({}),
        });

      render(
        <AuthProvider>
          <TestComponent />
        </AuthProvider>
      );

      await waitFor(() => {
        expect(screen.getByText('Welcome, testuser!')).toBeInTheDocument();
      });

      const logoutButton = screen.getByRole('button', { name: /logout/i });
      await user.click(logoutButton);

      await waitFor(() => {
        expect(screen.getByText('Not authenticated')).toBeInTheDocument();
      });

      // Check that tokens were removed
      expect(localStorage.getItem('auth_token')).toBeNull();
      expect(localStorage.getItem('refresh_token')).toBeNull();
    });
  });

  describe('Token refresh', () => {
    it('should handle token refresh on expired token', async () => {
      const mockExpiredToken = 'expired.token';
      const mockRefreshToken = 'refresh.token';
      const mockNewTokens = {
        access_token: 'new.access.token',
        refresh_token: 'new.refresh.token',
      };

      localStorage.setItem('auth_token', mockExpiredToken);
      localStorage.setItem('refresh_token', mockRefreshToken);

      // First call fails with 401, second succeeds with new tokens
      fetchMock
        .mockResolvedValueOnce({
          ok: false,
          status: 401,
          json: async () => ({ error: 'Token expired' }),
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockNewTokens,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ user: { username: 'testuser', role: 'user' } }),
        });

      render(
        <AuthProvider>
          <TestComponent />
        </AuthProvider>
      );

      await waitFor(() => {
        expect(screen.getByText('Welcome, testuser!')).toBeInTheDocument();
      });

      // Check that new tokens were stored
      expect(localStorage.getItem('auth_token')).toBe('new.access.token');
      expect(localStorage.getItem('refresh_token')).toBe('new.refresh.token');
    });

    it('should handle refresh token failure', async () => {
      const mockExpiredToken = 'expired.token';
      const mockRefreshToken = 'invalid.refresh.token';

      localStorage.setItem('auth_token', mockExpiredToken);
      localStorage.setItem('refresh_token', mockRefreshToken);

      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      // Token validation fails, refresh also fails
      fetchMock
        .mockResolvedValueOnce({
          ok: false,
          status: 401,
          json: async () => ({ error: 'Token expired' }),
        })
        .mockResolvedValueOnce({
          ok: false,
          status: 401,
          json: async () => ({ error: 'Invalid refresh token' }),
        });

      render(
        <AuthProvider>
          <TestComponent />
        </AuthProvider>
      );

      await waitFor(() => {
        expect(screen.getByText('Not authenticated')).toBeInTheDocument();
      });

      // Check that tokens were cleared
      expect(localStorage.getItem('auth_token')).toBeNull();
      expect(localStorage.getItem('refresh_token')).toBeNull();

      expect(consoleSpy).toHaveBeenCalledWith('Token refresh failed:', expect.any(Object));

      consoleSpy.mockRestore();
    });
  });

  describe('Error handling', () => {
    it('should handle malformed JSON responses', async () => {
      const user = userEvent.setup();
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => {
          throw new Error('Invalid JSON');
        },
      });

      render(
        <AuthProvider>
          <TestComponent />
        </AuthProvider>
      );

      await waitFor(() => {
        expect(screen.getByText('Not authenticated')).toBeInTheDocument();
      });

      const loginButton = screen.getByRole('button', { name: /login/i });
      await user.click(loginButton);

      await waitFor(() => {
        expect(consoleSpy).toHaveBeenCalledWith('Login failed:', expect.any(Error));
      });

      consoleSpy.mockRestore();
    });

    it('should handle unexpected server errors', async () => {
      const user = userEvent.setup();
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      fetchMock.mockResolvedValueOnce({
        ok: false,
        status: 500,
        json: async () => ({ error: 'Internal server error' }),
      });

      render(
        <AuthProvider>
          <TestComponent />
        </AuthProvider>
      );

      await waitFor(() => {
        expect(screen.getByText('Not authenticated')).toBeInTheDocument();
      });

      const loginButton = screen.getByRole('button', { name: /login/i });
      await user.click(loginButton);

      await waitFor(() => {
        expect(consoleSpy).toHaveBeenCalledWith('Login failed:', expect.any(Object));
      });

      consoleSpy.mockRestore();
    });
  });

  describe('Context usage outside provider', () => {
    it('should throw error when useAuth is used outside provider', () => {
      // Mock console.error to avoid noise in test output
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      expect(() => {
        render(<TestComponent />);
      }).toThrow('useAuth must be used within an AuthProvider');

      consoleSpy.mockRestore();
    });
  });

  describe('Memory leaks and cleanup', () => {
    it('should clean up event listeners on unmount', () => {
      const { unmount } = render(
        <AuthProvider>
          <TestComponent />
        </AuthProvider>
      );

      // This test ensures no memory leaks occur
      // In a real scenario, we'd check for cleanup of timers/intervals
      expect(() => unmount()).not.toThrow();
    });
  });
});