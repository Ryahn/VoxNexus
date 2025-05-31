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

  const groups = await GroupDM.find({ memberIds: userPayload.id })
    .sort({ lastMessageAt: -1, createdAt: -1 })
    .lean();
  return { status: 200, groups };
}); 