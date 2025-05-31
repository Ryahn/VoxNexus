import { defineEventHandler, readMultipartFormData } from 'h3';
import { connectToDatabase } from '../../utils/mongoose';
import { getUserFromEvent } from '../../utils/auth';
import User from '../../models/User';
import { indexUser } from '../../utils/zincsearch';

export default defineEventHandler(async (event) => {
  await connectToDatabase();
  let userPayload;
  try {
    userPayload = getUserFromEvent(event);
  } catch (err) {
    return { status: 401, error: 'Unauthorized' };
  }

  const form = await readMultipartFormData(event);
  if (!form || !form.length) {
    return { status: 400, error: 'No file uploaded.' };
  }
  const file = form.find(f => f.name === 'avatar');
  if (!file || !file.data) {
    return { status: 400, error: 'No avatar file provided.' };
  }

  // Use AttachmentService
  // @ts-ignore
  const attachmentService = event.context.attachmentService as import('../../utils/attachment-service/AttachmentService').default;
  const avatarPath = await attachmentService.uploadUserAvatar(file.data, userPayload.id, file.type || 'image/png');
  const publicUrl = `/${avatarPath}`;

  const user = await User.findById(userPayload.id);
  if (!user) {
    return { status: 404, error: 'User not found.' };
  }
  user.avatarUrl = publicUrl;
  await user.save();
  // Index user in ZincSearch
  await indexUser({ id: user._id.toString(), username: user.username, email: user.email, avatarUrl: user.avatarUrl, bio: user.bio });

  return { status: 200, avatarUrl: publicUrl };
}); 