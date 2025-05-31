import { ref } from 'vue'

interface ReactionPayload {
  emoji: string
  action: 'add' | 'remove'
}

interface EditPayload {
  content: string
  attachments?: string[]
}

export function useMessageActions() {
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Add or remove a reaction
  async function reactToMessage({
    type,
    ids,
    emoji,
    action
  }: {
    type: 'channel' | 'group-dm' | 'dm'
    ids: { channelId?: string; groupId?: string; userId?: string; messageId: string }
    emoji: string
    action: 'add' | 'remove'
  }) {
    isLoading.value = true
    error.value = null
    try {
      let url = ''
      if (type === 'channel') {
        url = `/api/channels/${ids.channelId}/messages/${ids.messageId}/reactions`
      } else if (type === 'group-dm') {
        url = `/api/group-dms/${ids.groupId}/messages/${ids.messageId}/reactions`
      } else if (type === 'dm') {
        url = `/api/dms/${ids.userId}/messages/${ids.messageId}/reactions`
      }
      const res = await $fetch(url, {
        method: 'POST',
        body: { emoji, action }
      })
      return res
    } catch (e: any) {
      error.value = e?.data?.error || e.message || 'Failed to react to message.'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  // Edit a message
  async function editMessage({
    type,
    ids,
    content,
    attachments
  }: {
    type: 'channel' | 'group-dm' | 'dm'
    ids: { channelId?: string; groupId?: string; userId?: string; messageId: string }
    content: string
    attachments?: string[]
  }) {
    isLoading.value = true
    error.value = null
    try {
      let url = ''
      if (type === 'channel') {
        url = `/api/channels/${ids.channelId}/messages/${ids.messageId}`
      } else if (type === 'group-dm') {
        url = `/api/group-dms/${ids.groupId}/messages/${ids.messageId}`
      } else if (type === 'dm') {
        url = `/api/dms/${ids.userId}/messages/${ids.messageId}`
      }
      const res = await $fetch(url, {
        method: 'PUT',
        body: { content, attachments }
      })
      return res
    } catch (e: any) {
      error.value = e?.data?.error || e.message || 'Failed to edit message.'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  // Delete a message
  async function deleteMessage({
    type,
    ids
  }: {
    type: 'channel' | 'group-dm' | 'dm'
    ids: { channelId?: string; groupId?: string; userId?: string; messageId: string }
  }) {
    isLoading.value = true
    error.value = null
    try {
      let url = ''
      if (type === 'channel') {
        url = `/api/channels/${ids.channelId}/[messageId].delete` // TODO: update to correct endpoint
      } else if (type === 'group-dm') {
        url = `/api/group-dms/${ids.groupId}/messages/${ids.messageId}` // TODO: implement delete endpoint
      } else if (type === 'dm') {
        url = `/api/dms/${ids.userId}/messages/${ids.messageId}` // TODO: implement delete endpoint
      }
      const res = await $fetch(url, {
        method: 'DELETE'
      })
      return res
    } catch (e: any) {
      error.value = e?.data?.error || e.message || 'Failed to delete message.'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  // Upload an attachment (returns URL)
  async function uploadAttachment({
    type,
    ids,
    file
  }: {
    type: 'channel' | 'group-dm' | 'dm'
    ids: { channelId?: string; groupId?: string; userId?: string }
    file: File
  }) {
    isLoading.value = true
    error.value = null
    try {
      const formData = new FormData()
      formData.append('attachment', file)
      let url = ''
      if (type === 'channel') {
        url = `/api/channels/${ids.channelId}/attachment`
      } else if (type === 'dm') {
        url = `/api/dms/${ids.userId}/attachment`
      } else if (type === 'group-dm') {
        // Not implemented in backend yet
        throw new Error('Group DM attachment upload not implemented')
      }
      const res = await $fetch(url, {
        method: 'POST',
        body: formData
      })
      return res
    } catch (e: any) {
      error.value = e?.data?.error || e.message || 'Failed to upload attachment.'
      throw e
    } finally {
      isLoading.value = false
    }
  }

  return {
    isLoading,
    error,
    reactToMessage,
    editMessage,
    deleteMessage,
    uploadAttachment
  }
} 