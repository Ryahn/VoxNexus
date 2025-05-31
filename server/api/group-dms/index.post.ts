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

  const body = await readBody(event);
  let { name, memberIds, avatarUrl } = body;
  if (!Array.isArray(memberIds)) {
    return { status: 400, error: 'memberIds must be an array.' };
  }
  // Ensure self is included
  if (!memberIds.includes(userPayload.id)) {
    memberIds.push(userPayload.id);
  }
  // Remove duplicates
  memberIds = [...new Set(memberIds)];
  if (memberIds.length < 2 || memberIds.length > 20) {
    return { status: 400, error: 'Group DM must have between 2 and 20 members.' };
  }
  // Validate all users exist
  const users = await User.find({ _id: { $in: memberIds } }).lean();
  if (users.length !== memberIds.length) {
    return { status: 400, error: 'One or more users do not exist.' };
  }
  // Prevent duplicate group with same members
  const existing = await GroupDM.findOne({
    memberIds: { $all: memberIds, $size: memberIds.length },
  }).lean();
  if (existing) {
    return { status: 409, error: 'A group DM with these members already exists.', group: existing };
  }
  const group = await GroupDM.create({
    name,
    ownerId: userPayload.id,
    memberIds,
    avatarUrl,
    createdAt: new Date(),
  });
  return { status: 201, group };
}); 