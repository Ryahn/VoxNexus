import mongoose, { Schema, Document, Model, Types } from 'mongoose';

export interface IBan extends Document {
  serverId: Types.ObjectId;
  userId: Types.ObjectId;
  bannedBy: Types.ObjectId;
  reason?: string;
  createdAt: Date;
  expiresAt?: Date;
}

const BanSchema: Schema<IBan> = new Schema({
  serverId: { type: Schema.Types.ObjectId, ref: 'Server', required: true },
  userId: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  bannedBy: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  reason: { type: String },
  createdAt: { type: Date, default: Date.now },
  expiresAt: { type: Date },
});

export default (mongoose.models.Ban as Model<IBan>) || mongoose.model<IBan>('Ban', BanSchema); 