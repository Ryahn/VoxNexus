import { defineEventHandler } from 'h3'
import { connectToDatabase } from '../../../utils/mongoose'
import { getUserFromEvent } from '../../../utils/auth'
import Server from '../../../models/Server'

interface MutualServer {
  id: string
  name: string
  avatarUrl?: string
}

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

  // Find all servers where both users are members
  const servers = await Server.find({
    members: { $all: [userPayload.id, friendId] }
  }).select('_id name avatarUrl').lean()

  const mutualServers: MutualServer[] = servers.map((s: any) => ({
    id: s._id.toString(),
    name: s.name,
    avatarUrl: s.avatarUrl || '',
  }))

  return { mutualServers }
}) 