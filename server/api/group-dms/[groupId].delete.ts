import { defineEventHandler } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import GroupDM from '../../models/GroupDM';
import GroupDMMessage from '../../models/GroupDMMessage';

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
  const isOwner = group.ownerId.toString() === userPayload.id;
  const isMember = group.memberIds.map(String).includes(userPayload.id);
  if (!isMember) {
    return { status: 403, error: 'You are not a member of this group DM.' };
  }
  if (isOwner) {
    // Delete group and all messages
    await GroupDMMessage.deleteMany({ groupId });
    await group.deleteOne();
    return { status: 200, message: 'Group DM deleted.' };
  } else {
    // Remove self from members
    group.memberIds = group.memberIds.filter(id => id.toString() !== userPayload.id);
    if (group.memberIds.length < 2) {
      // Delete group if last member leaves
      await GroupDMMessage.deleteMany({ groupId });
      await group.deleteOne();
      return { status: 200, message: 'Group DM deleted.' };
    } else {
      await group.save();
      return { status: 200, message: 'Left group DM.' };
    }
  }
}); 