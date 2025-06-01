import { requireAuthFromEvent } from '../utils/auth'

export default (nitroApp: any) => {
  nitroApp.hooks.hook('request', async (event: any) => {
    // Attach user to event.context if authenticated
    const user = await requireAuthFromEvent(event)
    if (user) {
      event.context.authUser = user
    }
  })
}
