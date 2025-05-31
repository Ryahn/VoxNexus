import { defineNitroPlugin } from 'nitropack/runtime'
import AttachmentService from '../utils/attachment-service/AttachmentService'

export default defineNitroPlugin((nitroApp: any) => {
  nitroApp.hooks.hook('request', (event: any) => {
    // @ts-ignore
    event.context.attachmentService = AttachmentService
  })
}) 