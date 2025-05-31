import { defineEventHandler, readBody } from 'h3';
import { connectToDatabase } from '../../../../utils/mongoose';
import { getUserFromEvent } from '../../../../utils/auth';
import GroupDMMessage, { IGroupDMMessage } from '../../../../models/GroupDMMessage';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const groupId = event.context.params?.groupId;
  const messageId = event.context.params?.messageId;
  if (!groupId || !messageId) {
    return { status: 400, error: 'Group ID and Message ID are required.' };
  }

  const body = await readBody(event);
  const { content, attachments } = body;
  if (!content || typeof content !== 'string' || !content.trim()) {
    return { status: 400, error: 'Message content required.' };
  }

  const message = await GroupDMMessage.findById(messageId) as IGroupDMMessage | null;
  if (!message) {
    return { status: 404, error: 'Message not found.' };
  }
  if (message.groupId.toString() !== groupId) {
    return { status: 400, error: 'Message does not belong to this group DM.' };
  }
  if (message.authorId.toString() !== userPayload.id) {
    return { status: 403, error: 'You do not have permission to edit this message.' };
  }

  message.content = content;
  if (attachments && Array.isArray(attachments)) {
    message.attachments = attachments;
  }
  message.updatedAt = new Date();
  await message.save();
  return { status: 200, message };
}); 