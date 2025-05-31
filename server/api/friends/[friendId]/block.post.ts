import { defineEventHandler } from 'h3'
import { connectToDatabase } from '../../../utils/mongoose'
import { getUserFromEvent } from '../../../utils/auth'
import FriendRequest from '../../../models/FriendRequest'
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

  // Check if already blocked
  const alreadyBlocked = await Block.findOne({
    blockerId: userPayload.id,
    blockedId: friendId,
  })
  if (alreadyBlocked) {
    return { status: 200, message: 'User already blocked.' }
  }

  // Create block
  await Block.create({
    blockerId: userPayload.id,
    blockedId: friendId,
  })

  // Remove any friend relationship (accepted or pending)
  await FriendRequest.deleteMany({
    $or: [
      { from: userPayload.id, to: friendId },
      { from: friendId, to: userPayload.id },
    ],
  })

  // Emit real-time event to the blocked user
  emitToUser(friendId, 'user:blocked', { blockerId: userPayload.id })

  return { status: 200, message: 'User blocked.' }
}) 