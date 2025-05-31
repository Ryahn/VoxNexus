import mongoose, { Schema, Document, Model, Types } from 'mongoose';

export type FriendRequestStatus = 'pending' | 'accepted' | 'rejected';

export interface IFriendRequest extends Document {
  from: Types.ObjectId;
  to: Types.ObjectId;
  status: FriendRequestStatus;
  createdAt: Date;
}

const FriendRequestSchema: Schema<IFriendRequest> = new Schema({
  from: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  to: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  status: { type: String, enum: ['pending', 'accepted', 'rejected'], default: 'pending' },
  createdAt: { type: Date, default: Date.now },
});

export default (mongoose.models.FriendRequest as Model<IFriendRequest>) || mongoose.model<IFriendRequest>('FriendRequest', FriendRequestSchema); 