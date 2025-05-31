import { defineEventHandler } from 'h3'
import { connectToDatabase } from '../../../utils/mongoose'
import { getUserFromEvent } from '../../../utils/auth'
import Block from '../../../models/Block'
import { emitToUser } from '../../../socket/index'

export default defineEventHandler(async (event) => {
  await connectToDatabase()
  let userPayload
  try {
    userPayload = getUserFromEvent(event)
  } catch (err) {
    return { status: 401, error: 'Unauthorized' }
  }

  const friendId = event.context.params?.friendId as string
  if (!friendId) {
    return { status: 400, error: 'Friend ID required.' }
  }

  // Remove the block
  const result = await Block.findOneAndDelete({
    blockerId: userPayload.id,
    blockedId: friendId,
  })
  if (!result) {
    return { status: 404, error: 'Block not found.' }
  }

  // Emit real-time event to the unblocked user
  emitToUser(friendId, 'user:unblocked', { by: userPayload.id })

  return { status: 200, message: 'User unblocked.' }
}) 