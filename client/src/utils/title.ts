/**
 * Utility functions for managing the page title
 */

/**
 * Updates the page title with the server name
 * @param serverName - The name of the current server
 */
export const updateTitle = (serverName?: string): void => {
  const baseTitle = 'VoxNexus'
  document.title = serverName ? `${baseTitle} | Server: ${serverName}` : baseTitle
}

/**
 * Resets the page title to the default
 */
export const resetTitle = (): void => {
  document.title = 'VoxNexus'
} 