import { defineEventHandler, getQuery } from 'h3';
import { connectToDatabase } from '../../../utils/mongoose';
import { getUserFromEvent } from '../../../utils/auth';
import GroupDM from '../../../models/GroupDM';
import GroupDMMessage from '../../../models/GroupDMMessage';

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
  const group = await GroupDM.findById(groupId).lean();
  if (!group) {
    return { status: 404, error: 'Group DM not found.' };
  }
  if (!group.memberIds.map(String).includes(userPayload.id)) {
    return { status: 403, error: 'You are not a member of this group DM.' };
  }

  const { limit, after } = getQuery(event);
  const pageLimit = Math.min(Number(limit) || 50, 100);
  const query: any = { groupId };
  if (after) {
    const afterMsg = await GroupDMMessage.findById(after).lean();
    if (afterMsg) {
      query.createdAt = { $gt: afterMsg.createdAt };
    }
  }
  const messages = await GroupDMMessage.find(query)
    .sort({ createdAt: 1 })
    .limit(pageLimit + 1)
    .lean();
  let nextCursor = null;
  if (messages.length > pageLimit) {
    nextCursor = messages[pageLimit]._id.toString();
    messages.length = pageLimit;
  }
  return { status: 200, messages, nextCursor };
}); 