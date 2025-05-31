import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import GroupDM from '../../models/GroupDM';

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
  return { status: 200, group };
}); 