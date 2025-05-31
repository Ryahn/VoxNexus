import { defineEventHandler } from 'h3'
import { connectToDatabase } from '../../../utils/mongoose'
import { getUserFromEvent } from '../../../utils/auth'
import FriendRequest from '../../../models/FriendRequest'
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

  // Remove the accepted friend request (either direction)
  const result = await FriendRequest.findOneAndDelete({
    $or: [
      { from: userPayload.id, to: friendId },
      { from: friendId, to: userPayload.id },
    ],
    status: 'accepted',
  })

  if (!result) {
    return { status: 404, error: 'Friend relationship not found.' }
  }

  // Emit real-time event to both users
  emitToUser(userPayload.id, 'friend:removed', { friendId })
  emitToUser(friendId, 'friend:removed', { friendId: userPayload.id })

  return { status: 200, message: 'Friend removed.' }
}) 