import type { ApiError } from '@/types'

export class ApiErrorHandler {
  static handle(error: unknown): string {
    if (this.isApiError(error)) {
      return error.message
    }
    
    if (error instanceof Error) {
      return error.message
    }
    
    return 'An unexpected error occurred'
  }

  static isApiError(error: unknown): error is ApiError {
    return (
      typeof error === 'object' &&
      error !== null &&
      'status' in error &&
      'message' in error
    )
  }

  static getValidationErrors(error: unknown): Record<string, string[]> {
    if (this.isApiError(error) && error.errors) {
      return error.errors
    }
    return {}
  }

  static isNetworkError(error: unknown): boolean {
    return error instanceof Error && error.message === 'Network Error'
  }

  static isUnauthorized(error: unknown): boolean {
    return this.isApiError(error) && error.status === 401
  }

  static isForbidden(error: unknown): boolean {
    return this.isApiError(error) && error.status === 403
  }

  static isNotFound(error: unknown): boolean {
    return this.isApiError(error) && error.status === 404
  }

  static isServerError(error: unknown): boolean {
    return this.isApiError(error) && error.status >= 500
  }
} 