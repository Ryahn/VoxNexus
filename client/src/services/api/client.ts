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
    // Use the current domain for API requests if accessed via voxnexus.test
    const currentHost = window.location.hostname
    if (currentHost === 'voxnexus.test') {
      // Use the same domain, let Caddy handle routing to the server
      this.baseURL = `${window.location.protocol}//${window.location.host}/api`
    } else {
      this.baseURL = process.env.VUE_APP_API_URL || 'http://localhost:3555/api'
    }
    
    console.log('API Client initialized with baseURL:', this.baseURL)
    
    this.client = axios.create({
      baseURL: this.baseURL,
      timeout: 30000,
      withCredentials: true,
      headers: {
        'Content-Type': 'application/json',
      },
    })

    // CSRF initialization completely removed for now
    // this.initializationPromise = this.initializeCsrfToken()

    // Add request interceptor
    this.client.interceptors.request.use(async (config) => {
      await this.checkRateLimit()

      // Skip CSRF token requirement for the csrf-token endpoint itself
      if (config.url?.includes('/auth/csrf-token')) {
        return config
      }

      // TEMPORARILY BYPASS CSRF TOKEN INITIALIZATION
      /*
      // Wait for CSRF token initialization
      if (this.initializationPromise) {
        try {
          await this.initializationPromise
        } catch (error) {
          console.error('Failed to initialize CSRF token:', error)
          // Retry initialization once
          this.initializationPromise = this.initializeCsrfToken()
          try {
            await this.initializationPromise
          } catch (retryError) {
            console.error('Failed to initialize CSRF token on retry:', retryError)
            // Don't throw error, let the request proceed without CSRF token
            // The server will handle the missing token appropriately
          }
        }
      }
      */

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

      // TEMPORARILY BYPASS CSRF TOKEN ADDITION
      /*
      // Add CSRF token if available
      if (this.csrfToken) {
        console.log('Adding CSRF token to request:', this.csrfToken)
        config.headers['X-CSRF-Token'] = this.csrfToken
      } else {
        console.warn('No CSRF token available for request')
        // Don't throw error, let the server handle the missing token
      }
      */

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
        withCredentials: true,
        timeout: 10000 // 10 second timeout
      })
      
      if (response.data && response.data.csrfToken) {
        this.csrfToken = response.data.csrfToken
        console.log('CSRF token initialized successfully:', this.csrfToken)
      } else {
        console.error('Invalid CSRF token response:', response.data)
        throw new Error('Invalid CSRF token response')
      }
    } catch (error) {
      console.error('Failed to initialize CSRF token:', error)
      // Reset the token to null so we know it failed
      this.csrfToken = null
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
    console.log('[AUTH_ROUTE] Base URL:', this.baseURL)
    console.log('[AUTH_ROUTE] POST request to', url)
    console.log('[AUTH_ROUTE] Data:', data)
    console.log('[AUTH_ROUTE] Config:', config)
    const response = await this.client.post<T>(url, data, config)
    console.log('[AUTH_ROUTE] Response:', response)
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