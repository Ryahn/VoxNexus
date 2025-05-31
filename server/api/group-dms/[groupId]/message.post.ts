import { defineEventHandler, readBody } from 'h3';
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
  const group = await GroupDM.findById(groupId);
  if (!group) {
    return { status: 404, error: 'Group DM not found.' };
  }
  if (!group.memberIds.map(String).includes(userPayload.id)) {
    return { status: 403, error: 'You are not a member of this group DM.' };
  }

  const body = await readBody(event);
  const { content, attachments } = body;
  if (!content || typeof content !== 'string' || !content.trim()) {
    return { status: 400, error: 'Message content required.' };
  }
  const message = await GroupDMMessage.create({
    groupId,
    authorId: userPayload.id,
    content,
    attachments: Array.isArray(attachments) ? attachments : [],
    createdAt: new Date(),
    updatedAt: new Date(),
  });
  group.lastMessageAt = new Date();
  await group.save();
  return { status: 201, message };
}); 