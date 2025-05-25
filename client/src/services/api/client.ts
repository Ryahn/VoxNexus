import axios, { AxiosInstance, AxiosRequestConfig } from 'axios'
import type { ApiResponse, ApiError } from './types'

class ApiClient {
  private client: AxiosInstance
  private baseURL: string
  private csrfToken: string | null = null
  private initializationPromise: Promise<void> | null = null

  constructor() {
    this.baseURL = process.env.VUE_APP_API_URL || 'http://localhost:3000'
    this.client = axios.create({
      baseURL: this.baseURL,
      headers: {
        'Content-Type': 'application/json',
      },
      withCredentials: true // Enable sending cookies
    })

    // Initialize CSRF token
    this.initializationPromise = this.initializeCsrfToken()

    // Add request interceptor for authentication and CSRF
    this.client.interceptors.request.use(async (config) => {
      // Wait for CSRF token initialization
      if (this.initializationPromise) {
        try {
          await this.initializationPromise
        } catch (error) {
          console.error('Failed to initialize CSRF token:', error)
          // Try to initialize again
          this.initializationPromise = this.initializeCsrfToken()
          await this.initializationPromise
        }
      }

      const token = localStorage.getItem('token')
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
        // If CSRF token is invalid, try to get a new one
        if (error.response?.status === 403 && error.response?.data?.error === 'Invalid CSRF token') {
          console.log('CSRF token invalid, refreshing...')
          this.initializationPromise = this.initializeCsrfToken()
          await this.initializationPromise
          // Retry the original request
          if (this.csrfToken) {
            error.config.headers['X-CSRF-Token'] = this.csrfToken
            return this.client(error.config)
          }
        }

        const apiError: ApiError = {
          status: error.response?.status || 500,
          message: error.response?.data?.message || 'An unexpected error occurred',
          errors: error.response?.data?.errors,
        }
        return Promise.reject(apiError)
      }
    )
  }

  private async initializeCsrfToken() {
    try {
      console.log('Initializing CSRF token...')
      const response = await this.client.get('/auth/csrf-token')
      this.csrfToken = response.data.csrfToken
      console.log('CSRF token initialized:', this.csrfToken)
    } catch (error) {
      console.error('Failed to initialize CSRF token:', error)
      throw error
    }
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