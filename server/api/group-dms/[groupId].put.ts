import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import GroupDM from '../../models/GroupDM';
import User from '../../models/User';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const groupId = event.context.params?.groupId;
  if (!groupId) {
    return { status: 400, error: 'Group ID is required.' };
  }
  const group = await GroupDM.findById(groupId);
  if (!group) {
    return { status: 404, error: 'Group DM not found.' };
  }
  if (group.ownerId.toString() !== userPayload.id) {
    return { status: 403, error: 'Only the group owner can update the group.' };
  }

  const body = await readBody(event);
  let { name, avatarUrl, memberIds } = body;
  if (memberIds) {
    if (!Array.isArray(memberIds)) {
      return { status: 400, error: 'memberIds must be an array.' };
    }
    // Ensure owner is included
    if (!memberIds.includes(userPayload.id)) {
      memberIds.push(userPayload.id);
    }
    memberIds = [...new Set(memberIds)];
    if (memberIds.length < 2 || memberIds.length > 20) {
      return { status: 400, error: 'Group DM must have between 2 and 20 members.' };
    }
    // Validate all users exist
    const users = await User.find({ _id: { $in: memberIds } }).lean();
    if (users.length !== memberIds.length) {
      return { status: 400, error: 'One or more users do not exist.' };
    }
    group.memberIds = memberIds;
  }
  if (name !== undefined) group.name = name;
  if (avatarUrl !== undefined) group.avatarUrl = avatarUrl;
  await group.save();
  return { status: 200, group };
}); 