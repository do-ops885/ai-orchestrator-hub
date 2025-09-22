export interface User {
  id: string
  username: string
  roles: string[]
  permissions: string[]
  client_type: string
}

export interface AuthState {
  user: User | null
  token: string | null
  refreshToken: string | null
  isAuthenticated: boolean
  isLoading: boolean
}

export interface AuthContextType extends AuthState {
  login: (username: string, password: string, clientType?: string) => Promise<void>
  logout: () => Promise<void>
  refreshTokenFn: () => Promise<void>
  hasPermission: (permission: string) => boolean
  hasRole: (role: string) => boolean
}