import axios, { AxiosInstance, AxiosRequestConfig } from 'axios'
import type { ApiResponse, ApiError } from './types'
import { useAuthStore } from '@/stores/auth'

class ApiClient {
  private client: AxiosInstance
  private baseURL: string
  private csrfToken: string | null = null
  private initializationPromise: Promise<void> | null = null
  private refreshPromise: Promise<void> | null = null
  private requestCount = 0
  private lastRequestTime = 0
  private readonly RATE_LIMIT = 100 // requests per minute
  private readonly RATE_LIMIT_WINDOW = 60000 // 1 minute in milliseconds

  constructor() {
    this.baseURL = process.env.VUE_APP_API_URL || 'http://localhost:3000'
    this.client = axios.create({
      baseURL: this.baseURL,
      headers: {
        'Content-Type': 'application/json',
        'X-Content-Type-Options': 'nosniff',
        'X-Frame-Options': 'DENY',
        'X-XSS-Protection': '1; mode=block',
        'Strict-Transport-Security': 'max-age=31536000; includeSubDomains',
        'Referrer-Policy': 'strict-origin-when-cross-origin',
        'Permissions-Policy': 'geolocation=(), microphone=(), camera=()',
      },
      withCredentials: true // Enable sending cookies
    })

    // Initialize CSRF token
    this.initializationPromise = this.initializeCsrfToken()

    // Add request interceptor for authentication, CSRF, and rate limiting
    this.client.interceptors.request.use(async (config) => {
      // Rate limiting check
      await this.checkRateLimit()

      // Skip CSRF token requirement for the csrf-token endpoint itself
      if (config.url?.includes('/auth/csrf-token')) {
        return config
      }

      // Wait for CSRF token initialization
      if (this.initializationPromise) {
        try {
          await this.initializationPromise
        } catch (error) {
          console.error('Failed to initialize CSRF token:', error)
          this.initializationPromise = this.initializeCsrfToken()
          await this.initializationPromise
        }
      }

      const authStore = useAuthStore()
      
      // Check if token is expired or about to expire (within 30 seconds)
      if (authStore.isTokenExpired || 
          (authStore.tokenExpiry && new Date(authStore.tokenExpiry).getTime() - Date.now() < 30000)) {
        await this.refreshTokens()
      }

      const token = authStore.token
      if (token) {
        config.headers.Authorization = `Bearer ${token}`
      }

      // Add CSRF token if available
      if (this.csrfToken) {
        console.log('Adding CSRF token to request:', this.csrfToken)
        config.headers['X-CSRF-Token'] = this.csrfToken
      } else {
        throw new Error('No CSRF token available')
      }

      console.log('Request headers:', config.headers)
      return config
    })

    // Add response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      async (error) => {
        // Handle CSRF token invalidation
        if (error.response?.status === 403 && error.response?.data?.error === 'Invalid CSRF token') {
          console.log('CSRF token invalid, refreshing...')
          this.initializationPromise = this.initializeCsrfToken()
          await this.initializationPromise
          if (this.csrfToken) {
            error.config.headers['X-CSRF-Token'] = this.csrfToken
            return this.client(error.config)
          }
        }

        // Handle token expiration
        if (error.response?.status === 401) {
          try {
            await this.refreshTokens()
            // Retry the original request with new token
            const authStore = useAuthStore()
            error.config.headers.Authorization = `Bearer ${authStore.token}`
            return this.client(error.config)
          } catch (refreshError) {
            // If refresh fails, clear auth and redirect to login
            const authStore = useAuthStore()
            authStore.clearAuth()
            window.location.href = '/login'
            return Promise.reject(refreshError)
          }
        }

        const apiError: ApiError = {
          status: error.response?.status || 500,
          message: error.response?.data?.error || error.response?.data?.message || 'An unexpected error occurred',
          errors: error.response?.data?.errors,
        }
        return Promise.reject(apiError)
      }
    )
  }

  private async initializeCsrfToken() {
    try {
      console.log('Initializing CSRF token...')
      // Use axios directly to avoid the request interceptor that requires CSRF token
      const response = await axios.get(`${this.baseURL}/auth/csrf-token`, {
        withCredentials: true
      })
      this.csrfToken = response.data.csrfToken
      console.log('CSRF token initialized:', this.csrfToken)
    } catch (error) {
      console.error('Failed to initialize CSRF token:', error)
      throw error
    }
  }

  private async refreshTokens(): Promise<void> {
    // If a refresh is already in progress, wait for it
    if (this.refreshPromise) {
      await this.refreshPromise
      return
    }

    // Start new refresh
    this.refreshPromise = (async () => {
      try {
        const authStore = useAuthStore()
        await authStore.refreshTokens()
      } finally {
        this.refreshPromise = null
      }
    })()

    await this.refreshPromise
  }

  private async checkRateLimit(): Promise<void> {
    const now = Date.now()
    
    // Reset counter if window has passed
    if (now - this.lastRequestTime > this.RATE_LIMIT_WINDOW) {
      this.requestCount = 0
      this.lastRequestTime = now
    }

    // Check if we've exceeded the rate limit
    if (this.requestCount >= this.RATE_LIMIT) {
      const waitTime = this.RATE_LIMIT_WINDOW - (now - this.lastRequestTime)
      await new Promise(resolve => setTimeout(resolve, waitTime))
      this.requestCount = 0
      this.lastRequestTime = Date.now()
    }

    this.requestCount++
  }

  async get<T>(url: string, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.client.get<T>(url, config)
    return {
      data: response.data,
      status: response.status,
    }
  }

  async post<T>(url: string, data?: any, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.client.post<T>(url, data, config)
    return {
      data: response.data,
      status: response.status,
    }
  }

  async put<T>(url: string, data?: any, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.client.put<T>(url, data, config)
    return {
      data: response.data,
      status: response.status,
    }
  }

  async delete<T>(url: string, config?: AxiosRequestConfig): Promise<ApiResponse<T>> {
    const response = await this.client.delete<T>(url, config)
    return {
      data: response.data,
      status: response.status,
    }
  }
}

export const apiClient = new ApiClient() 