export function generateSnowflake(): string {
  // Discord-like snowflake: timestamp + random
  return (
    Date.now().toString(36) + Math.random().toString(36).slice(2, 10)
  )
} 